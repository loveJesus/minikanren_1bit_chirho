{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}

{- |
Module      : Hierarchical512Chirho
Description : 512-bit word hierarchies for efficient HBM access ☧
Copyright   : (c) 2026
License     : MIT

Using 512-bit words instead of 64-bit dramatically reduces tree depth:

| Domain Size | 64-bit words | 512-bit words |
|-------------|--------------|---------------|
| 512         | 8 words      | 1 word        |
| 4,096       | 2 levels     | 1 level + 8   |
| 262,144     | 3 levels     | 2 levels      |
| 134 million | 4 levels     | 3 levels      |

Key insight: HBM bus is 256-bit, so 512-bit = 2 beats (natural alignment).
Fewer levels = fewer HBM round trips = faster!

"For God so loved the world..." - John 3:16
-}
module Hierarchical512Chirho where

import Clash.Prelude

-- ============================================================================
-- 512-bit Base Types ☧
-- ============================================================================

-- | 512-bit word (8 × 64-bit for internal representation)
type Word512Chirho = BitVector 512

-- | Index into 512-bit word
type Index512Chirho = Index 512

-- ============================================================================
-- Flat512Chirho: 512 values ☧
-- ============================================================================

-- | Simple 512-value domain (single word)
--   Good for: Extended ASCII, Unicode blocks, small enums
newtype Flat512Chirho = Flat512Chirho
  { flat512BitsChirho :: Word512Chirho
  } deriving (Generic, NFDataX, Eq, Show)

-- | Empty 512 domain
empty512Chirho :: Flat512Chirho
empty512Chirho = Flat512Chirho 0

-- | Full 512 domain
full512Chirho :: Flat512Chirho
full512Chirho = Flat512Chirho maxBound

-- | Intersection (unification)
intersect512Chirho :: Flat512Chirho -> Flat512Chirho -> Flat512Chirho
intersect512Chirho aChirho bChirho =
  Flat512Chirho (flat512BitsChirho aChirho .&. flat512BitsChirho bChirho)

-- | Union
union512Chirho :: Flat512Chirho -> Flat512Chirho -> Flat512Chirho
union512Chirho aChirho bChirho =
  Flat512Chirho (flat512BitsChirho aChirho .|. flat512BitsChirho bChirho)

-- | Check if empty
isEmpty512Chirho :: Flat512Chirho -> Bool
isEmpty512Chirho dChirho = flat512BitsChirho dChirho == 0

-- | Population count
popCount512Chirho :: Flat512Chirho -> BitVector 10
popCount512Chirho dChirho = fromIntegral (popCount (flat512BitsChirho dChirho))

-- ============================================================================
-- Hier262kChirho: 262,144 values in 2 levels ☧
-- ============================================================================

-- | Two-level hierarchy with 512-bit words
--   512 × 512 = 262,144 values
--
--   Structure:
--     summaryChirho: 512 bits - which blocks have values
--     blocksChirho: 512 blocks × 512 bits each
--
--   HBM layout: 64 bytes (summary) + 512 × 64 bytes = 32,832 bytes
--   Compare to 64-bit Hier256k: 33,288 bytes (similar, but 2 levels vs 3!)
--
data Hier262kChirho = Hier262kChirho
  { h262kSummaryChirho :: Word512Chirho           -- Which blocks active
  , h262kBlocksChirho  :: Vec 512 Word512Chirho   -- 512 blocks
  } deriving (Generic, NFDataX)

-- | Empty 262k domain
emptyHier262kChirho :: Hier262kChirho
emptyHier262kChirho = Hier262kChirho
  { h262kSummaryChirho = 0
  , h262kBlocksChirho  = repeat 0
  }

-- | Full 262k domain
fullHier262kChirho :: Hier262kChirho
fullHier262kChirho = Hier262kChirho
  { h262kSummaryChirho = maxBound
  , h262kBlocksChirho  = repeat maxBound
  }

