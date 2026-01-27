{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}

{- |
Module      : Hierarchical512Chirho
Description : Multi-width hierarchies (64/256/512-bit) for HBM ☧
Copyright   : (c) 2026
License     : MIT

Three word sizes optimized for different bandwidth/capacity tradeoffs:

| Word Size | HBM Beats | Pack Factor | 2-Level Values |
|-----------|-----------|-------------|----------------|
| 64-bit    | 1/4 beat  | 4 per beat  | 4,096          |
| 256-bit   | 1 beat    | 1 per beat  | 65,536         |
| 512-bit   | 2 beats   | 1/2 beat    | 262,144        |

Domain types provided:
  - Flat64Chirho: 64 values (8 bytes, pack 4 per beat!)
  - Hier4kChirho: 4,096 values (520 bytes)
  - Packed64x4Chirho: 4 Flat64 domains in 1 HBM beat
  - Flat256Chirho: 256 values (32 bytes, 1 beat)
  - Hier65kChirho: 65,536 values (8KB, 1 beat each)
  - Flat512Chirho: 512 values (64 bytes, 2 beats)
  - Hier262kChirho: 262,144 values (33KB, 2 beats each)

Key insight: Use smallest type that fits your domain!
  - 64-bit: Best bandwidth (4× throughput for small domains)
  - 256-bit: Best balance (1 beat, 65k values)
  - 512-bit: Most values (262k per domain)

"For God so loved the world..." - John 3:16
-}
module Hierarchical512Chirho where

import Clash.Prelude

-- ============================================================================
-- 64-bit Types (Quarter HBM Beat - Packable) ☧
-- ============================================================================

-- | 64-bit word = 1/4 HBM beat
--   Advantage: Pack 4 domains per HBM access for batch ops!
type Word64Chirho = BitVector 64

-- | Index into 64-bit word
type Index64Chirho = Index 64

-- ============================================================================
-- 256-bit Types (Single HBM Beat) ☧
-- ============================================================================

-- | 256-bit word = 1 HBM beat (fastest access)
type Word256Chirho = BitVector 256

-- | Index into 256-bit word
type Index256Chirho = Index 256

-- ============================================================================
-- 512-bit Types (Two HBM Beats) ☧
-- ============================================================================

-- | 512-bit word = 2 HBM beats
type Word512Chirho = BitVector 512

-- | Index into 512-bit word
type Index512Chirho = Index 512

-- ============================================================================
-- Flat64Chirho: 64 values (Quarter Beat - Packable) ☧
-- ============================================================================

-- | Simple 64-value domain
--   Advantage: Pack 4 per HBM beat for batch processing!
--   Perfect for: Small enums, digit sets, chess pieces
newtype Flat64Chirho = Flat64Chirho
  { flat64BitsChirho :: Word64Chirho
  } deriving (Generic, NFDataX, Eq, Show)

-- | Empty 64 domain
empty64Chirho :: Flat64Chirho
empty64Chirho = Flat64Chirho 0

-- | Full 64 domain
full64Chirho :: Flat64Chirho
full64Chirho = Flat64Chirho maxBound

-- | Intersection
intersect64Chirho :: Flat64Chirho -> Flat64Chirho -> Flat64Chirho
intersect64Chirho aChirho bChirho =
  Flat64Chirho (flat64BitsChirho aChirho .&. flat64BitsChirho bChirho)

-- | Union
union64Chirho :: Flat64Chirho -> Flat64Chirho -> Flat64Chirho
union64Chirho aChirho bChirho =
  Flat64Chirho (flat64BitsChirho aChirho .|. flat64BitsChirho bChirho)

-- | Check if empty
isEmpty64Chirho :: Flat64Chirho -> Bool
isEmpty64Chirho dChirho = flat64BitsChirho dChirho == 0

-- | Population count
popCount64Chirho :: Flat64Chirho -> BitVector 7
popCount64Chirho dChirho = fromIntegral (popCount (flat64BitsChirho dChirho))

