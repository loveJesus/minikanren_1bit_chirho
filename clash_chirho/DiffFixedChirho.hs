{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}

{- |
Module      : DiffFixedChirho
Description : Fixed-point differentiable operations for FPGA ☧
Copyright   : (c) 2026
License     : MIT

Hardware-friendly differentiable logic using fixed-point arithmetic.
Supports both 16-bit and 32-bit precision.

Resource usage on VU47P:
  - Current design: ~22K LUTs (1.7%)
  - This module: ~500 LUTs (0.04%)
  - Remaining: 1.28M LUTs (98%)

"For God so loved the world..." - John 3:16
-}
module DiffFixedChirho where

import Clash.Prelude

-- ============================================================================
-- Fixed-Point Types ☧
-- ============================================================================

-- | 16-bit fixed-point probability (0.16 format)
--   Range: [0, 0.99998] with precision ~0.0015%
--   0x0000 = 0.0, 0xFFFF = 0.99998
type Prob16Chirho = BitVector 16

-- | 32-bit fixed-point probability (0.32 format)
--   Range: [0, 0.99999999] with precision ~0.00000002%
--   0x00000000 = 0.0, 0xFFFFFFFF = 0.99999999
type Prob32Chirho = BitVector 32

-- | 16-bit fixed-point with sign (1.15 format)
--   Range: [-1, 0.99997] for gradients
type Grad16Chirho = Signed 16

-- | 32-bit fixed-point with sign (1.31 format)
--   Range: [-1, 0.99999999] for gradients
type Grad32Chirho = Signed 32

-- | Log probability (tropical semiring, 8.8 format)
--   Stores -log(p) so small probabilities are large numbers
--   Range: [0, 255.996] representing p in [~0, 1]
type LogProb16Chirho = BitVector 16

-- ============================================================================
-- Conversion Functions ☧
-- ============================================================================

-- | Convert probability to 16-bit fixed (saturating)
toProb16Chirho :: Double -> Prob16Chirho
toProb16Chirho pChirho =
  let clampedChirho = max 0 (min 0.99999 pChirho)
      scaledChirho = clampedChirho * 65536
  in fromInteger (round scaledChirho)

-- | Convert 16-bit fixed back to Double (for debugging)
fromProb16Chirho :: Prob16Chirho -> Double
fromProb16Chirho pChirho = fromIntegral pChirho / 65536.0

-- | Convert probability to 32-bit fixed
toProb32Chirho :: Double -> Prob32Chirho
toProb32Chirho pChirho =
  let clampedChirho = max 0 (min 0.99999999 pChirho)
      scaledChirho = clampedChirho * 4294967296
  in fromInteger (round scaledChirho)

-- | Widen 16-bit to 32-bit (no precision loss)
widen16to32Chirho :: Prob16Chirho -> Prob32Chirho
widen16to32Chirho p16Chirho = resize p16Chirho `shiftL` 16

-- | Narrow 32-bit to 16-bit (truncates lower bits)
narrow32to16Chirho :: Prob32Chirho -> Prob16Chirho
narrow32to16Chirho p32Chirho = resize (p32Chirho `shiftR` 16)

-- ============================================================================
-- 16-bit Probability Operations ☧
-- ============================================================================

-- | Soft AND (16-bit): P(A ∧ B) = P(A) × P(B)
--   ~20 LUTs (16×16 multiply + shift)
softAnd16Chirho :: Prob16Chirho -> Prob16Chirho -> Prob16Chirho
softAnd16Chirho aChirho bChirho =
  let prodChirho = (resize aChirho :: BitVector 32) * resize bChirho
  in resize (prodChirho `shiftR` 16)

-- | Soft OR (16-bit): P(A ∨ B) = P(A) + P(B) - P(A)×P(B)
--   ~40 LUTs
softOr16Chirho :: Prob16Chirho -> Prob16Chirho -> Prob16Chirho
softOr16Chirho aChirho bChirho =
  let sumChirho = (resize aChirho :: BitVector 17) + resize bChirho
      prodChirho = softAnd16Chirho aChirho bChirho
      resultChirho = sumChirho - resize prodChirho
  in saturate16Chirho resultChirho

-- | Soft NOT (16-bit): P(¬A) = 1 - P(A)
--   ~16 LUTs
softNot16Chirho :: Prob16Chirho -> Prob16Chirho
softNot16Chirho aChirho = maxBound - aChirho

-- | Soft equality (16-bit): smooth approximation of (a == b)
--   Uses exp(-|a-b|/temp) approximated by (1 - |a-b|/temp)
--   ~30 LUTs
softEq16Chirho :: Prob16Chirho -> Prob16Chirho -> Prob16Chirho -> Prob16Chirho
softEq16Chirho aChirho bChirho tempChirho =
  let diffChirho = if aChirho > bChirho
                   then aChirho - bChirho
                   else bChirho - aChirho
      scaledDiffChirho = (resize diffChirho :: BitVector 32) * 65536
                         `div` max 1 (resize tempChirho)
  in if scaledDiffChirho > 65535
     then 0
     else maxBound - resize scaledDiffChirho

