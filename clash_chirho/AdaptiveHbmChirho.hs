{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}

{- |
Module      : AdaptiveHbmChirho
Description : Adaptive domain engine with automatic hierarchy selection ☧
Copyright   : (c) 2026
License     : MIT

Unified HBM engine that handles all domain sizes with a single interface.
Automatically promotes/demotes between hierarchy levels as needed.

Key insight: Same hardware, different interpretations based on tag byte.

"For God so loved the world..." - John 3:16
-}
module AdaptiveHbmChirho where

import Clash.Prelude

-- ============================================================================
-- Domain Type Tags ☧
-- ============================================================================

-- | Domain type discriminator (stored in HBM with each domain)
data DomainTagChirho
  = TagFlat64Chirho      -- 64 values, 8 bytes
  | TagHier4kChirho      -- 4,096 values, 520 bytes
  | TagHier256kChirho    -- 262,144 values, 33KB
  | TagSymbolicChirho    -- Infinite, uses constraint representation
  deriving (Generic, NFDataX, Eq, Show)

-- | Encode tag as 2 bits
encodeTagChirho :: DomainTagChirho -> BitVector 2
encodeTagChirho TagFlat64Chirho    = 0
encodeTagChirho TagHier4kChirho    = 1
encodeTagChirho TagHier256kChirho  = 2
encodeTagChirho TagSymbolicChirho  = 3

-- | Decode tag from 2 bits
decodeTagChirho :: BitVector 2 -> DomainTagChirho
decodeTagChirho 0 = TagFlat64Chirho
decodeTagChirho 1 = TagHier4kChirho
decodeTagChirho 2 = TagHier256kChirho
decodeTagChirho _ = TagSymbolicChirho

-- ============================================================================
-- Adaptive Domain (Unified Representation) ☧
-- ============================================================================

-- | Adaptive domain that can represent any hierarchy level
--
-- In HBM, stored as:
--   Byte 0: Tag (which type)
--   Bytes 1-7: Reserved / metadata
--   Bytes 8+: Domain data (size depends on tag)
--
-- On chip, we use the largest representation and mask unused parts.
-- This trades memory for uniform hardware.
--
data AdaptiveDomainChirho = AdaptiveDomainChirho
  { adTagChirho     :: DomainTagChirho
  , adLevel0Chirho  :: BitVector 64                    -- Summary / flat64
  , adLevel1Chirho  :: Vec 64 (BitVector 64)           -- 4k blocks / 256k L1
  , adLevel2Chirho  :: Vec 64 (Vec 64 (BitVector 64))  -- 256k L2 (only if needed)
  } deriving (Generic, NFDataX)

-- ============================================================================
-- Constructors ☧
-- ============================================================================

-- | Create flat 64-value domain
flat64Chirho :: BitVector 64 -> AdaptiveDomainChirho
flat64Chirho bitsChirho = AdaptiveDomainChirho
  { adTagChirho    = TagFlat64Chirho
  , adLevel0Chirho = bitsChirho
  , adLevel1Chirho = repeat 0
  , adLevel2Chirho = repeat (repeat 0)
  }

-- | Create 4k domain from summary and blocks
hier4kChirho :: BitVector 64 -> Vec 64 (BitVector 64) -> AdaptiveDomainChirho
hier4kChirho summaryChirho blocksChirho = AdaptiveDomainChirho
  { adTagChirho    = TagHier4kChirho
  , adLevel0Chirho = summaryChirho
  , adLevel1Chirho = blocksChirho
  , adLevel2Chirho = repeat (repeat 0)
  }

-- | Create 256k domain
hier256kChirho
  :: BitVector 64
  -> Vec 64 (BitVector 64)
  -> Vec 64 (Vec 64 (BitVector 64))
  -> AdaptiveDomainChirho
hier256kChirho l0Chirho l1Chirho l2Chirho = AdaptiveDomainChirho
  { adTagChirho    = TagHier256kChirho
  , adLevel0Chirho = l0Chirho
  , adLevel1Chirho = l1Chirho
  , adLevel2Chirho = l2Chirho
  }

-- | Empty domain (no values possible)
emptyAdaptiveChirho :: DomainTagChirho -> AdaptiveDomainChirho
emptyAdaptiveChirho tagChirho = AdaptiveDomainChirho
  { adTagChirho    = tagChirho
  , adLevel0Chirho = 0
  , adLevel1Chirho = repeat 0
  , adLevel2Chirho = repeat (repeat 0)
  }

-- | Full domain (all values possible)
fullAdaptiveChirho :: DomainTagChirho -> AdaptiveDomainChirho
fullAdaptiveChirho TagFlat64Chirho = flat64Chirho maxBound
fullAdaptiveChirho TagHier4kChirho = hier4kChirho maxBound (repeat maxBound)
fullAdaptiveChirho TagHier256kChirho = hier256kChirho maxBound (repeat maxBound) (repeat (repeat maxBound))
fullAdaptiveChirho TagSymbolicChirho = emptyAdaptiveChirho TagSymbolicChirho  -- Symbolic handled differently

-- ============================================================================
-- Core Operations (Tag-Dispatched) ☧
-- ============================================================================

