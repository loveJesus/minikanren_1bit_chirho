{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}
{-# LANGUAGE RecordWildCards #-}

{- |
Module      : HbmEngineChirho
Description : General-purpose constraint engine with HBM storage ☧
Copyright   : (c) 2026
License     : MIT

HBM-backed miniKanren acceleration engine.

Design philosophy:
- CPU controls search strategy (branching, backtracking)
- FPGA accelerates bulk constraint operations
- HBM stores working state (domains, terms, tabling cache)

This is NOT a full miniKanren reimplementation - it's an accelerator
for the computationally intensive parts.

"For God so loved the world..." - John 3:16
-}
module HbmEngineChirho where

import Clash.Prelude

-- ============================================================================
-- Core Types ☧
-- ============================================================================

-- | 64-bit domain (which values are possible for a variable)
type DomainChirho = BitVector 64

-- | Variable ID (supports up to 64K variables with HBM)
type VarIdChirho = BitVector 16

-- | Term ID for hash-consed terms (supports 16M terms)
type TermIdChirho = BitVector 24

-- | HBM address (34 bits for 16GB)
type HbmAddrChirho = BitVector 34

-- ============================================================================
-- Command Interface (CPU → FPGA) ☧
-- ============================================================================

-- | Operations the CPU can request
data OpCodeChirho
  = OpNopChirho           -- No operation
  | OpLoadStateChirho     -- Load state from HBM address
  | OpStoreStateChirho    -- Store state to HBM address
  | OpConstrainChirho     -- Apply domain constraint: var &= mask
  | OpUnifyVarsChirho     -- Unify two variables: var1.domain &= var2.domain
  | OpBatchConstrainChirho -- Apply N constraints from HBM
  | OpCheckFailureChirho  -- Check if any domain is empty
  | OpFindBranchChirho    -- Find variable with smallest domain > 1
  | OpForkChirho          -- Split domain: return (singleton, rest)
  deriving (Generic, NFDataX, Eq, Show)

-- | Command from CPU to FPGA
data CommandChirho = CommandChirho
  { cmdOpcodeChirho   :: OpCodeChirho
  , cmdVar1Chirho     :: VarIdChirho      -- First variable
  , cmdVar2Chirho     :: VarIdChirho      -- Second variable (for unify)
  , cmdMaskChirho     :: DomainChirho     -- Domain mask (for constrain)
  , cmdAddrChirho     :: HbmAddrChirho    -- HBM address (for load/store)
  , cmdCountChirho    :: BitVector 16     -- Count (for batch operations)
  } deriving (Generic, NFDataX, Eq, Show)

-- | Default/idle command
idleCommandChirho :: CommandChirho
idleCommandChirho = CommandChirho
  { cmdOpcodeChirho = OpNopChirho
  , cmdVar1Chirho   = 0
  , cmdVar2Chirho   = 0
  , cmdMaskChirho   = maxBound
  , cmdAddrChirho   = 0
  , cmdCountChirho  = 0
  }

-- ============================================================================
-- Response Interface (FPGA → CPU) ☧
-- ============================================================================

-- | Status flags
data StatusChirho = StatusChirho
  { statReadyChirho    :: Bool    -- Ready for next command
  , statValidChirho    :: Bool    -- Current state is valid (no empty domains)
  , statSolutionChirho :: Bool    -- All domains are singletons
  , statFailedChirho   :: Bool    -- At least one domain is empty
  , statBranchVarChirho :: VarIdChirho  -- Best variable to branch on
  , statBranchSizeChirho :: BitVector 7 -- Domain size of branch var (0-64)
  } deriving (Generic, NFDataX, Eq, Show)

-- | Response from FPGA to CPU
data ResponseChirho = ResponseChirho
  { respStatusChirho  :: StatusChirho
  , respDomainChirho  :: DomainChirho   -- Result domain (for fork, etc.)
  , respDomain2Chirho :: DomainChirho   -- Second result (remainder after fork)
  } deriving (Generic, NFDataX, Eq, Show)

-- ============================================================================
-- HBM Interface ☧
-- ============================================================================

-- | Simplified HBM port (one channel)
data HbmPortChirho = HbmPortChirho
  { hbmAddrOutChirho   :: HbmAddrChirho  -- Address to read/write
  , hbmWriteEnChirho   :: Bool           -- Write enable
  , hbmReadEnChirho    :: Bool           -- Read enable
  , hbmWdataChirho     :: BitVector 256  -- Write data (256-bit bus)
  , hbmRdataChirho     :: BitVector 256  -- Read data (from HBM)
  , hbmRvalidChirho    :: Bool           -- Read data valid
  , hbmWreadyChirho    :: Bool           -- Write accepted
  } deriving (Generic, NFDataX, Eq, Show)

-- | Idle HBM port
idleHbmPortChirho :: HbmPortChirho
idleHbmPortChirho = HbmPortChirho
  { hbmAddrOutChirho = 0
  , hbmWriteEnChirho = False
  , hbmReadEnChirho  = False
  , hbmWdataChirho   = 0
  , hbmRdataChirho   = 0
  , hbmRvalidChirho  = False
  , hbmWreadyChirho  = True
  }

-- ============================================================================
-- Engine State ☧
-- ============================================================================

-- | Number of variables supported (configurable)
type NumVarsChirho = 64

-- | Internal engine state
data EngineStateChirho = EngineStateChirho
  { engDomainsChirho   :: Vec NumVarsChirho DomainChirho  -- Variable domains
  , engValidChirho     :: Bool                            -- No empty domains
  , engPhaseChirho     :: EnginePhaseChirho               -- Current phase
  , engPendingChirho   :: BitVector 16                    -- Pending batch ops
  } deriving (Generic, NFDataX)

-- | Engine execution phase
data EnginePhaseChirho
  = PhaseIdleChirho
  | PhaseLoadingChirho
  | PhaseComputingChirho
  | PhaseStoringChirho
  | PhaseBatchChirho
  deriving (Generic, NFDataX, Eq, Show)

-- | Initial engine state (all domains full)
initEngineChirho :: EngineStateChirho
initEngineChirho = EngineStateChirho
  { engDomainsChirho = repeat maxBound  -- All 1s = all values possible
  , engValidChirho   = True
  , engPhaseChirho   = PhaseIdleChirho
  , engPendingChirho = 0
  }

-- ============================================================================
-- Core Operations (Combinational) ☧
-- ============================================================================

-- | Domain intersection (unification)
intersectChirho :: DomainChirho -> DomainChirho -> DomainChirho
intersectChirho d1Chirho d2Chirho = d1Chirho .&. d2Chirho

-- | Check if domain is empty (failure)
isEmptyDomainChirho :: DomainChirho -> Bool
isEmptyDomainChirho dChirho = dChirho == 0

-- | Check if domain is singleton (exactly one value)
isSingletonChirho :: DomainChirho -> Bool
isSingletonChirho dChirho = dChirho /= 0 && (dChirho .&. (dChirho - 1)) == 0

-- | Get lowest set bit (one possible value)
lowestBitChirho :: DomainChirho -> DomainChirho
lowestBitChirho dChirho = dChirho .&. negate dChirho

-- | Clear lowest bit (remaining possibilities)
clearLowestChirho :: DomainChirho -> DomainChirho
clearLowestChirho dChirho = dChirho .&. (dChirho - 1)

-- | Fork domain into (one choice, remaining choices)
forkDomainChirho :: DomainChirho -> (DomainChirho, DomainChirho)
forkDomainChirho dChirho = (lowestBitChirho dChirho, clearLowestChirho dChirho)

-- | Population count (number of possible values)
domainSizeChirho :: DomainChirho -> BitVector 7
domainSizeChirho dChirho = fromIntegral (popCount dChirho)

-- | Check if all domains are singletons (solution found)
allSingletonsChirho :: Vec n DomainChirho -> Bool
allSingletonsChirho domainsChirho = fold (&&) (map isSingletonChirho domainsChirho)

-- | Check if any domain is empty (failure)
anyEmptyChirho :: Vec n DomainChirho -> Bool
anyEmptyChirho domainsChirho = fold (||) (map isEmptyDomainChirho domainsChirho)

-- | Find variable with smallest non-singleton domain (best to branch on)
-- Returns (variable index, domain size)
findBranchVarChirho
  :: KnownNat n
  => Vec n DomainChirho
  -> (Index n, BitVector 7)
findBranchVarChirho domainsChirho =
  let sizesChirho = map domainSizeChirho domainsChirho
      -- Find smallest size > 1 (not singleton, not empty)
      validSizesChirho = zipWith
        (\sChirho dChirho -> if sChirho > 1 then sChirho else 127)
        sizesChirho
        domainsChirho
      minIdxChirho = fold
        (\(i1Chirho, s1Chirho) (i2Chirho, s2Chirho) ->
          if s1Chirho <= s2Chirho then (i1Chirho, s1Chirho) else (i2Chirho, s2Chirho))
        (zip (indicesI :: Vec n (Index n)) validSizesChirho)
  in minIdxChirho

-- ============================================================================
-- Engine FSM (Sequential) ☧
-- ============================================================================

-- | Apply a single command to engine state
applyCommandChirho
  :: CommandChirho
  -> EngineStateChirho
  -> EngineStateChirho
applyCommandChirho cmdChirho stateChirho = case cmdOpcodeChirho cmdChirho of

  OpNopChirho -> stateChirho

  OpConstrainChirho ->
    let varIdxChirho = unpack (resize (cmdVar1Chirho cmdChirho)) :: Index NumVarsChirho
        oldDomainChirho = (engDomainsChirho stateChirho) !! varIdxChirho
        newDomainChirho = intersectChirho oldDomainChirho (cmdMaskChirho cmdChirho)
        newDomainsChirho = replace varIdxChirho newDomainChirho (engDomainsChirho stateChirho)
    in stateChirho
      { engDomainsChirho = newDomainsChirho
      , engValidChirho = not (anyEmptyChirho newDomainsChirho)
      }

  OpUnifyVarsChirho ->
    let idx1Chirho = unpack (resize (cmdVar1Chirho cmdChirho)) :: Index NumVarsChirho
        idx2Chirho = unpack (resize (cmdVar2Chirho cmdChirho)) :: Index NumVarsChirho
        d1Chirho = (engDomainsChirho stateChirho) !! idx1Chirho
        d2Chirho = (engDomainsChirho stateChirho) !! idx2Chirho
        intersectedChirho = intersectChirho d1Chirho d2Chirho
        newDomainsChirho = replace idx1Chirho intersectedChirho
                        $ replace idx2Chirho intersectedChirho
                        $ engDomainsChirho stateChirho
    in stateChirho
      { engDomainsChirho = newDomainsChirho
      , engValidChirho = not (anyEmptyChirho newDomainsChirho)
      }

  _ -> stateChirho  -- Other ops need HBM, handled by FSM

-- | Generate response from current state
generateResponseChirho
  :: CommandChirho
  -> EngineStateChirho
  -> ResponseChirho
generateResponseChirho cmdChirho stateChirho =
  let domainsChirho = engDomainsChirho stateChirho
      (branchIdxChirho, branchSizeChirho) = findBranchVarChirho domainsChirho
      varIdxChirho = unpack (resize (cmdVar1Chirho cmdChirho)) :: Index NumVarsChirho
      varDomainChirho = domainsChirho !! varIdxChirho
      (forkOneChirho, forkRestChirho) = forkDomainChirho varDomainChirho
  in ResponseChirho
    { respStatusChirho = StatusChirho
      { statReadyChirho = engPhaseChirho stateChirho == PhaseIdleChirho
      , statValidChirho = engValidChirho stateChirho
      , statSolutionChirho = allSingletonsChirho domainsChirho
      , statFailedChirho = anyEmptyChirho domainsChirho
      , statBranchVarChirho = pack (resize (pack branchIdxChirho))
      , statBranchSizeChirho = branchSizeChirho
      }
    , respDomainChirho = case cmdOpcodeChirho cmdChirho of
        OpForkChirho -> forkOneChirho
        _            -> varDomainChirho
    , respDomain2Chirho = forkRestChirho
    }

-- ============================================================================
-- Top-Level Engine ☧
-- ============================================================================

-- | HBM-backed constraint engine
--
-- Inputs:
--   - Command from CPU (what operation to perform)
--   - HBM read data (when loading state)
--
-- Outputs:
--   - Response to CPU (status, results)
--   - HBM port (for reading/writing state)
--
hbmEngineChirho
  :: Clock XilinxSystem
  -> Reset XilinxSystem
  -> Signal XilinxSystem CommandChirho
  -> Signal XilinxSystem (BitVector 256)  -- HBM read data
  -> Signal XilinxSystem Bool             -- HBM read valid
  -> ( Signal XilinxSystem ResponseChirho
     , Signal XilinxSystem HbmAddrChirho  -- HBM address
     , Signal XilinxSystem Bool           -- HBM read enable
     , Signal XilinxSystem Bool           -- HBM write enable
     , Signal XilinxSystem (BitVector 256) -- HBM write data
     )
hbmEngineChirho clkChirho rstChirho cmdChirho hbmRdataChirho hbmRvalidChirho =
  ( respChirho
  , pure 0        -- HBM address (TODO: implement load/store)
  , pure False    -- HBM read enable
  , pure False    -- HBM write enable
  , pure 0        -- HBM write data
  )
  where
    -- State register
    stateChirho = register clkChirho rstChirho enableGen initEngineChirho nextStateChirho

    -- Next state logic
    nextStateChirho = applyCommandChirho <$> cmdChirho <*> stateChirho

    -- Response generation
    respChirho = generateResponseChirho <$> cmdChirho <*> stateChirho

-- ============================================================================
-- Synthesis Wrapper ☧
-- ============================================================================

{-# ANN topEntityChirho
  (Synthesize
    { t_name = "hbm_engine_chirho"
    , t_inputs =
      [ PortName "clk_chirho"
      , PortName "rst_chirho"
      , PortName "cmd_chirho"
      , PortName "hbm_rdata_chirho"
      , PortName "hbm_rvalid_chirho"
      ]
    , t_output = PortProduct ""
      [ PortName "resp_chirho"
      , PortName "hbm_addr_chirho"
      , PortName "hbm_ren_chirho"
      , PortName "hbm_wen_chirho"
      , PortName "hbm_wdata_chirho"
      ]
    }) #-}
topEntityChirho
  :: Clock XilinxSystem
  -> Reset XilinxSystem
  -> Signal XilinxSystem CommandChirho
  -> Signal XilinxSystem (BitVector 256)
  -> Signal XilinxSystem Bool
  -> ( Signal XilinxSystem ResponseChirho
     , Signal XilinxSystem HbmAddrChirho
     , Signal XilinxSystem Bool
     , Signal XilinxSystem Bool
     , Signal XilinxSystem (BitVector 256)
     )
topEntityChirho = hbmEngineChirho