-- | Saturating addition (prevents overflow)
saturate16Chirho :: BitVector 17 -> Prob16Chirho
saturate16Chirho xChirho =
  if testBit xChirho 16
  then maxBound  -- Overflow → clamp to max
  else resize xChirho

-- ============================================================================
-- 32-bit Probability Operations ☧
-- ============================================================================

-- | Soft AND (32-bit): higher precision
--   ~40 LUTs (32×32 multiply + shift)
softAnd32Chirho :: Prob32Chirho -> Prob32Chirho -> Prob32Chirho
softAnd32Chirho aChirho bChirho =
  let prodChirho = (resize aChirho :: BitVector 64) * resize bChirho
  in resize (prodChirho `shiftR` 32)

-- | Soft OR (32-bit)
--   ~60 LUTs
softOr32Chirho :: Prob32Chirho -> Prob32Chirho -> Prob32Chirho
softOr32Chirho aChirho bChirho =
  let sumChirho = (resize aChirho :: BitVector 33) + resize bChirho
      prodChirho = softAnd32Chirho aChirho bChirho
      resultChirho = sumChirho - resize prodChirho
  in saturate32Chirho resultChirho

-- | Soft NOT (32-bit)
softNot32Chirho :: Prob32Chirho -> Prob32Chirho
softNot32Chirho aChirho = maxBound - aChirho

-- | Saturating addition (32-bit)
saturate32Chirho :: BitVector 33 -> Prob32Chirho
saturate32Chirho xChirho =
  if testBit xChirho 32
  then maxBound
  else resize xChirho

-- ============================================================================
-- Gradient Operations ☧
-- ============================================================================

-- | Gradient of soft AND: ∂(a×b)/∂a = b, ∂(a×b)/∂b = a
--   Returns (grad_a, grad_b) scaled by upstream gradient
gradAnd16Chirho
  :: Prob16Chirho      -- ^ a
  -> Prob16Chirho      -- ^ b
  -> Grad16Chirho      -- ^ upstream gradient
  -> (Grad16Chirho, Grad16Chirho)
gradAnd16Chirho aChirho bChirho upstreamChirho =
  let gradAChirho = resize ((resize upstreamChirho :: Signed 32)
                           * resize (unpack bChirho :: Signed 16) `shiftR` 16)
      gradBChirho = resize ((resize upstreamChirho :: Signed 32)
                           * resize (unpack aChirho :: Signed 16) `shiftR` 16)
  in (gradAChirho, gradBChirho)

-- | Gradient of soft OR: ∂(a+b-ab)/∂a = 1-b, ∂(a+b-ab)/∂b = 1-a
gradOr16Chirho
  :: Prob16Chirho
  -> Prob16Chirho
  -> Grad16Chirho
  -> (Grad16Chirho, Grad16Chirho)
gradOr16Chirho aChirho bChirho upstreamChirho =
  let oneMinusBChirho = maxBound - bChirho
      oneMinusAChirho = maxBound - aChirho
      gradAChirho = resize ((resize upstreamChirho :: Signed 32)
                           * resize (unpack oneMinusBChirho :: Signed 16) `shiftR` 16)
      gradBChirho = resize ((resize upstreamChirho :: Signed 32)
                           * resize (unpack oneMinusAChirho :: Signed 16) `shiftR` 16)
  in (gradAChirho, gradBChirho)

-- | Gradient accumulator (32-bit for precision)
--   Accumulates gradients without overflow
type GradAccumChirho = Signed 32

-- | Add gradient to accumulator with clipping
accumGradChirho :: GradAccumChirho -> Grad16Chirho -> GradAccumChirho
accumGradChirho accumChirho gradChirho =
  let newAccumChirho = accumChirho + resize gradChirho
  in clipGradChirho newAccumChirho

-- | Gradient clipping to prevent explosion
clipGradChirho :: GradAccumChirho -> GradAccumChirho
clipGradChirho gChirho
  | gChirho > 32767  = 32767   -- Max positive
  | gChirho < -32768 = -32768  -- Max negative
  | otherwise        = gChirho

-- ============================================================================
-- Log-Domain (Tropical Semiring) ☧
-- ============================================================================

{-
For very small probabilities, log-domain prevents underflow:
  - Store -log(p) instead of p
  - Multiplication becomes addition
  - Very small p (like 1e-10) becomes manageable number (~23)
-}

-- | Convert probability to log-domain (8.8 fixed point)
--   -log(p) where log is natural log
toLogProb16Chirho :: Double -> LogProb16Chirho
toLogProb16Chirho pChirho =
  let logValChirho = negate (log (max 1e-20 pChirho))
      scaledChirho = logValChirho * 256  -- 8.8 format
  in fromInteger (min 65535 (round scaledChirho))

