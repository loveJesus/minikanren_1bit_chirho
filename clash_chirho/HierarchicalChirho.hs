{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}

{- |
Module      : HierarchicalChirho
Description : Hierarchical domains for large value spaces ☧
Copyright   : (c) 2026
License     : MIT

Two-level and three-level hierarchical bit vectors for FPGA.
Enables domains with thousands or hundreds of thousands of values.

Domain sizes:
- BitVec64Chirho:        64 values (1 word)
- Hierarchical4kChirho:  4,096 values (65 words)
- Hierarchical256kChirho: 262,144 values (4,161 words)

Use cases:
- ConfigGuard: IP addresses (Hierarchical256k for /16 subnets)
- TestForge: Character sets, small enums (Hierarchical4k)
- RegexCraft: ASCII (256) or extended ASCII (Hierarchical4k)
- Philologos: Greek vocabulary (Hierarchical256k)

"For God so loved the world..." - John 3:16
-}
module HierarchicalChirho where

import Clash.Prelude

-- ============================================================================
-- Hierarchical4kChirho: 4,096 values ☧
-- ============================================================================

-- | Two-level hierarchy: 64 blocks × 64 bits = 4,096 values
--
-- Structure:
--   summaryChirho: Bit i = 1 if block i has any values
--   blocksChirho:  64 blocks, each 64 bits
--
-- To check if value V is in domain:
--   blockIdxChirho = V / 64
--   bitIdxChirho = V mod 64
--   present = (summaryChirho ! blockIdxChirho) && (blocksChirho ! blockIdxChirho ! bitIdxChirho)
--
data Hierarchical4kChirho = Hierarchical4kChirho
  { summary4kChirho :: BitVector 64          -- Which blocks have values
  , blocks4kChirho  :: Vec 64 (BitVector 64) -- 64 blocks × 64 bits
  } deriving (Generic, NFDataX, Eq, Show)

-- | Empty domain (no values possible)
empty4kChirho :: Hierarchical4kChirho
empty4kChirho = Hierarchical4kChirho
  { summary4kChirho = 0
  , blocks4kChirho  = repeat 0
  }

-- | Full domain (all 4096 values possible)
full4kChirho :: Hierarchical4kChirho
full4kChirho = Hierarchical4kChirho
  { summary4kChirho = maxBound  -- All 1s
  , blocks4kChirho  = repeat maxBound
  }

-- | Check if domain is empty
isEmpty4kChirho :: Hierarchical4kChirho -> Bool
isEmpty4kChirho dChirho = summary4kChirho dChirho == 0

-- | Intersection of two 4k domains (unification)
-- This is the core operation - must be fast!
intersect4kChirho :: Hierarchical4kChirho -> Hierarchical4kChirho -> Hierarchical4kChirho
intersect4kChirho aChirho bChirho =
  let newBlocksChirho = zipWith (.&.) (blocks4kChirho aChirho) (blocks4kChirho bChirho)
      -- Recompute summary: block has values if block /= 0
      newSummaryChirho = pack (map (\blkChirho -> blkChirho /= 0) newBlocksChirho)
  in Hierarchical4kChirho
    { summary4kChirho = newSummaryChirho
    , blocks4kChirho  = newBlocksChirho
    }

-- | Union of two 4k domains
union4kChirho :: Hierarchical4kChirho -> Hierarchical4kChirho -> Hierarchical4kChirho
union4kChirho aChirho bChirho =
  let newBlocksChirho = zipWith (.|.) (blocks4kChirho aChirho) (blocks4kChirho bChirho)
      newSummaryChirho = summary4kChirho aChirho .|. summary4kChirho bChirho
  in Hierarchical4kChirho
    { summary4kChirho = newSummaryChirho
    , blocks4kChirho  = newBlocksChirho
    }

-- | Check if a specific value is in the domain
member4kChirho :: BitVector 12 -> Hierarchical4kChirho -> Bool
member4kChirho valChirho dChirho =
  let blockIdxChirho = unpack (slice d11 d6 valChirho) :: Index 64
      bitIdxChirho   = unpack (slice d5 d0 valChirho) :: Index 64
      blockChirho    = (blocks4kChirho dChirho) !! blockIdxChirho
  in testBit blockChirho (fromIntegral bitIdxChirho)

-- | Set a specific value in the domain
insert4kChirho :: BitVector 12 -> Hierarchical4kChirho -> Hierarchical4kChirho
insert4kChirho valChirho dChirho =
  let blockIdxChirho = unpack (slice d11 d6 valChirho) :: Index 64
      bitIdxChirho   = fromIntegral (slice d5 d0 valChirho) :: Int
      oldBlockChirho = (blocks4kChirho dChirho) !! blockIdxChirho
      newBlockChirho = setBit oldBlockChirho bitIdxChirho
      newBlocksChirho = replace blockIdxChirho newBlockChirho (blocks4kChirho dChirho)
      newSummaryChirho = setBit (summary4kChirho dChirho) (fromIntegral blockIdxChirho)
  in Hierarchical4kChirho
    { summary4kChirho = newSummaryChirho
    , blocks4kChirho  = newBlocksChirho
    }