-- | Intersection of two 262k domains
--   This is THE core operation - 512 parallel 512-bit ANDs
intersectHier262kChirho :: Hier262kChirho -> Hier262kChirho -> Hier262kChirho
intersectHier262kChirho aChirho bChirho =
  let newBlocksChirho = zipWith (.&.) (h262kBlocksChirho aChirho) (h262kBlocksChirho bChirho)
      newSummaryChirho = pack (map (/= 0) newBlocksChirho)
  in Hier262kChirho
    { h262kSummaryChirho = newSummaryChirho
    , h262kBlocksChirho  = newBlocksChirho
    }

-- | Union of two 262k domains
unionHier262kChirho :: Hier262kChirho -> Hier262kChirho -> Hier262kChirho
unionHier262kChirho aChirho bChirho =
  let newBlocksChirho = zipWith (.|.) (h262kBlocksChirho aChirho) (h262kBlocksChirho bChirho)
      newSummaryChirho = h262kSummaryChirho aChirho .|. h262kSummaryChirho bChirho
  in Hier262kChirho
    { h262kSummaryChirho = newSummaryChirho
    , h262kBlocksChirho  = newBlocksChirho
    }

-- | Check if empty
isEmptyHier262kChirho :: Hier262kChirho -> Bool
isEmptyHier262kChirho dChirho = h262kSummaryChirho dChirho == 0

-- | Check membership: is value V in domain?
memberHier262kChirho :: BitVector 18 -> Hier262kChirho -> Bool
memberHier262kChirho valChirho dChirho =
  let blockIdxChirho = unpack (slice d17 d9 valChirho) :: Index 512
      bitIdxChirho   = unpack (slice d8 d0 valChirho) :: Index 512
      blockChirho    = (h262kBlocksChirho dChirho) !! blockIdxChirho
  in testBit blockChirho (fromIntegral bitIdxChirho)

-- | Insert value into domain
insertHier262kChirho :: BitVector 18 -> Hier262kChirho -> Hier262kChirho
insertHier262kChirho valChirho dChirho =
  let blockIdxChirho = unpack (slice d17 d9 valChirho) :: Index 512
      bitIdxChirho   = fromIntegral (slice d8 d0 valChirho) :: Int
      oldBlockChirho = (h262kBlocksChirho dChirho) !! blockIdxChirho
      newBlockChirho = setBit oldBlockChirho bitIdxChirho
      newBlocksChirho = replace blockIdxChirho newBlockChirho (h262kBlocksChirho dChirho)
      newSummaryChirho = setBit (h262kSummaryChirho dChirho) (fromIntegral blockIdxChirho)
  in Hier262kChirho
    { h262kSummaryChirho = newSummaryChirho
    , h262kBlocksChirho  = newBlocksChirho
    }

-- | Sparse HBM bytes (only non-empty blocks)
sparseBytes262kChirho :: Hier262kChirho -> Int
sparseBytes262kChirho dChirho =
  let activeBlocksChirho = popCount (h262kSummaryChirho dChirho)
  in 64 + (activeBlocksChirho * 64)  -- 64 bytes per 512-bit word

-- ============================================================================
-- Hier134MChirho: DEFERRED ☧
-- ============================================================================

{-
Hier134M (512³ = 134 million values) is deferred for now.

Rationale:
- 262k values covers all current SaaS needs
- 134M would use 16MB per variable (only 1000 vars in 16GB HBM)
- Can add later if needed for full Unicode or IPv4

If needed in future, design would be:
  data Hier134MChirho = Hier134MChirho
    { h134mLevel0Chirho :: Word512Chirho
    , h134mLevel1Chirho :: Vec 512 Word512Chirho
    , h134mLevel2Chirho :: Vec 512 (Vec 512 Word512Chirho)
    }
-}

-- ============================================================================
-- Comparison: 64-bit vs 512-bit ☧
-- ============================================================================