-- | Tropical AND: -log(a×b) = -log(a) + -log(b)
--   Just addition! Much simpler than multiply.
tropicalAnd16Chirho :: LogProb16Chirho -> LogProb16Chirho -> LogProb16Chirho
tropicalAnd16Chirho aChirho bChirho = saturateLogChirho (resize aChirho + resize bChirho)

-- | Tropical OR: -log(a+b) ≈ min(-log(a), -log(b)) for small probs
--   This is the "min-plus" semiring
tropicalOr16Chirho :: LogProb16Chirho -> LogProb16Chirho -> LogProb16Chirho
tropicalOr16Chirho aChirho bChirho = min aChirho bChirho

-- | Saturate log probability
saturateLogChirho :: BitVector 17 -> LogProb16Chirho
saturateLogChirho xChirho =
  if testBit xChirho 16 then maxBound else resize xChirho

-- ============================================================================
-- Domain with Probability ☧
-- ============================================================================

-- | Probabilistic domain: each bit has an associated probability
--   For 64-value domain, we have 64 probabilities
data ProbDomain64Chirho = ProbDomain64Chirho
  { pdBitsChirho  :: BitVector 64          -- Which values possible (hard constraint)
  , pdProbsChirho :: Vec 64 Prob16Chirho   -- Probability for each value
  } deriving (Generic, NFDataX)

-- | Intersect probabilistic domains
--   Bits: AND (hard constraint)
--   Probs: multiply (soft constraint)
intersectProbDomain64Chirho
  :: ProbDomain64Chirho
  -> ProbDomain64Chirho
  -> ProbDomain64Chirho
intersectProbDomain64Chirho aChirho bChirho = ProbDomain64Chirho
  { pdBitsChirho = pdBitsChirho aChirho .&. pdBitsChirho bChirho
  , pdProbsChirho = zipWith softAnd16Chirho (pdProbsChirho aChirho) (pdProbsChirho bChirho)
  }

-- | Union probabilistic domains
--   Bits: OR
--   Probs: probabilistic sum
unionProbDomain64Chirho
  :: ProbDomain64Chirho
  -> ProbDomain64Chirho
  -> ProbDomain64Chirho
unionProbDomain64Chirho aChirho bChirho = ProbDomain64Chirho
  { pdBitsChirho = pdBitsChirho aChirho .|. pdBitsChirho bChirho
  , pdProbsChirho = zipWith softOr16Chirho (pdProbsChirho aChirho) (pdProbsChirho bChirho)
  }

-- | Total probability mass in domain (sum of probs for set bits)
totalProbChirho :: ProbDomain64Chirho -> Prob32Chirho
totalProbChirho dChirho =
  let maskedChirho = zipWith
        (\bitChirho probChirho -> if bitChirho then widen16to32Chirho probChirho else 0)
        (unpack (pdBitsChirho dChirho) :: Vec 64 Bool)
        (pdProbsChirho dChirho)
  in fold saturateAdd32Chirho maskedChirho
  where
    saturateAdd32Chirho aChirho bChirho =
      let sumChirho = (resize aChirho :: BitVector 33) + resize bChirho
      in saturate32Chirho sumChirho

-- ============================================================================
-- Resource Summary ☧
-- ============================================================================

{-
LUT ESTIMATES FOR THIS MODULE:

| Operation | LUTs | Notes |
|-----------|------|-------|
| softAnd16 | ~20 | 16×16 mult + shift |
| softOr16 | ~40 | 2 adds + mult |
| softNot16 | ~16 | subtraction |
| softEq16 | ~30 | abs + div + sub |
| softAnd32 | ~40 | 32×32 mult |
| softOr32 | ~60 | 2 adds + mult |
| gradAnd16 | ~40 | 2 mults |
| gradOr16 | ~50 | 2 mults + 2 subs |
| tropicalAnd | ~20 | just addition |
| tropicalOr | ~10 | just min |
| intersectProbDomain64 | ~1500 | 64 parallel softAnd |
| unionProbDomain64 | ~3000 | 64 parallel softOr |

TOTAL MODULE: ~500-5000 LUTs depending on what's instantiated

COMPARED TO VU47P CAPACITY:
  - Available: 1,303,680 LUTs
  - This module: ~5,000 LUTs max
  - Usage: 0.4%

PLENTY OF ROOM! ✅
-}

-- ============================================================================
-- Synthesis Annotations ☧
-- ============================================================================

{-# ANN softAnd16Chirho
  (Synthesize
    { t_name = "soft_and_16_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

{-# ANN softAnd32Chirho
  (Synthesize
    { t_name = "soft_and_32_chirho"
    , t_inputs = [PortName "a_chirho", PortName "b_chirho"]
    , t_output = PortName "result_chirho"
    }) #-}

{-# ANN intersectProbDomain64Chirho
  (Synthesize
    { t_name = "intersect_prob_domain_64_chirho"
    , t_inputs = [PortName "domain_a_chirho", PortName "domain_b_chirho"]
    , t_output = PortName "domain_out_chirho"
    }) #-}
