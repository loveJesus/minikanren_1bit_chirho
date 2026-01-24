{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeFamilies #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}
{-# LANGUAGE StandaloneDeriving #-}
{-# LANGUAGE UndecidableInstances #-}

-- | Hardware Hash Consing Module ☧
--
-- Enables infinite domains via demand-driven term creation.
-- This module implements a hardware-friendly hash-consing system
-- with O(1) amortized intern/lookup operations.
--
-- Architecture:
--   - Term Store: Block RAM storing term structures
--   - CAM: Content-Addressable Memory for deduplication
--   - Free Pointer: Tracks next available term ID
--   - Reference Counter: Optional GC support

module HashConsChirho where

import Clash.Prelude

-- | Term ID: unique identifier for interned terms
newtype TermIdChirho = TermIdChirho (Unsigned 16)
  deriving (Show, Eq, Generic, NFDataX, BitPack)

-- | Term tag indicating structure type (2-bit encoding)
-- We use Unsigned 2 internally for clean BitPack support
data TermTagChirho
  = NilChirho      -- ^ Empty list / nil (0)
  | SymbolChirho   -- ^ Symbol (left child = symbol ID) (1)
  | ConsChirho     -- ^ Cons cell (left, right children) (2)
  | VarChirho      -- ^ Logic variable (left child = var ID) (3)
  deriving (Show, Eq, Generic, NFDataX, Enum, Bounded)

-- Manual BitPack instance for TermTagChirho
instance BitPack TermTagChirho where
  type BitSize TermTagChirho = 2
  pack tChirho = case tChirho of
    NilChirho    -> 0
    SymbolChirho -> 1
    ConsChirho   -> 2
    VarChirho    -> 3
  unpack bChirho = case bChirho of
    0 -> NilChirho
    1 -> SymbolChirho
    2 -> ConsChirho
    _ -> VarChirho

-- | A term structure stored in RAM
-- BitSize = 2 (tag) + 16 (left) + 16 (right) = 34 bits
data TermChirho = TermChirho
  { tagChirho   :: TermTagChirho
  , leftChirho  :: TermIdChirho
  , rightChirho :: TermIdChirho
  } deriving (Show, Eq, Generic, NFDataX)

-- Manual BitPack instance for TermChirho
instance BitPack TermChirho where
  type BitSize TermChirho = 34  -- 2 + 16 + 16
  pack (TermChirho tChirho lChirho rChirho) =
    pack tChirho ++# pack lChirho ++# pack rChirho
  unpack bChirho =
    let (tBitsChirho, restChirho) = split bChirho :: (BitVector 2, BitVector 32)
        (lBitsChirho, rBitsChirho) = split restChirho :: (BitVector 16, BitVector 16)
    in TermChirho (unpack tBitsChirho) (unpack lBitsChirho) (unpack rBitsChirho)

-- | CAM entry for hash table
data CamEntryChirho = CamEntryChirho
  { camValidChirho :: Bool
  , camTermChirho  :: TermChirho
  , camIdChirho    :: TermIdChirho
  } deriving (Show, Eq, Generic, NFDataX)

-- | Commands to the hash cons unit
data HcCmdChirho
  = HcNopChirho                        -- ^ No operation
  | HcInternChirho TermChirho          -- ^ Intern a term (hash-cons)
  | HcDerefChirho TermIdChirho         -- ^ Dereference a term ID
  | HcIncRefChirho TermIdChirho        -- ^ Increment reference count
  | HcDecRefChirho TermIdChirho        -- ^ Decrement reference count
  deriving (Show, Eq, Generic, NFDataX)

-- | Response from hash cons unit
data HcRespChirho = HcRespChirho
  { respIdChirho    :: TermIdChirho    -- ^ Result term ID
  , respTermChirho  :: TermChirho      -- ^ Dereferenced term
  , respFoundChirho :: Bool            -- ^ True if term already existed
  , respDoneChirho  :: Bool            -- ^ Operation complete
  } deriving (Show, Eq, Generic, NFDataX)

-- | Hash cons unit state
data HcStateChirho = HcStateChirho
  { freePtrChirho  :: TermIdChirho
  , busyChirho     :: Bool
  , resultChirho   :: HcRespChirho
  } deriving (Show, Eq, Generic, NFDataX)

-- | Initial state with nil pre-allocated at ID 0
initialHcStateChirho :: HcStateChirho
initialHcStateChirho = HcStateChirho
  { freePtrChirho = TermIdChirho 1  -- 0 reserved for nil
  , busyChirho = False
  , resultChirho = HcRespChirho
      { respIdChirho = TermIdChirho 0
      , respTermChirho = nilTermChirho
      , respFoundChirho = False
      , respDoneChirho = False
      }
  }

-- | The nil term (pre-allocated)
nilTermChirho :: TermChirho
nilTermChirho = TermChirho NilChirho (TermIdChirho 0) (TermIdChirho 0)

-- | Compute hash from term (XOR-based, 6-bit output for 64-entry CAM)
hashTermChirho :: TermChirho -> Unsigned 6
hashTermChirho (TermChirho tChirho (TermIdChirho lChirho) (TermIdChirho rChirho)) =
  let tagBitsChirho = resize (unpack (pack tChirho) :: Unsigned 2) :: Unsigned 16
  in truncateB $ tagBitsChirho `xor` lChirho `xor` rChirho

-- | Check if two terms are equal (for CAM matching)
termEqChirho :: TermChirho -> TermChirho -> Bool
termEqChirho t1Chirho t2Chirho =
  tagChirho t1Chirho == tagChirho t2Chirho &&
  leftChirho t1Chirho == leftChirho t2Chirho &&
  rightChirho t1Chirho == rightChirho t2Chirho

-- | CAM lookup: check if term exists at computed hash location
camLookupChirho
  :: Vec 64 CamEntryChirho  -- ^ CAM contents
  -> TermChirho             -- ^ Term to look up
  -> Maybe TermIdChirho     -- ^ Just id if found, Nothing otherwise
camLookupChirho camChirho termChirho =
  let hChirho = hashTermChirho termChirho
      entryChirho = camChirho !! hChirho
  in if camValidChirho entryChirho && termEqChirho (camTermChirho entryChirho) termChirho
     then Just (camIdChirho entryChirho)
     else Nothing

-- | Update CAM with new entry
camInsertChirho
  :: Vec 64 CamEntryChirho  -- ^ Old CAM
  -> TermChirho             -- ^ Term to insert
  -> TermIdChirho           -- ^ ID to associate
  -> Vec 64 CamEntryChirho  -- ^ Updated CAM
camInsertChirho camChirho termChirho tidChirho =
  let hChirho = hashTermChirho termChirho
      newEntryChirho = CamEntryChirho True termChirho tidChirho
  in replace hChirho newEntryChirho camChirho

-- | Empty CAM
emptyCamChirho :: Vec 64 CamEntryChirho
emptyCamChirho = repeat $ CamEntryChirho False nilTermChirho (TermIdChirho 0)

-- | Block RAM for term storage (1024 terms)
type TermStoreChirho = Vec 1024 TermChirho

-- | Empty term store (all nil)
emptyTermStoreChirho :: TermStoreChirho
emptyTermStoreChirho = repeat nilTermChirho

-- | Block RAM for reference counts
type RefCountsChirho = Vec 1024 (Unsigned 8)

-- | Initialize ref counts (nil has count 1)
initialRefCountsChirho :: RefCountsChirho
initialRefCountsChirho = replace (0 :: Index 1024) (1 :: Unsigned 8) (repeat 0)

-- | Complete hash cons unit state including memories
data HashConsUnitChirho = HashConsUnitChirho
  { hcuStateChirho     :: HcStateChirho
  , hcuTermStoreChirho :: TermStoreChirho
  , hcuCamChirho       :: Vec 64 CamEntryChirho
  , hcuRefCountsChirho :: RefCountsChirho
  } deriving (Show, Generic, NFDataX)

-- | Initial hash cons unit
initialHashConsUnitChirho :: HashConsUnitChirho
initialHashConsUnitChirho = HashConsUnitChirho
  { hcuStateChirho = initialHcStateChirho
  , hcuTermStoreChirho = emptyTermStoreChirho
  , hcuCamChirho = emptyCamChirho
  , hcuRefCountsChirho = initialRefCountsChirho
  }

-- | Main hash consing transition function
-- This is the core Mealy machine for the hash cons unit
hashConsStepChirho
  :: HashConsUnitChirho     -- ^ Current state
  -> HcCmdChirho            -- ^ Input command
  -> (HashConsUnitChirho, HcRespChirho)  -- ^ (New state, Output)
hashConsStepChirho unitChirho cmdChirho = case cmdChirho of

  HcNopChirho -> (unitChirho, defaultRespChirho)

  HcInternChirho termChirho ->
    case camLookupChirho (hcuCamChirho unitChirho) termChirho of
      Just tidChirho ->
        -- Term already exists, return existing ID
        (unitChirho, HcRespChirho tidChirho termChirho True True)
      Nothing ->
        -- Allocate new term
        let TermIdChirho ptrChirho = freePtrChirho (hcuStateChirho unitChirho)
            newTidChirho = TermIdChirho ptrChirho
            newPtrChirho = TermIdChirho (ptrChirho + 1)
            -- Update memories
            newStoreChirho = replace ptrChirho termChirho (hcuTermStoreChirho unitChirho)
            newCamChirho = camInsertChirho (hcuCamChirho unitChirho) termChirho newTidChirho
            newRefsChirho = replace ptrChirho 1 (hcuRefCountsChirho unitChirho)
            -- Update state
            newStateChirho = (hcuStateChirho unitChirho) { freePtrChirho = newPtrChirho }
            newUnitChirho = unitChirho
              { hcuStateChirho = newStateChirho
              , hcuTermStoreChirho = newStoreChirho
              , hcuCamChirho = newCamChirho
              , hcuRefCountsChirho = newRefsChirho
              }
        in (newUnitChirho, HcRespChirho newTidChirho termChirho False True)

  HcDerefChirho (TermIdChirho tidChirho) ->
    let termChirho = hcuTermStoreChirho unitChirho !! tidChirho
    in (unitChirho, HcRespChirho (TermIdChirho tidChirho) termChirho True True)

  HcIncRefChirho (TermIdChirho tidChirho) ->
    let oldCountChirho = hcuRefCountsChirho unitChirho !! tidChirho
        newRefsChirho = replace tidChirho (oldCountChirho + 1) (hcuRefCountsChirho unitChirho)
        newUnitChirho = unitChirho { hcuRefCountsChirho = newRefsChirho }
    in (newUnitChirho, defaultRespChirho { respDoneChirho = True })

  HcDecRefChirho (TermIdChirho tidChirho) ->
    let oldCountChirho = hcuRefCountsChirho unitChirho !! tidChirho
        newRefsChirho = replace tidChirho (oldCountChirho - 1) (hcuRefCountsChirho unitChirho)
        newUnitChirho = unitChirho { hcuRefCountsChirho = newRefsChirho }
    in (newUnitChirho, defaultRespChirho { respDoneChirho = True })

  where
    defaultRespChirho = HcRespChirho (TermIdChirho 0) nilTermChirho False False

-- | Convenience: create a cons cell
mkConsChirho :: TermIdChirho -> TermIdChirho -> TermChirho
mkConsChirho lChirho rChirho = TermChirho ConsChirho lChirho rChirho

-- | Convenience: create a symbol
mkSymbolChirho :: Unsigned 16 -> TermChirho
mkSymbolChirho sidChirho = TermChirho SymbolChirho (TermIdChirho sidChirho) (TermIdChirho 0)

-- | Convenience: create a variable
mkVarChirho :: Unsigned 16 -> TermChirho
mkVarChirho vidChirho = TermChirho VarChirho (TermIdChirho vidChirho) (TermIdChirho 0)

-- | Top-level entity: hash cons unit as Mealy machine
-- Can be synthesized to hardware
{-# ANN hashConsTopChirho
  (Synthesize
    { t_name   = "hashconsChirho"
    , t_inputs = [PortName "clk", PortName "rst", PortName "enChirho", PortName "cmdChirho"]
    , t_output = PortName "respChirho"
    }) #-}
hashConsTopChirho
  :: Clock System
  -> Reset System
  -> Enable System
  -> Signal System HcCmdChirho
  -> Signal System HcRespChirho
hashConsTopChirho clkChirho rstChirho enChirho =
  exposeClockResetEnable (mealy hashConsStepChirho initialHashConsUnitChirho) clkChirho rstChirho enChirho

-- | Test bench: intern some terms and verify structural sharing
testHashConsChirho :: [HcRespChirho]
testHashConsChirho =
  let cmdsChirho =
        [ HcInternChirho (mkSymbolChirho 1)        -- a
        , HcInternChirho (mkSymbolChirho 2)        -- b
        , HcInternChirho (mkConsChirho (TermIdChirho 1) (TermIdChirho 0))  -- [a]
        , HcInternChirho (mkConsChirho (TermIdChirho 1) (TermIdChirho 0))  -- [a] again (should find)
        ]
      goChirho _unitChirho [] = []
      goChirho unitChirho (cChirho:csChirho) =
        let (unitChirho', respChirho) = hashConsStepChirho unitChirho cChirho
        in respChirho : goChirho unitChirho' csChirho
  in goChirho initialHashConsUnitChirho cmdsChirho

-- Soli Deo Gloria ☧
