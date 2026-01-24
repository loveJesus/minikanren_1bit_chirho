{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeFamilies #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}

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

module HashCons_chirho where

import Clash.Prelude
import GHC.Generics (Generic)

-- | Term ID: unique identifier for interned terms
newtype TermId_chirho = TermId_chirho (Unsigned 16)
  deriving (Show, Eq, Generic, NFDataX, BitPack)

-- | Term tag indicating structure type
data TermTag_chirho
  = Nil_chirho      -- ^ Empty list / nil
  | Symbol_chirho   -- ^ Symbol (left child = symbol ID)
  | Cons_chirho     -- ^ Cons cell (left, right children)
  | Var_chirho      -- ^ Logic variable (left child = var ID)
  deriving (Show, Eq, Generic, NFDataX, BitPack, Enum, Bounded)

-- | A term structure stored in RAM
data Term_chirho = Term_chirho
  { tag_chirho   :: TermTag_chirho
  , left_chirho  :: TermId_chirho
  , right_chirho :: TermId_chirho
  } deriving (Show, Eq, Generic, NFDataX, BitPack)

-- | CAM entry for hash table
data CamEntry_chirho = CamEntry_chirho
  { camValid_chirho :: Bool
  , camTerm_chirho  :: Term_chirho
  , camId_chirho    :: TermId_chirho
  } deriving (Show, Eq, Generic, NFDataX)

-- | Commands to the hash cons unit
data HcCmd_chirho
  = HcNop_chirho                          -- ^ No operation
  | HcIntern_chirho Term_chirho           -- ^ Intern a term (hash-cons)
  | HcDeref_chirho TermId_chirho          -- ^ Dereference a term ID
  | HcIncRef_chirho TermId_chirho         -- ^ Increment reference count
  | HcDecRef_chirho TermId_chirho         -- ^ Decrement reference count
  deriving (Show, Eq, Generic, NFDataX)

-- | Response from hash cons unit
data HcResp_chirho = HcResp_chirho
  { respId_chirho    :: TermId_chirho     -- ^ Result term ID
  , respTerm_chirho  :: Term_chirho       -- ^ Dereferenced term
  , respFound_chirho :: Bool              -- ^ True if term already existed
  , respDone_chirho  :: Bool              -- ^ Operation complete
  } deriving (Show, Eq, Generic, NFDataX)

-- | Hash cons unit state
data HcState_chirho = HcState_chirho
  { freePtr_chirho    :: TermId_chirho
  , busy_chirho       :: Bool
  , result_chirho     :: HcResp_chirho
  } deriving (Show, Eq, Generic, NFDataX)

-- | Initial state with nil pre-allocated at ID 0
initialHcState_chirho :: HcState_chirho
initialHcState_chirho = HcState_chirho
  { freePtr_chirho = TermId_chirho 1  -- 0 reserved for nil
  , busy_chirho = False
  , result_chirho = HcResp_chirho
      { respId_chirho = TermId_chirho 0
      , respTerm_chirho = nilTerm_chirho
      , respFound_chirho = False
      , respDone_chirho = False
      }
  }

-- | The nil term (pre-allocated)
nilTerm_chirho :: Term_chirho
nilTerm_chirho = Term_chirho Nil_chirho (TermId_chirho 0) (TermId_chirho 0)

-- | Compute hash from term (XOR-based, 6-bit output for 64-entry CAM)
hashTerm_chirho :: Term_chirho -> Unsigned 6
hashTerm_chirho (Term_chirho t (TermId_chirho l) (TermId_chirho r)) =
  truncateB $ resize (pack t) `xor` l `xor` r

-- | Check if two terms are equal (for CAM matching)
termEq_chirho :: Term_chirho -> Term_chirho -> Bool
termEq_chirho t1 t2 =
  tag_chirho t1 == tag_chirho t2 &&
  left_chirho t1 == left_chirho t2 &&
  right_chirho t1 == right_chirho t2

-- | CAM lookup: check if term exists at computed hash location
camLookup_chirho
  :: Vec 64 CamEntry_chirho  -- ^ CAM contents
  -> Term_chirho             -- ^ Term to look up
  -> Maybe TermId_chirho     -- ^ Just id if found, Nothing otherwise
camLookup_chirho cam term =
  let h = hashTerm_chirho term
      entry = cam !! h
  in if camValid_chirho entry && termEq_chirho (camTerm_chirho entry) term
     then Just (camId_chirho entry)
     else Nothing

-- | Update CAM with new entry
camInsert_chirho
  :: Vec 64 CamEntry_chirho  -- ^ Old CAM
  -> Term_chirho             -- ^ Term to insert
  -> TermId_chirho           -- ^ ID to associate
  -> Vec 64 CamEntry_chirho  -- ^ Updated CAM
camInsert_chirho cam term tid =
  let h = hashTerm_chirho term
      newEntry = CamEntry_chirho True term tid
  in replace h newEntry cam

-- | Empty CAM
emptyCam_chirho :: Vec 64 CamEntry_chirho
emptyCam_chirho = repeat $ CamEntry_chirho False nilTerm_chirho (TermId_chirho 0)

-- | Block RAM for term storage (1024 terms)
type TermStore_chirho = Vec 1024 Term_chirho

-- | Empty term store (all nil)
emptyTermStore_chirho :: TermStore_chirho
emptyTermStore_chirho = repeat nilTerm_chirho

-- | Block RAM for reference counts
type RefCounts_chirho = Vec 1024 (Unsigned 8)

-- | Initialize ref counts (nil has count 1)
initialRefCounts_chirho :: RefCounts_chirho
initialRefCounts_chirho = replace 0 1 $ repeat 0

-- | Complete hash cons unit state including memories
data HashConsUnit_chirho = HashConsUnit_chirho
  { hcuState_chirho     :: HcState_chirho
  , hcuTermStore_chirho :: TermStore_chirho
  , hcuCam_chirho       :: Vec 64 CamEntry_chirho
  , hcuRefCounts_chirho :: RefCounts_chirho
  } deriving (Show, Generic, NFDataX)

-- | Initial hash cons unit
initialHashConsUnit_chirho :: HashConsUnit_chirho
initialHashConsUnit_chirho = HashConsUnit_chirho
  { hcuState_chirho = initialHcState_chirho
  , hcuTermStore_chirho = emptyTermStore_chirho
  , hcuCam_chirho = emptyCam_chirho
  , hcuRefCounts_chirho = initialRefCounts_chirho
  }

-- | Main hash consing transition function
-- This is the core Mealy machine for the hash cons unit
hashConsStep_chirho
  :: HashConsUnit_chirho     -- ^ Current state
  -> HcCmd_chirho            -- ^ Input command
  -> (HashConsUnit_chirho, HcResp_chirho)  -- ^ (New state, Output)
hashConsStep_chirho unit cmd = case cmd of

  HcNop_chirho -> (unit, defaultResp_chirho)

  HcIntern_chirho term ->
    case camLookup_chirho (hcuCam_chirho unit) term of
      Just tid ->
        -- Term already exists, return existing ID
        (unit, HcResp_chirho tid term True True)
      Nothing ->
        -- Allocate new term
        let TermId_chirho ptr = freePtr_chirho (hcuState_chirho unit)
            newTid = TermId_chirho ptr
            newPtr = TermId_chirho (ptr + 1)
            -- Update memories
            newStore = replace ptr term (hcuTermStore_chirho unit)
            newCam = camInsert_chirho (hcuCam_chirho unit) term newTid
            newRefs = replace ptr 1 (hcuRefCounts_chirho unit)
            -- Update state
            newState = (hcuState_chirho unit) { freePtr_chirho = newPtr }
            newUnit = unit
              { hcuState_chirho = newState
              , hcuTermStore_chirho = newStore
              , hcuCam_chirho = newCam
              , hcuRefCounts_chirho = newRefs
              }
        in (newUnit, HcResp_chirho newTid term False True)

  HcDeref_chirho (TermId_chirho tid) ->
    let term = hcuTermStore_chirho unit !! tid
    in (unit, HcResp_chirho (TermId_chirho tid) term True True)

  HcIncRef_chirho (TermId_chirho tid) ->
    let oldCount = hcuRefCounts_chirho unit !! tid
        newRefs = replace tid (oldCount + 1) (hcuRefCounts_chirho unit)
        newUnit = unit { hcuRefCounts_chirho = newRefs }
    in (newUnit, defaultResp_chirho { respDone_chirho = True })

  HcDecRef_chirho (TermId_chirho tid) ->
    let oldCount = hcuRefCounts_chirho unit !! tid
        newRefs = replace tid (oldCount - 1) (hcuRefCounts_chirho unit)
        newUnit = unit { hcuRefCounts_chirho = newRefs }
    in (newUnit, defaultResp_chirho { respDone_chirho = True })

  where
    defaultResp_chirho = HcResp_chirho (TermId_chirho 0) nilTerm_chirho False False

-- | Convenience: create a cons cell
mkCons_chirho :: TermId_chirho -> TermId_chirho -> Term_chirho
mkCons_chirho l r = Term_chirho Cons_chirho l r

-- | Convenience: create a symbol
mkSymbol_chirho :: Unsigned 16 -> Term_chirho
mkSymbol_chirho sid = Term_chirho Symbol_chirho (TermId_chirho sid) (TermId_chirho 0)

-- | Convenience: create a variable
mkVar_chirho :: Unsigned 16 -> Term_chirho
mkVar_chirho vid = Term_chirho Var_chirho (TermId_chirho vid) (TermId_chirho 0)

-- | Top-level entity: hash cons unit as Mealy machine
-- Can be synthesized to hardware
{-# ANN hashConsTop_chirho
  (Synthesize
    { t_name   = "hashcons_chirho"
    , t_inputs = [PortName "clk", PortName "rst", PortName "cmd_chirho"]
    , t_output = PortName "resp_chirho"
    }) #-}
hashConsTop_chirho
  :: Clock System
  -> Reset System
  -> Signal System HcCmd_chirho
  -> Signal System HcResp_chirho
hashConsTop_chirho = exposeClockResetEnable $ mealy hashConsStep_chirho initialHashConsUnit_chirho

-- | Test bench: intern some terms and verify structural sharing
testHashCons_chirho :: [HcResp_chirho]
testHashCons_chirho =
  let cmds =
        [ HcIntern_chirho (mkSymbol_chirho 1)        -- a
        , HcIntern_chirho (mkSymbol_chirho 2)        -- b
        , HcIntern_chirho (mkCons_chirho (TermId_chirho 1) (TermId_chirho 0))  -- [a]
        , HcIntern_chirho (mkCons_chirho (TermId_chirho 1) (TermId_chirho 0))  -- [a] again (should find)
        ]
      go unit [] = []
      go unit (c:cs) =
        let (unit', resp) = hashConsStep_chirho unit c
        in resp : go unit' cs
  in go initialHashConsUnit_chirho cmds

-- Soli Deo Gloria ☧