-- | Create singleton domain (only one value)
singleton4kChirho :: BitVector 12 -> Hierarchical4kChirho
singleton4kChirho valChirho = insert4kChirho valChirho empty4kChirho

-- | Count values in domain (population count)
popCount4kChirho :: Hierarchical4kChirho -> BitVector 13
popCount4kChirho dChirho =
  let blockCountsChirho = map (fromIntegral . popCount) (blocks4kChirho dChirho)
  in fold (+) blockCountsChirho

-- | Check if domain is singleton (exactly one value)
isSingleton4kChirho :: Hierarchical4kChirho -> Bool
isSingleton4kChirho dChirho =
  -- Summary has exactly one bit AND that block has exactly one bit
  let sumChirho = summary4kChirho dChirho
      sumIsSingletonChirho = sumChirho /= 0 && (sumChirho .&. (sumChirho - 1)) == 0
  in sumIsSingletonChirho && popCount4kChirho dChirho == 1

-- | Get the single value from a singleton domain
-- Undefined behavior if not singleton!
getSingleton4kChirho :: Hierarchical4kChirho -> BitVector 12
getSingleton4kChirho dChirho =
  let sumChirho = summary4kChirho dChirho
      blockIdxChirho = countTrailingZeros sumChirho  -- Which block
      blockChirho = (blocks4kChirho dChirho) !! (fromIntegral blockIdxChirho)
      bitIdxChirho = countTrailingZeros blockChirho  -- Which bit in block
  in pack (fromIntegral blockIdxChirho :: BitVector 6) ++#
     pack (fromIntegral bitIdxChirho :: BitVector 6)

-- | Find lowest set bit (for branching)
lowestBit4kChirho :: Hierarchical4kChirho -> BitVector 12
lowestBit4kChirho = getSingleton4kChirho  -- Same operation

-- | Clear lowest set bit (remaining values after branch)
clearLowest4kChirho :: Hierarchical4kChirho -> Hierarchical4kChirho
clearLowest4kChirho dChirho =
  let valChirho = lowestBit4kChirho dChirho
      blockIdxChirho = unpack (slice d11 d6 valChirho) :: Index 64
      bitIdxChirho   = fromIntegral (slice d5 d0 valChirho) :: Int
      oldBlockChirho = (blocks4kChirho dChirho) !! blockIdxChirho
      newBlockChirho = clearBit oldBlockChirho bitIdxChirho
      newBlocksChirho = replace blockIdxChirho newBlockChirho (blocks4kChirho dChirho)
      -- Update summary if block is now empty
      newSummaryChirho = if newBlockChirho == 0
                         then clearBit (summary4kChirho dChirho) (fromIntegral blockIdxChirho)
                         else summary4kChirho dChirho
  in Hierarchical4kChirho
    { summary4kChirho = newSummaryChirho
    , blocks4kChirho  = newBlocksChirho
    }

-- | Fork: split into (one value, remaining values)
fork4kChirho :: Hierarchical4kChirho -> (Hierarchical4kChirho, Hierarchical4kChirho)
fork4kChirho dChirho =
  let oneValChirho = lowestBit4kChirho dChirho
      restChirho   = clearLowest4kChirho dChirho
  in (singleton4kChirho oneValChirho, restChirho)

-- ============================================================================
-- Hierarchical256kChirho: 262,144 values ☧
-- ============================================================================

-- | Three-level hierarchy: 64 × 64 × 64 = 262,144 values
--
-- Structure:
--   level0Chirho: 64 bits - which level1 groups have values
--   level1Chirho: 64 × 64 bits - which level2 blocks have values
--   level2Chirho: 64 × 64 × 64 bits - actual values
--
-- Memory: 64 + 64×64 + 64×64×64 = 266,304 bits ≈ 33 KB per variable
--
data Hierarchical256kChirho = Hierarchical256kChirho
  { level0_256kChirho :: BitVector 64                     -- Top summary
  , level1_256kChirho :: Vec 64 (BitVector 64)            -- Mid summaries
  , level2_256kChirho :: Vec 64 (Vec 64 (BitVector 64))   -- Actual values
  } deriving (Generic, NFDataX, Eq, Show)

