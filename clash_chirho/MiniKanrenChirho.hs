{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE TemplateHaskell #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}
{-# LANGUAGE KindSignatures #-}

{- |
Module      : MiniKanrenChirho
Description : miniKanren as 1-bit matrix operations for FPGA ☧
Copyright   : (c) 2024
License     : MIT

Clash implementation of miniKanren search primitives.
Compiles to Verilog/VHDL for FPGA synthesis.

"Whether therefore ye eat, or drink, or whatsoever ye do,
 do all to the glory of God." — 1 Corinthians 10:31
-}
module MiniKanrenChirho where

import Clash.Prelude

-- | 64-bit domain representing possible values for a variable
-- Bit i = 1 means value i is possible
type DomainChirho = BitVector 64

-- | Variable index (supports up to 8 variables)
type VarIdxChirho = Index 8

-- | Search state: N variables with 64-bit domains
data SearchStateChirho (n :: Nat) = SearchStateChirho
  { domainsChirho :: Vec n DomainChirho
  , validChirho   :: Bool
  } deriving (Generic, NFDataX, Show, Eq, Bundle)

-- | Full domain: all values possible
fullDomainChirho :: DomainChirho
fullDomainChirho = maxBound  -- All 1s

-- | Empty domain: no values possible (failure)
emptyDomainChirho :: DomainChirho
emptyDomainChirho = 0

-- | Initialize search state with all full domains
initStateChirho :: KnownNat n => SearchStateChirho n
initStateChirho = SearchStateChirho
  { domainsChirho = repeat fullDomainChirho
  , validChirho = True
  }

-- | Unification: AND two domains together (single cycle)
-- Core operation: domain1 ∩ domain2
unifyChirho :: DomainChirho -> DomainChirho -> DomainChirho
unifyChirho d1Chirho d2Chirho = d1Chirho .&. d2Chirho

-- | Check if domain is empty (failure)
isEmptyChirho :: DomainChirho -> Bool
isEmptyChirho dChirho = dChirho == 0

-- | Check if domain is singleton (exactly one value)
isSingletonChirho :: DomainChirho -> Bool
isSingletonChirho xChirho =
  xChirho /= 0 && (xChirho .&. (xChirho - 1)) == 0

-- | Get lowest set bit (isolate one possible value)
lowestBitChirho :: DomainChirho -> DomainChirho
lowestBitChirho xChirho = xChirho .&. negate xChirho

-- | Clear lowest set bit (remaining possibilities)
clearLowestChirho :: DomainChirho -> DomainChirho
clearLowestChirho xChirho = xChirho .&. (xChirho - 1)

-- | Fork: split domain into two branches
-- Returns (lowest single value, remaining values)
forkChirho :: DomainChirho -> (DomainChirho, DomainChirho)
forkChirho xChirho = (lowestBitChirho xChirho, clearLowestChirho xChirho)

-- | Disjunction: OR two domains (for conde)
disjChirho :: DomainChirho -> DomainChirho -> DomainChirho
disjChirho d1Chirho d2Chirho = d1Chirho .|. d2Chirho

-- | Population count: number of possible values
popCountChirho :: DomainChirho -> Index 65
popCountChirho = fromIntegral . popCount

-- | Unify two variables in a search state
unifyVarsChirho
  :: KnownNat n
  => Index n
  -> Index n
  -> SearchStateChirho n
  -> SearchStateChirho n
unifyVarsChirho v1Chirho v2Chirho stateChirho
  | not (validChirho stateChirho) = stateChirho  -- Already failed
  | otherwise =
      let d1Chirho = (domainsChirho stateChirho) !! v1Chirho
          d2Chirho = (domainsChirho stateChirho) !! v2Chirho
          resultChirho = unifyChirho d1Chirho d2Chirho
          newDomainsChirho = replace v1Chirho resultChirho
                           $ replace v2Chirho resultChirho
                           $ domainsChirho stateChirho
      in SearchStateChirho
          { domainsChirho = newDomainsChirho
          , validChirho = not (isEmptyChirho resultChirho)
          }

-- | Constrain a variable to a specific value
constrainChirho
  :: KnownNat n
  => Index n
  -> DomainChirho
  -> SearchStateChirho n
  -> SearchStateChirho n
constrainChirho vChirho maskChirho stateChirho
  | not (validChirho stateChirho) = stateChirho
  | otherwise =
      let dChirho = (domainsChirho stateChirho) !! vChirho
          resultChirho = unifyChirho dChirho maskChirho
          newDomainsChirho = replace vChirho resultChirho (domainsChirho stateChirho)
      in SearchStateChirho
          { domainsChirho = newDomainsChirho
          , validChirho = not (isEmptyChirho resultChirho)
          }

-- | Branch on a variable: create two alternative states
branchChirho
  :: KnownNat n
  => Index n
  -> SearchStateChirho n
  -> (SearchStateChirho n, SearchStateChirho n)
branchChirho vChirho stateChirho
  | not (validChirho stateChirho) = (stateChirho, stateChirho)
  | otherwise =
      let dChirho = (domainsChirho stateChirho) !! vChirho
          (loChirho, hiChirho) = forkChirho dChirho
          state1Chirho = constrainChirho vChirho loChirho stateChirho
          state2Chirho = constrainChirho vChirho hiChirho stateChirho
      in (state1Chirho, state2Chirho)

-- | Check if state represents a complete solution (all singletons)
isSolutionChirho :: KnownNat n => SearchStateChirho n -> Bool
isSolutionChirho stateChirho =
  validChirho stateChirho &&
  all isSingletonChirho (domainsChirho stateChirho)

-- | Extract solution values (assumes isSolutionChirho is True)
extractSolutionChirho :: KnownNat n => SearchStateChirho n -> Vec n (Index 64)
extractSolutionChirho stateChirho =
  map (fromIntegral . countTrailingZeros) (domainsChirho stateChirho)

-------------------------------------------------------------------------------
-- Hardware: Mealy machine for search engine
-------------------------------------------------------------------------------

-- | Search engine command
data SearchCmdChirho
  = InitChirho                             -- Reset to initial state
  | UnifyVarsChirho VarIdxChirho VarIdxChirho  -- Unify two variables
  | ConstrainVarChirho VarIdxChirho DomainChirho  -- Constrain variable to domain
  | BranchVarChirho VarIdxChirho           -- Branch on variable (push to stack)
  | BacktrackChirho                        -- Pop from stack and continue
  | NopChirho                              -- No operation
  deriving (Generic, NFDataX, Show, Eq, Bundle)

-- | Search engine response
data SearchRespChirho = SearchRespChirho
  { respValidChirho    :: Bool
  , respSolutionChirho :: Bool
  , respDomainsChirho  :: Vec 8 DomainChirho
  } deriving (Generic, NFDataX, Show, Eq, Bundle)

-- | Stack entry for backtracking
data StackEntryChirho = StackEntryChirho
  { stackVarChirho     :: VarIdxChirho
  , stackDomainChirho  :: DomainChirho
  , stackDomainsChirho :: Vec 8 DomainChirho
  } deriving (Generic, NFDataX, Show, Eq, Bundle)

-- | Search engine state
data EngineStateChirho = EngineStateChirho
  { engDomainsChirho :: Vec 8 DomainChirho
  , engValidChirho   :: Bool
  , engStackChirho   :: Vec 16 (Maybe StackEntryChirho)
  , engSPChirho      :: Index 16
  } deriving (Generic, NFDataX, Show, Eq, Bundle)

-- | Initial engine state
initEngineChirho :: EngineStateChirho
initEngineChirho = EngineStateChirho
  { engDomainsChirho = repeat fullDomainChirho
  , engValidChirho = True
  , engStackChirho = repeat Nothing
  , engSPChirho = 0
  }

-- | Search engine transition function (Mealy machine)
engineStepChirho :: EngineStateChirho -> SearchCmdChirho -> (EngineStateChirho, SearchRespChirho)
engineStepChirho stChirho cmdChirho = (stChirho', respChirho)
  where
    -- Process command
    stChirho' = case cmdChirho of
      InitChirho -> initEngineChirho

      UnifyVarsChirho v1Chirho v2Chirho ->
        let d1Chirho = (engDomainsChirho stChirho) !! v1Chirho
            d2Chirho = (engDomainsChirho stChirho) !! v2Chirho
            resultChirho = unifyChirho d1Chirho d2Chirho
            newDomainsChirho = replace v1Chirho resultChirho
                              $ replace v2Chirho resultChirho
                              $ engDomainsChirho stChirho
        in stChirho
            { engDomainsChirho = newDomainsChirho
            , engValidChirho = engValidChirho stChirho && not (isEmptyChirho resultChirho)
            }

      ConstrainVarChirho vChirho maskChirho ->
        let dChirho = (engDomainsChirho stChirho) !! vChirho
            resultChirho = unifyChirho dChirho maskChirho
            newDomainsChirho = replace vChirho resultChirho (engDomainsChirho stChirho)
        in stChirho
            { engDomainsChirho = newDomainsChirho
            , engValidChirho = engValidChirho stChirho && not (isEmptyChirho resultChirho)
            }

      BranchVarChirho vChirho ->
        let dChirho = (engDomainsChirho stChirho) !! vChirho
            (loChirho, hiChirho) = forkChirho dChirho
            -- Push alternative to stack
            entryChirho = StackEntryChirho vChirho hiChirho (engDomainsChirho stChirho)
            newStackChirho = replace (engSPChirho stChirho) (Just entryChirho) (engStackChirho stChirho)
            newSPChirho = engSPChirho stChirho + 1
            -- Continue with first choice
            newDomainsChirho = replace vChirho loChirho (engDomainsChirho stChirho)
        in stChirho
            { engDomainsChirho = newDomainsChirho
            , engStackChirho = newStackChirho
            , engSPChirho = newSPChirho
            , engValidChirho = engValidChirho stChirho && not (isEmptyChirho loChirho)
            }

      BacktrackChirho ->
        if engSPChirho stChirho == 0
          then stChirho { engValidChirho = False }  -- No more alternatives
          else
            let newSPChirho = engSPChirho stChirho - 1
                entryChirho = (engStackChirho stChirho) !! newSPChirho
            in case entryChirho of
                Nothing -> stChirho { engValidChirho = False }
                Just eChirho -> stChirho
                    { engDomainsChirho = replace (stackVarChirho eChirho)
                                                 (stackDomainChirho eChirho)
                                                 (stackDomainsChirho eChirho)
                    , engSPChirho = newSPChirho
                    , engValidChirho = not (isEmptyChirho (stackDomainChirho eChirho))
                    }

      NopChirho -> stChirho

    -- Build response
    respChirho = SearchRespChirho
      { respValidChirho = engValidChirho stChirho'
      , respSolutionChirho = engValidChirho stChirho' &&
                             all isSingletonChirho (engDomainsChirho stChirho')
      , respDomainsChirho = engDomainsChirho stChirho'
      }

-- | Top-level 64-bit search engine (synthesizable)
-- NOTE: This is the LEGACY 64-bit engine. For larger domains, use:
--   - intersect_hier_262k_chirho (512² = 262K values)
--   - intersect_hier_65k_chirho (256² = 65K values)
{-# ANN searchEngine64BitChirho
  (Synthesize
    { t_name   = "searchEngine64BitChirho"
    , t_inputs = [PortName "clk", PortName "rst", PortName "enChirho", PortName "cmdChirho"]
    , t_output = PortName "respChirho"
    }) #-}
searchEngine64BitChirho
  :: Clock System
  -> Reset System
  -> Enable System
  -> Signal System SearchCmdChirho
  -> Signal System SearchRespChirho
searchEngine64BitChirho clkChirho rstChirho enChirho =
  exposeClockResetEnable (mealy engineStepChirho initEngineChirho) clkChirho rstChirho enChirho

-------------------------------------------------------------------------------
-- Soli Deo Gloria ☧
-------------------------------------------------------------------------------