-- ============================================================================
-- Hier4kChirho: 4,096 values (Efficient for Medium Domains) ☧
-- ============================================================================

-- | Two-level hierarchy with 64-bit words
--   64 × 64 = 4,096 values
--
--   HBM: 8 bytes (summary) + 64 × 8 bytes = 520 bytes (~2 beats)
--   Good for: Character classes, small vocabularies, N-Queens board
--
data Hier4kChirho = Hier4kChirho
  { h4kSummaryChirho :: Word64Chirho         -- Which blocks active
  , h4kBlocksChirho  :: Vec 64 Word64Chirho  -- 64 blocks
  } deriving (Generic, NFDataX)

-- | Empty 4k domain
emptyHier4kChirho :: Hier4kChirho
emptyHier4kChirho = Hier4kChirho
  { h4kSummaryChirho = 0
  , h4kBlocksChirho  = repeat 0
  }

-- | Full 4k domain
fullHier4kChirho :: Hier4kChirho
fullHier4kChirho = Hier4kChirho
  { h4kSummaryChirho = maxBound
  , h4kBlocksChirho  = repeat maxBound
  }

-- | Intersection of two 4k domains
intersectHier4kChirho :: Hier4kChirho -> Hier4kChirho -> Hier4kChirho
intersectHier4kChirho aChirho bChirho =
  let newBlocksChirho = zipWith (.&.) (h4kBlocksChirho aChirho) (h4kBlocksChirho bChirho)
      newSummaryChirho = pack (map (/= 0) newBlocksChirho)
  in Hier4kChirho
    { h4kSummaryChirho = newSummaryChirho
    , h4kBlocksChirho  = newBlocksChirho
    }

-- | Union
unionHier4kChirho :: Hier4kChirho -> Hier4kChirho -> Hier4kChirho
unionHier4kChirho aChirho bChirho =
  let newBlocksChirho = zipWith (.|.) (h4kBlocksChirho aChirho) (h4kBlocksChirho bChirho)
      newSummaryChirho = h4kSummaryChirho aChirho .|. h4kSummaryChirho bChirho
  in Hier4kChirho
    { h4kSummaryChirho = newSummaryChirho
    , h4kBlocksChirho  = newBlocksChirho
    }

-- | Check if empty
isEmptyHier4kChirho :: Hier4kChirho -> Bool
isEmptyHier4kChirho dChirho = h4kSummaryChirho dChirho == 0

-- | Check membership
memberHier4kChirho :: BitVector 12 -> Hier4kChirho -> Bool
memberHier4kChirho valChirho dChirho =
  let blockIdxChirho = unpack (slice d11 d6 valChirho) :: Index 64
      bitIdxChirho   = unpack (slice d5 d0 valChirho) :: Index 64
      blockChirho    = (h4kBlocksChirho dChirho) !! blockIdxChirho
  in testBit blockChirho (fromIntegral bitIdxChirho)

-- | Sparse HBM bytes
sparseBytes4kChirho :: Hier4kChirho -> Int
sparseBytes4kChirho dChirho =
  let activeBlocksChirho = popCount (h4kSummaryChirho dChirho)
  in 8 + (activeBlocksChirho * 8)  -- 8 bytes per 64-bit word

-- ============================================================================
-- Packed64Chirho: 4 domains per HBM beat ☧
-- ============================================================================

-- | Four 64-bit domains packed for batch HBM access
--   Read/write 4 variable domains in 1 HBM beat!
data Packed64x4Chirho = Packed64x4Chirho
  { p64d0Chirho :: Flat64Chirho
  , p64d1Chirho :: Flat64Chirho
  , p64d2Chirho :: Flat64Chirho
  , p64d3Chirho :: Flat64Chirho
  } deriving (Generic, NFDataX, Eq, Show)