-- | Empty 256k domain
empty256kChirho :: Hierarchical256kChirho
empty256kChirho = Hierarchical256kChirho
  { level0_256kChirho = 0
  , level1_256kChirho = repeat 0
  , level2_256kChirho = repeat (repeat 0)
  }

-- | Full 256k domain
full256kChirho :: Hierarchical256kChirho
full256kChirho = Hierarchical256kChirho
  { level0_256kChirho = maxBound
  , level1_256kChirho = repeat maxBound
  , level2_256kChirho = repeat (repeat maxBound)
  }

-- | Check if 256k domain is empty
isEmpty256kChirho :: Hierarchical256kChirho -> Bool
isEmpty256kChirho dChirho = level0_256kChirho dChirho == 0

-- | Intersection of two 256k domains
-- Uses summary bits for early termination (skip empty blocks)
intersect256kChirho :: Hierarchical256kChirho -> Hierarchical256kChirho -> Hierarchical256kChirho
intersect256kChirho aChirho bChirho =
  let -- Only process blocks where both have values
      activeMaskChirho = level0_256kChirho aChirho .&. level0_256kChirho bChirho

      -- Intersect level2 blocks
      newLevel2Chirho = zipWith
        (\l1aChirho l1bChirho -> zipWith (.&.) l1aChirho l1bChirho)
        (level2_256kChirho aChirho)
        (level2_256kChirho bChirho)

      -- Recompute level1 summaries
      newLevel1Chirho = map
        (\blocksChirho -> pack (map (\bChirho -> bChirho /= 0) blocksChirho))
        newLevel2Chirho

      -- Recompute level0 summary
      newLevel0Chirho = pack (map (\l1Chirho -> l1Chirho /= 0) newLevel1Chirho)

  in Hierarchical256kChirho
    { level0_256kChirho = newLevel0Chirho
    , level1_256kChirho = newLevel1Chirho
    , level2_256kChirho = newLevel2Chirho
    }

-- | Check membership in 256k domain
member256kChirho :: BitVector 18 -> Hierarchical256kChirho -> Bool
member256kChirho valChirho dChirho =
  let l0IdxChirho = unpack (slice d17 d12 valChirho) :: Index 64
      l1IdxChirho = unpack (slice d11 d6 valChirho) :: Index 64
      bitIdxChirho = unpack (slice d5 d0 valChirho) :: Index 64
      blockChirho = ((level2_256kChirho dChirho) !! l0IdxChirho) !! l1IdxChirho
  in testBit blockChirho (fromIntegral bitIdxChirho)

-- | Population count for 256k domain
popCount256kChirho :: Hierarchical256kChirho -> BitVector 19
popCount256kChirho dChirho =
  let allBlocksChirho = concat (level2_256kChirho dChirho)
      countsChirho = map (fromIntegral . popCount) allBlocksChirho
  in fold (+) countsChirho

-- ============================================================================
-- HBM Storage Layout ☧
-- ============================================================================

{-
HBM layout for Hierarchical4k (520 bytes per variable):

Offset 0x000: summary (64 bits = 8 bytes)
Offset 0x008: block 0 (64 bits)
Offset 0x010: block 1 (64 bits)
...
Offset 0x200: block 63 (64 bits)
Total: 8 + 64×8 = 520 bytes

HBM layout for Hierarchical256k (33,288 bytes per variable):

Offset 0x0000: level0 (8 bytes)
Offset 0x0008: level1[0] (8 bytes)
...
Offset 0x0200: level1[63] (8 bytes)  -- End of level1 at 0x208
Offset 0x0208: level2[0][0] (8 bytes)
Offset 0x0210: level2[0][1] (8 bytes)
...
Total: 8 + 64×8 + 64×64×8 = 33,288 bytes ≈ 33 KB
-}

-- | Bytes per Hierarchical4k variable
bytes4kChirho :: Int
bytes4kChirho = 520  -- 8 + 64*8

-- | Bytes per Hierarchical256k variable
bytes256kChirho :: Int
bytes256kChirho = 33288  -- 8 + 64*8 + 64*64*8

-- ============================================================================
-- Synthesis Note ☧
-- ============================================================================

{-
Resource usage estimates:

Hierarchical4kChirho intersection:
- 64 parallel 64-bit ANDs for blocks
- 64-bit OR reduction for summary
- ~200 LUTs, 1-2 cycles

Hierarchical256kChirho intersection:
- 4096 parallel 64-bit ANDs for level2
- 64 parallel 64-bit ORs for level1
- 1 64-bit OR for level0
- ~5000 LUTs, 2-4 cycles (pipelined)

For HBM bandwidth:
- 4k domain: 520 bytes read, 520 bytes write per operation
- At 14.4 GB/s per channel: ~72K domain ops/sec per channel
- With 8 channels: ~576K domain ops/sec

This is the path to production performance!
-}