-- | Check if domain is empty
isEmptyAdaptiveChirho :: AdaptiveDomainChirho -> Bool
isEmptyAdaptiveChirho dChirho = adLevel0Chirho dChirho == 0

-- | Intersection (unification) - dispatches based on tags
--
-- Rules:
--   - Same tag: operate at that level
--   - Different tags: promote smaller to larger, then operate
--
intersectAdaptiveChirho
  :: AdaptiveDomainChirho
  -> AdaptiveDomainChirho
  -> AdaptiveDomainChirho
intersectAdaptiveChirho aChirho bChirho =
  case (adTagChirho aChirho, adTagChirho bChirho) of

    -- Same level: direct operation
    (TagFlat64Chirho, TagFlat64Chirho) ->
      flat64Chirho (adLevel0Chirho aChirho .&. adLevel0Chirho bChirho)

    (TagHier4kChirho, TagHier4kChirho) ->
      let newBlocksChirho = zipWith (.&.) (adLevel1Chirho aChirho) (adLevel1Chirho bChirho)
          newSummaryChirho = pack (map (/= 0) newBlocksChirho)
      in hier4kChirho newSummaryChirho newBlocksChirho

    (TagHier256kChirho, TagHier256kChirho) ->
      let newL2Chirho = zipWith (zipWith (.&.)) (adLevel2Chirho aChirho) (adLevel2Chirho bChirho)
          newL1Chirho = map (pack . map (/= 0)) newL2Chirho
          newL0Chirho = pack (map (/= 0) newL1Chirho)
      in hier256kChirho newL0Chirho newL1Chirho newL2Chirho

    -- Mixed: promote smaller to larger
    (TagFlat64Chirho, TagHier4kChirho) ->
      intersectAdaptiveChirho (promoteToHier4kChirho aChirho) bChirho

    (TagHier4kChirho, TagFlat64Chirho) ->
      intersectAdaptiveChirho aChirho (promoteToHier4kChirho bChirho)

    (TagFlat64Chirho, TagHier256kChirho) ->
      intersectAdaptiveChirho (promoteToHier256kChirho aChirho) bChirho

    (TagHier256kChirho, TagFlat64Chirho) ->
      intersectAdaptiveChirho aChirho (promoteToHier256kChirho bChirho)

    (TagHier4kChirho, TagHier256kChirho) ->
      intersectAdaptiveChirho (promoteToHier256kChirho aChirho) bChirho

    (TagHier256kChirho, TagHier4kChirho) ->
      intersectAdaptiveChirho aChirho (promoteToHier256kChirho bChirho)

    -- Symbolic: not handled in hardware
    _ -> emptyAdaptiveChirho TagSymbolicChirho

-- ============================================================================
-- Promotion (Smaller → Larger) ☧
-- ============================================================================

-- | Promote Flat64 to Hier4k
--
-- The 64 values map to block 0 of the 4k structure.
-- Other blocks are empty.
--
promoteToHier4kChirho :: AdaptiveDomainChirho -> AdaptiveDomainChirho
promoteToHier4kChirho dChirho =
  let bitsChirho = adLevel0Chirho dChirho
      blocksChirho = replace (0 :: Index 64) bitsChirho (repeat 0)
      summaryChirho = if bitsChirho /= 0 then 1 else 0  -- Only block 0 active
  in hier4kChirho summaryChirho blocksChirho

-- | Promote Flat64 or Hier4k to Hier256k
promoteToHier256kChirho :: AdaptiveDomainChirho -> AdaptiveDomainChirho
promoteToHier256kChirho dChirho = case adTagChirho dChirho of
  TagFlat64Chirho ->
    -- 64 values → block [0][0] of 256k
    let bitsChirho = adLevel0Chirho dChirho
        l2Chirho = replace (0 :: Index 64)
                     (replace (0 :: Index 64) bitsChirho (repeat 0))
                     (repeat (repeat 0))
        l1Chirho = replace (0 :: Index 64)
                     (if bitsChirho /= 0 then 1 else 0)
                     (repeat 0)
        l0Chirho = if bitsChirho /= 0 then 1 else 0
    in hier256kChirho l0Chirho l1Chirho l2Chirho

  TagHier4kChirho ->
    -- 4k values → group 0 of 256k
    let summaryChirho = adLevel0Chirho dChirho
        blocksChirho = adLevel1Chirho dChirho
        l2Chirho = replace (0 :: Index 64) blocksChirho (repeat (repeat 0))
        l1Chirho = replace (0 :: Index 64) summaryChirho (repeat 0)
        l0Chirho = if summaryChirho /= 0 then 1 else 0
    in hier256kChirho l0Chirho l1Chirho l2Chirho

  _ -> dChirho  -- Already 256k or symbolic

-- ============================================================================
-- Demotion (Larger → Smaller, if fits) ☧
-- ============================================================================