-- | Pack 4 domains into 256 bits (1 HBM beat)
packDomains64Chirho :: Packed64x4Chirho -> BitVector 256
packDomains64Chirho pChirho =
  (resize (flat64BitsChirho (p64d0Chirho pChirho)) `shiftL` 192) .|.
  (resize (flat64BitsChirho (p64d1Chirho pChirho)) `shiftL` 128) .|.
  (resize (flat64BitsChirho (p64d2Chirho pChirho)) `shiftL` 64) .|.
  resize (flat64BitsChirho (p64d3Chirho pChirho))

-- | Unpack 256 bits into 4 domains
unpackDomains64Chirho :: BitVector 256 -> Packed64x4Chirho
unpackDomains64Chirho bitsChirho = Packed64x4Chirho
  { p64d0Chirho = Flat64Chirho (resize (bitsChirho `shiftR` 192))
  , p64d1Chirho = Flat64Chirho (resize (bitsChirho `shiftR` 128))
  , p64d2Chirho = Flat64Chirho (resize (bitsChirho `shiftR` 64))
  , p64d3Chirho = Flat64Chirho (resize bitsChirho)
  }

-- | Batch intersect 4 domains at once
intersectPacked64Chirho :: Packed64x4Chirho -> Packed64x4Chirho -> Packed64x4Chirho
intersectPacked64Chirho aChirho bChirho = Packed64x4Chirho
  { p64d0Chirho = intersect64Chirho (p64d0Chirho aChirho) (p64d0Chirho bChirho)
  , p64d1Chirho = intersect64Chirho (p64d1Chirho aChirho) (p64d1Chirho bChirho)
  , p64d2Chirho = intersect64Chirho (p64d2Chirho aChirho) (p64d2Chirho bChirho)
  , p64d3Chirho = intersect64Chirho (p64d3Chirho aChirho) (p64d3Chirho bChirho)
  }

-- ============================================================================
-- Hier262k64Chirho: 262,144 values with 64-bit words (Sparse Optimized) ☧
-- ============================================================================

-- | Three-level hierarchy with 64-bit words
--   64 × 64 × 64 = 262,144 values
--
--   Same capacity as Hier262k (512²), but:
--   - 3 levels vs 2 levels
--   - 8 bytes per access vs 64 bytes
--   - 5× better bandwidth for SPARSE domains!
--
--   Use when: Domain is sparse (few active regions)
--   Use Hier262k when: Domain is dense or need fewer round trips
--
data Hier262k64Chirho = Hier262k64Chirho
  { h262k64Level0Chirho :: Word64Chirho                    -- Top summary (64 bits)
  , h262k64Level1Chirho :: Vec 64 Word64Chirho             -- Mid summaries
  , h262k64Level2Chirho :: Vec 64 (Vec 64 Word64Chirho)    -- Actual values
  } deriving (Generic, NFDataX)

-- | Empty 262k-64 domain
emptyHier262k64Chirho :: Hier262k64Chirho
emptyHier262k64Chirho = Hier262k64Chirho
  { h262k64Level0Chirho = 0
  , h262k64Level1Chirho = repeat 0
  , h262k64Level2Chirho = repeat (repeat 0)
  }

-- | Full 262k-64 domain
fullHier262k64Chirho :: Hier262k64Chirho
fullHier262k64Chirho = Hier262k64Chirho
  { h262k64Level0Chirho = maxBound
  , h262k64Level1Chirho = repeat maxBound
  , h262k64Level2Chirho = repeat (repeat maxBound)
  }

-- | Intersection
intersectHier262k64Chirho :: Hier262k64Chirho -> Hier262k64Chirho -> Hier262k64Chirho
intersectHier262k64Chirho aChirho bChirho =
  let newLevel2Chirho = zipWith (zipWith (.&.))
        (h262k64Level2Chirho aChirho) (h262k64Level2Chirho bChirho)
      newLevel1Chirho = map (pack . map (/= 0)) newLevel2Chirho
      newLevel0Chirho = pack (map (/= 0) newLevel1Chirho)
  in Hier262k64Chirho
    { h262k64Level0Chirho = newLevel0Chirho
    , h262k64Level1Chirho = newLevel1Chirho
    , h262k64Level2Chirho = newLevel2Chirho
    }