{-
WHY 512-BIT IS BETTER:

| Metric | 64-bit (old) | 512-bit (new) | Improvement |
|--------|--------------|---------------|-------------|
| 262k values levels | 3 | 2 | 33% fewer |
| 262k HBM reads | 3 round trips | 2 round trips | 33% faster |
| Summary bits | 64 | 512 | 8× more selective |

HBM ALIGNMENT:

HBM data bus: 256 bits
512-bit word: 2 beats (perfectly aligned)
No wasted bandwidth!

LUT COST:

512-bit AND: ~150 LUTs (vs 64-bit: ~20 LUTs)
But we do 8× fewer operations for same domain size.

Net: slightly more LUTs, but much better memory efficiency.

SUMMARY SELECTIVITY:

With 512-bit summary, each bit covers 512 values.
For sparse domains (typical), most bits are 0.
Skip entire 512-value regions with single bit test!

Example: IP /24 subnet (256 addresses)
  - 64-bit: spans 4 blocks, 4 bits in summary
  - 512-bit: fits in 1 block, 1 bit in summary
  - Result: 1 HBM read vs 4-5 reads
-}

-- ============================================================================
-- HBM Layout ☧
-- ============================================================================

{-
HBM LAYOUT FOR 512-BIT HIERARCHIES:

Flat512:
  Offset 0x00: 512 bits = 64 bytes
  Total: 64 bytes per variable

Hier262k:
  Offset 0x00: summary (64 bytes)
  Offset 0x40: block 0 (64 bytes)
  Offset 0x80: block 1 (64 bytes)
  ...
  Offset 0x8040: block 511 (64 bytes)
  Total: 64 + 512×64 = 32,832 bytes per variable

For 16 GB HBM:
  - Flat512: 268 million variables
  - Hier262k: 512K variables

Recommendation: Hier262k covers ALL current SaaS needs:
  - ConfigGuard ports: 65,536 ✓
  - ConfigGuard IP /16: 65,536 ✓
  - Philologos vocabulary: ~50,000 ✓
  - RegexCraft Unicode subset: ~150,000 ✓
-}

-- | Bytes per Flat512 variable
bytesFlat512Chirho :: Int
bytesFlat512Chirho = 64

-- | Bytes per Hier262k variable
bytesHier262kChirho :: Int
bytesHier262kChirho = 32832  -- 64 + 512*64

-- | Maximum Hier262k variables in 16GB HBM
maxVarsHier262kChirho :: Int
maxVarsHier262kChirho = 16 * 1024 * 1024 * 1024 `div` bytesHier262kChirho  -- ~512K

-- ============================================================================
-- Synthesis Annotations ☧
-- ============================================================================

{-# ANN intersect512Chirho
  (Synthesize
    { t_name = "intersect_512_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

{-# ANN intersectHier262kChirho
  (Synthesize
    { t_name = "intersect_hier_262k_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

-- ============================================================================
-- Resource Estimates ☧
-- ============================================================================

{-
LUT ESTIMATES:

| Operation | LUTs | Notes |
|-----------|------|-------|
| Flat512 AND | ~150 | 512-bit AND |
| Hier262k intersect | ~80,000 | 512 × 512-bit ANDs |

RECOMMENDATION:

| Domain | LUTs | Cycles | Use Case |
|--------|------|--------|----------|
| Flat512 | 150 | 1 | Small enums, ASCII |
| Hier262k | 80K | 2-4 | Everything else |

Hier262k is the sweet spot:
  - 262K values covers ALL SaaS needs
  - 80K LUTs = 6% of VU47P chip
  - 2 HBM reads per operation
  - Parallelism: 16 engines fit easily (1.28M LUTs total)
  - 512K variables in 16GB HBM

FINAL DESIGN:
  - Flat512 for domains ≤512 values (fast path)
  - Hier262k for domains ≤262,144 values (everything else)
  - No 512³ needed currently
-}