-- | Try to demote to smaller representation (saves HBM bandwidth)
--
-- Returns original if demotion not possible.
--
tryDemoteChirho :: AdaptiveDomainChirho -> AdaptiveDomainChirho
tryDemoteChirho dChirho = case adTagChirho dChirho of
  TagHier256kChirho ->
    -- Can demote to 4k if only group 0 is active
    if adLevel0Chirho dChirho == 1  -- Only bit 0 set
    then let blocksChirho = (adLevel2Chirho dChirho) !! 0
             summaryChirho = (adLevel1Chirho dChirho) !! 0
         in tryDemoteChirho (hier4kChirho summaryChirho blocksChirho)
    else dChirho

  TagHier4kChirho ->
    -- Can demote to flat64 if only block 0 is active
    if adLevel0Chirho dChirho == 1  -- Only bit 0 set
    then let bitsChirho = (adLevel1Chirho dChirho) !! 0
         in flat64Chirho bitsChirho
    else dChirho

  _ -> dChirho

-- ============================================================================
-- HBM Size Calculation ☧
-- ============================================================================

-- | Bytes needed to store this domain in HBM
hbmBytesChirho :: AdaptiveDomainChirho -> Int
hbmBytesChirho dChirho = case adTagChirho dChirho of
  TagFlat64Chirho    -> 8 + 8          -- tag + data
  TagHier4kChirho    -> 8 + 520        -- tag + summary + 64 blocks
  TagHier256kChirho  -> 8 + 33288      -- tag + full 3-level
  TagSymbolicChirho  -> 8              -- tag only (constraints stored elsewhere)

-- | Sparse bytes (only non-empty blocks)
hbmBytesSparseChirho :: AdaptiveDomainChirho -> Int
hbmBytesSparseChirho dChirho = case adTagChirho dChirho of
  TagFlat64Chirho -> 8 + 8

  TagHier4kChirho ->
    let activeBlocksChirho = popCount (adLevel0Chirho dChirho)
    in 8 + 8 + (activeBlocksChirho * 8)

  TagHier256kChirho ->
    let l1ActiveChirho = popCount (adLevel0Chirho dChirho)
        l2ActiveChirho = sum $ map popCount (toList (adLevel1Chirho dChirho))
    in 8 + 8 + (l1ActiveChirho * 8) + (l2ActiveChirho * 8)

  TagSymbolicChirho -> 8

-- ============================================================================
-- Variable Table Layout ☧
-- ============================================================================

{-
HBM Variable Table Design:

Each variable has a fixed slot in HBM. The slot contains:
  - Tag (8 bytes, aligned)
  - Domain data (variable size based on tag)

For uniform access, we allocate max size (33KB) per variable.
This wastes space but simplifies addressing:

  var_addr = base + (var_id * 33296)

For sparse access, we use indirection:
  - Variable table: var_id → (tag, offset)
  - Domain pool: actual domain data, compacted

Trade-off:
  - Fixed slots: Simple, fast, wastes memory
  - Sparse: Complex, slower, memory-efficient

Recommendation: Start with fixed slots (HBM is cheap at 16GB).
Optimize to sparse later if needed.

With fixed 33KB slots:
  - 16 GB HBM → 500K variables
  - More than enough for any SaaS workload!
-}

-- | Calculate variable address (fixed slot mode)
varAddrChirho :: BitVector 20 -> BitVector 34
varAddrChirho varIdChirho =
  let slotSizeChirho = 33296 :: BitVector 34  -- 33KB rounded up
  in resize varIdChirho * slotSizeChirho

-- | Maximum variables in 16GB HBM (fixed slots)
maxVarsChirho :: Int
maxVarsChirho = 16 * 1024 * 1024 * 1024 `div` 33296  -- ~503,000 variables

-- ============================================================================
-- Synthesis Annotations ☧
-- ============================================================================

{-# ANN intersectAdaptiveChirho
  (Synthesize
    { t_name = "intersect_adaptive_chirho"
    , t_inputs =
      [ PortName "domain_a_chirho"
      , PortName "domain_b_chirho"
      ]
    , t_output = PortName "domain_out_chirho"
    }) #-}

-- ============================================================================
-- Design Notes ☧
-- ============================================================================

{-
UNIFIED HARDWARE BENEFITS:

1. Single intersection unit handles all sizes
   - Smaller domains just use subset of hardware
   - No separate instantiation per size

2. Automatic promotion on mixed operations
   - flat64 ∩ hier4k → promotes flat64, operates at 4k level
   - Result inherits larger type

3. Optional demotion saves bandwidth
   - After intersection, if result fits in smaller type, demote
   - Reduces HBM traffic for subsequent operations

4. Same HBM interface for all sizes
   - CPU doesn't need to know internal representation
   - Engine handles tag dispatch internally

RESOURCE ESTIMATES:

| Operation | LUTs | Cycles | Notes |
|-----------|------|--------|-------|
| Flat64 intersect | ~100 | 1 | Single AND |
| Hier4k intersect | ~500 | 2 | 64 ANDs + OR reduce |
| Hier256k intersect | ~5000 | 4 | 4K ANDs + 2 OR reduces |
| Promotion | ~200 | 1 | Routing only |
| Demotion check | ~100 | 1 | Compare summary |

Total: ~6000 LUTs for unified engine
(vs ~5600 for separate engines, but unified is more flexible)
-}