-- | Check if empty
isEmptyHier262k64Chirho :: Hier262k64Chirho -> Bool
isEmptyHier262k64Chirho dChirho = h262k64Level0Chirho dChirho == 0

-- | Check membership
memberHier262k64Chirho :: BitVector 18 -> Hier262k64Chirho -> Bool
memberHier262k64Chirho valChirho dChirho =
  let l0IdxChirho  = unpack (slice d17 d12 valChirho) :: Index 64
      l1IdxChirho  = unpack (slice d11 d6 valChirho) :: Index 64
      bitIdxChirho = unpack (slice d5 d0 valChirho) :: Index 64
      blockChirho  = ((h262k64Level2Chirho dChirho) !! l0IdxChirho) !! l1IdxChirho
  in testBit blockChirho (fromIntegral bitIdxChirho)

-- | Sparse HBM bytes (much better than 512² for sparse!)
sparseBytes262k64Chirho :: Hier262k64Chirho -> Int
sparseBytes262k64Chirho dChirho =
  let l1ActiveChirho = popCount (h262k64Level0Chirho dChirho)
      l2ActiveChirho = sum $ map popCount (toList (h262k64Level1Chirho dChirho))
  in 8 + (l1ActiveChirho * 8) + (l2ActiveChirho * 8)

-- ============================================================================
-- Adaptive262kChirho: Auto-select 512² vs 64³ ☧
-- ============================================================================

-- | Tag for which representation is active
data Repr262kTagChirho
  = Repr512x512Chirho   -- Dense: use 512² (fewer round trips)
  | Repr64x64x64Chirho  -- Sparse: use 64³ (less bandwidth)
  deriving (Generic, NFDataX, Eq, Show)

-- | Adaptive 262k domain that picks best representation
--   Based on sparsity: if <25% full, use 64³; otherwise 512²
data Adaptive262kChirho = Adaptive262kChirho
  { a262kTagChirho   :: Repr262kTagChirho
  , a262k512Chirho   :: Hier262kChirho      -- 512² representation
  , a262k64Chirho    :: Hier262k64Chirho    -- 64³ representation
  } deriving (Generic, NFDataX)

-- | Estimate sparsity (ratio of active blocks to total)
sparsityHier262kChirho :: Hier262kChirho -> BitVector 10
sparsityHier262kChirho dChirho =
  fromIntegral (popCount (h262kSummaryChirho dChirho))

sparsityHier262k64Chirho :: Hier262k64Chirho -> BitVector 12
sparsityHier262k64Chirho dChirho =
  let l1CountChirho = popCount (h262k64Level0Chirho dChirho)
      l2CountChirho = sum $ map popCount (toList (h262k64Level1Chirho dChirho))
  in fromIntegral (l1CountChirho + l2CountChirho)

-- | Convert 512² to 64³ (when sparsity detected)
toSparse262kChirho :: Hier262kChirho -> Hier262k64Chirho
toSparse262kChirho dChirho =
  -- Each 512-bit block maps to 8 consecutive 64-bit blocks
  -- Block i in 512² → blocks [8i..8i+7] in level 1 of 64³
  -- This is a layout transformation, preserving all bits
  let flatBitsChirho = concatMap (\bChirho -> map (slice512to64Chirho bChirho) (0 :> 1 :> 2 :> 3 :> 4 :> 5 :> 6 :> 7 :> Nil))
                        (h262kBlocksChirho dChirho)
      -- Reshape into 64 × 64 structure
      level2Chirho = unconcat d64 flatBitsChirho
      level1Chirho = map (pack . map (/= 0)) level2Chirho
      level0Chirho = pack (map (/= 0) level1Chirho)
  in Hier262k64Chirho
    { h262k64Level0Chirho = level0Chirho
    , h262k64Level1Chirho = level1Chirho
    , h262k64Level2Chirho = level2Chirho
    }
  where
    slice512to64Chirho :: Word512Chirho -> Index 8 -> Word64Chirho
    slice512to64Chirho wChirho iChirho =
      resize (wChirho `shiftR` (fromIntegral iChirho * 64))

-- ============================================================================
-- Flat256Chirho: 256 values (Single Beat) ☧
-- ============================================================================

-- | Simple 256-value domain (single HBM beat)
--   Perfect for: ASCII, byte values, small enums
newtype Flat256Chirho = Flat256Chirho
  { flat256BitsChirho :: Word256Chirho
  } deriving (Generic, NFDataX, Eq, Show)

-- | Empty 256 domain
empty256Chirho :: Flat256Chirho
empty256Chirho = Flat256Chirho 0

-- | Full 256 domain
full256Chirho :: Flat256Chirho
full256Chirho = Flat256Chirho maxBound

-- | Intersection
intersect256Chirho :: Flat256Chirho -> Flat256Chirho -> Flat256Chirho
intersect256Chirho aChirho bChirho =
  Flat256Chirho (flat256BitsChirho aChirho .&. flat256BitsChirho bChirho)

-- | Union
union256Chirho :: Flat256Chirho -> Flat256Chirho -> Flat256Chirho
union256Chirho aChirho bChirho =
  Flat256Chirho (flat256BitsChirho aChirho .|. flat256BitsChirho bChirho)

-- | Check if empty
isEmpty256Chirho :: Flat256Chirho -> Bool
isEmpty256Chirho dChirho = flat256BitsChirho dChirho == 0

-- | Population count
popCount256Chirho :: Flat256Chirho -> BitVector 9
popCount256Chirho dChirho = fromIntegral (popCount (flat256BitsChirho dChirho))

-- ============================================================================
-- Hier65kChirho: 65,536 values (Single Beat Access) ☧
-- ============================================================================

-- | Two-level hierarchy with 256-bit words
--   256 × 256 = 65,536 values
--
--   Perfect for:
--   - TCP/UDP ports (exactly 65,536!)
--   - Unicode BMP (most common chars)
--   - Extended ASCII with room
--
--   HBM: 32 bytes (summary) + 256 × 32 bytes = 8,224 bytes
--   Each access is 1 HBM beat (fastest!)
--
data Hier65kChirho = Hier65kChirho
  { h65kSummaryChirho :: Word256Chirho           -- Which blocks active
  , h65kBlocksChirho  :: Vec 256 Word256Chirho   -- 256 blocks
  } deriving (Generic, NFDataX)

-- | Empty 65k domain
emptyHier65kChirho :: Hier65kChirho
emptyHier65kChirho = Hier65kChirho
  { h65kSummaryChirho = 0
  , h65kBlocksChirho  = repeat 0
  }

-- | Full 65k domain
fullHier65kChirho :: Hier65kChirho
fullHier65kChirho = Hier65kChirho
  { h65kSummaryChirho = maxBound
  , h65kBlocksChirho  = repeat maxBound
  }

-- | Intersection of two 65k domains
intersectHier65kChirho :: Hier65kChirho -> Hier65kChirho -> Hier65kChirho
intersectHier65kChirho aChirho bChirho =
  let newBlocksChirho = zipWith (.&.) (h65kBlocksChirho aChirho) (h65kBlocksChirho bChirho)
      newSummaryChirho = pack (map (/= 0) newBlocksChirho)
  in Hier65kChirho
    { h65kSummaryChirho = newSummaryChirho
    , h65kBlocksChirho  = newBlocksChirho
    }

-- | Union of two 65k domains
unionHier65kChirho :: Hier65kChirho -> Hier65kChirho -> Hier65kChirho
unionHier65kChirho aChirho bChirho =
  let newBlocksChirho = zipWith (.|.) (h65kBlocksChirho aChirho) (h65kBlocksChirho bChirho)
      newSummaryChirho = h65kSummaryChirho aChirho .|. h65kSummaryChirho bChirho
  in Hier65kChirho
    { h65kSummaryChirho = newSummaryChirho
    , h65kBlocksChirho  = newBlocksChirho
    }

-- | Check if empty
isEmptyHier65kChirho :: Hier65kChirho -> Bool
isEmptyHier65kChirho dChirho = h65kSummaryChirho dChirho == 0

-- | Check membership
memberHier65kChirho :: BitVector 16 -> Hier65kChirho -> Bool
memberHier65kChirho valChirho dChirho =
  let blockIdxChirho = unpack (slice d15 d8 valChirho) :: Index 256
      bitIdxChirho   = unpack (slice d7 d0 valChirho) :: Index 256
      blockChirho    = (h65kBlocksChirho dChirho) !! blockIdxChirho
  in testBit blockChirho (fromIntegral bitIdxChirho)

-- | Insert value
insertHier65kChirho :: BitVector 16 -> Hier65kChirho -> Hier65kChirho
insertHier65kChirho valChirho dChirho =
  let blockIdxChirho = unpack (slice d15 d8 valChirho) :: Index 256
      bitIdxChirho   = fromIntegral (slice d7 d0 valChirho) :: Int
      oldBlockChirho = (h65kBlocksChirho dChirho) !! blockIdxChirho
      newBlockChirho = setBit oldBlockChirho bitIdxChirho
      newBlocksChirho = replace blockIdxChirho newBlockChirho (h65kBlocksChirho dChirho)
      newSummaryChirho = setBit (h65kSummaryChirho dChirho) (fromIntegral blockIdxChirho)
  in Hier65kChirho
    { h65kSummaryChirho = newSummaryChirho
    , h65kBlocksChirho  = newBlocksChirho
    }

-- | Sparse HBM bytes
sparseBytes65kChirho :: Hier65kChirho -> Int
sparseBytes65kChirho dChirho =
  let activeBlocksChirho = popCount (h65kSummaryChirho dChirho)
  in 32 + (activeBlocksChirho * 32)  -- 32 bytes per 256-bit word

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

-- | Bytes per Flat64 variable (1/4 beat - packable!)
bytesFlat64Chirho :: Int
bytesFlat64Chirho = 8

-- | Bytes per Hier4k variable
bytesHier4kChirho :: Int
bytesHier4kChirho = 520  -- 8 + 64*8

-- | Bytes per Flat256 variable (1 beat)
bytesFlat256Chirho :: Int
bytesFlat256Chirho = 32

-- | Bytes per Hier65k variable (1 beat per access)
bytesHier65kChirho :: Int
bytesHier65kChirho = 8224  -- 32 + 256*32

-- | Bytes per Flat512 variable (2 beats)
bytesFlat512Chirho :: Int
bytesFlat512Chirho = 64

-- | Bytes per Hier262k variable (2 beats per access)
bytesHier262kChirho :: Int
bytesHier262kChirho = 32832  -- 64 + 512*64

-- | Bytes per Hier262k64 variable (sparse-optimized)
--   Worst case: 8 + 64*8 + 64*64*8 = 33,288 bytes
--   Typical sparse: 8 + 4*8 + 16*8 = 168 bytes (200× better!)
bytesHier262k64WorstChirho :: Int
bytesHier262k64WorstChirho = 33288

bytesHier262k64TypicalChirho :: Int
bytesHier262k64TypicalChirho = 168  -- ~4 active L1 groups, ~16 active blocks

-- | Maximum variables in 16GB HBM
maxVarsFlat64Chirho :: Int
maxVarsFlat64Chirho = 16 * 1024 * 1024 * 1024 `div` bytesFlat64Chirho  -- 2 BILLION vars!

maxVarsHier4kChirho :: Int
maxVarsHier4kChirho = 16 * 1024 * 1024 * 1024 `div` bytesHier4kChirho  -- ~33M vars

maxVarsFlat256Chirho :: Int
maxVarsFlat256Chirho = 16 * 1024 * 1024 * 1024 `div` bytesFlat256Chirho  -- 536M vars

maxVarsHier65kChirho :: Int
maxVarsHier65kChirho = 16 * 1024 * 1024 * 1024 `div` bytesHier65kChirho  -- ~2M vars

maxVarsHier262kChirho :: Int
maxVarsHier262kChirho = 16 * 1024 * 1024 * 1024 `div` bytesHier262kChirho  -- ~512K vars

-- ============================================================================
-- Synthesis Annotations ☧
-- ============================================================================

{-# ANN intersectHier262k64Chirho
  (Synthesize
    { t_name = "intersect_hier_262k_64_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

{-# ANN intersect64Chirho
  (Synthesize
    { t_name = "intersect_64_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

{-# ANN intersectHier4kChirho
  (Synthesize
    { t_name = "intersect_hier_4k_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

{-# ANN intersectPacked64Chirho
  (Synthesize
    { t_name = "intersect_packed_64_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

{-# ANN intersect256Chirho
  (Synthesize
    { t_name = "intersect_256_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

{-# ANN intersectHier65kChirho
  (Synthesize
    { t_name = "intersect_hier_65k_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

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

| Operation | LUTs | HBM Beats | Bandwidth |
|-----------|------|-----------|-----------|
| Flat64 AND | ~20 | 1/4 | 4× throughput |
| Hier4k intersect | ~1,500 | ~2 | Good for medium |
| Packed64x4 intersect | ~80 | 1 | 4 domains/beat! |
| Flat256 AND | ~80 | 1 | Standard |
| Hier65k intersect | ~20,000 | 1 each | |
| Flat512 AND | ~150 | 2 | |
| Hier262k intersect | ~80,000 | 2 each | |

DOMAIN SELECTION GUIDE:

| Values Needed | Best Choice | HBM Size | Max Vars | Bandwidth |
|---------------|-------------|----------|----------|-----------|
| ≤64 | Flat64/Packed | 8 B | 2B | 4× (packed) |
| ≤4,096 | Hier4k | 520 B | 33M | Good |
| ≤256 | Flat256 | 32 B | 536M | 1× |
| ≤65,536 | Hier65k | 8 KB | 2M | 1× |
| ≤262,144 | Hier262k | 33 KB | 512K | 1/2× |

SaaS APPLICATION FIT:

| Application | Domain | Best Choice | Why |
|-------------|--------|-------------|-----|
| N-Queens | 64 | Flat64/Packed | 4× throughput! |
| TestForge (enums) | ≤64 | Flat64/Packed | Batch process |
| RegexCraft (ASCII) | 256 | Flat256 | Exact fit |
| ConfigGuard (ports) | 65,536 | Hier65k | Exact fit! |
| Philologos (vocab) | ~50,000 | Hier65k | Fits well |
| RegexCraft (Unicode) | ~150,000 | Hier262k | Large domain |

BANDWIDTH COMPARISON (at 14.4 GB/s per HBM channel):

| Type | Throughput | Notes |
|------|------------|-------|
| Packed64x4 | 1.8B ops/sec | 4 domains per beat |
| Flat256 | 450M ops/sec | 1 domain per beat |
| Hier65k | 1.75M ops/sec | 8KB per domain |
| Hier262k | 438K ops/sec | 33KB per domain |

KEY INSIGHT: For small domains (≤64 values), use Packed64x4
for 4× bandwidth efficiency!
-}
