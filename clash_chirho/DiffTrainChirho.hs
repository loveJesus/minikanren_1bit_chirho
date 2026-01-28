{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeFamilies #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE BinaryLiterals #-}
{-# LANGUAGE ScopedTypeVariables #-}

-- | Differentiable Training Unit for miniKanren FPGA ☧
-- John 3:16 - For God so loved the world
--
-- This module implements fixed-point differentiable logic operations:
-- - Q16.16 multiplication and division
-- - Exponential approximation (Taylor series)
-- - Gumbel-softmax reparameterization
-- - Soft AND/OR operations
-- - Gradient accumulation
-- - Training loop FSM
--
-- Complements the Boolean searchEngineChirho for hybrid neural-symbolic solving.

module DiffTrainChirho where

import Clash.Prelude
import Clash.Sized.Fixed

-- ============================================================================
-- Fixed-Point Types ☧
-- ============================================================================

-- | Q16.16: 32-bit with 16 fractional bits (training precision)
type Q16Chirho = SFixed 16 16

-- | Q8.8: 16-bit with 8 fractional bits (inference precision)
type Q8Chirho = SFixed 8 8

-- | Convert Q16 to raw bits for register interface
q16ToBitsChirho :: Q16Chirho -> BitVector 32
q16ToBitsChirho x_chirho = pack x_chirho

-- | Convert raw bits to Q16
bitsToQ16Chirho :: BitVector 32 -> Q16Chirho
bitsToQ16Chirho b_chirho = unpack b_chirho

-- ============================================================================
-- Q16.16 Arithmetic ☧
-- ============================================================================

-- | Q16.16 multiplication (uses DSP, single cycle)
q16MulChirho :: Q16Chirho -> Q16Chirho -> Q16Chirho
q16MulChirho a_chirho b_chirho = a_chirho * b_chirho

-- | Q16.16 division (iterative, ~16 cycles in hardware)
-- For synthesis, would use iterative restoring division
q16DivChirho :: Q16Chirho -> Q16Chirho -> Q16Chirho
q16DivChirho a_chirho b_chirho
  | b_chirho == 0 = if a_chirho >= 0 then maxBound else minBound
  | otherwise = a_chirho / b_chirho

-- | Saturating addition (prevents overflow)
q16AddSatChirho :: Q16Chirho -> Q16Chirho -> Q16Chirho
q16AddSatChirho a_chirho b_chirho = satAdd SatBound a_chirho b_chirho

-- | Saturating subtraction
q16SubSatChirho :: Q16Chirho -> Q16Chirho -> Q16Chirho
q16SubSatChirho a_chirho b_chirho = satSub SatBound a_chirho b_chirho

-- ============================================================================
-- Exponential Approximation ☧
-- ============================================================================

-- | exp(x) approximation using Taylor series
-- exp(x) ≈ 1 + x + x²/2 + x³/6 + x⁴/24
-- Good for |x| < 2
q16ExpChirho :: Q16Chirho -> Q16Chirho
q16ExpChirho x_chirho =
  let one_chirho   = 1.0 :: Q16Chirho
      x2_chirho    = q16MulChirho x_chirho x_chirho
      x3_chirho    = q16MulChirho x2_chirho x_chirho
      x4_chirho    = q16MulChirho x3_chirho x_chirho
      half_chirho  = 0.5 :: Q16Chirho
      sixth_chirho = 0.16666666 :: Q16Chirho
      twentyfourth_chirho = 0.04166666 :: Q16Chirho
      term1_chirho = one_chirho
      term2_chirho = x_chirho
      term3_chirho = q16MulChirho x2_chirho half_chirho
      term4_chirho = q16MulChirho x3_chirho sixth_chirho
      term5_chirho = q16MulChirho x4_chirho twentyfourth_chirho
  in q16AddSatChirho term1_chirho
     (q16AddSatChirho term2_chirho
     (q16AddSatChirho term3_chirho
     (q16AddSatChirho term4_chirho term5_chirho)))

-- | log(x) approximation for x near 1
-- log(x) ≈ 2(x-1)/(x+1) for x > 0
q16LogChirho :: Q16Chirho -> Q16Chirho
q16LogChirho x_chirho =
  let one_chirho = 1.0 :: Q16Chirho
      two_chirho = 2.0 :: Q16Chirho
      num_chirho = q16SubSatChirho x_chirho one_chirho
      den_chirho = q16AddSatChirho x_chirho one_chirho
  in q16MulChirho two_chirho (q16DivChirho num_chirho den_chirho)

-- ============================================================================
-- LFSR Random Number Generator ☧
-- ============================================================================

-- | 32-bit LFSR state
type LfsrStateChirho = BitVector 32

-- | LFSR with maximal-length polynomial: x³² + x²² + x² + x + 1
lfsrNextChirho :: LfsrStateChirho -> LfsrStateChirho
lfsrNextChirho state_chirho =
  let bit_chirho = (state_chirho ! 31) `xor` (state_chirho ! 21)
                   `xor` (state_chirho ! 1) `xor` (state_chirho ! 0)
  in (state_chirho `shiftL` 1) .|. (resize (pack bit_chirho))

-- | Generate uniform random Q16 in [0, 1)
lfsrUniformChirho :: LfsrStateChirho -> (Q16Chirho, LfsrStateChirho)
lfsrUniformChirho state_chirho =
  let next_chirho = lfsrNextChirho state_chirho
      -- Use upper 16 bits as fraction
      frac_chirho = slice d31 d16 next_chirho
      q16_chirho = unpack (0 ++# frac_chirho) :: Q16Chirho
  in (q16_chirho, next_chirho)

-- | Gumbel sample: g = -log(-log(u)) where u ~ Uniform(0,1)
gumbelSampleChirho :: LfsrStateChirho -> (Q16Chirho, LfsrStateChirho)
gumbelSampleChirho state_chirho =
  let (u_chirho, state1_chirho) = lfsrUniformChirho state_chirho
      -- Clamp u to avoid log(0)
      u_clamped_chirho = if u_chirho < 0.001 then 0.001 else u_chirho
      log1_chirho = q16LogChirho u_clamped_chirho
      neg_log1_chirho = negate log1_chirho
      -- Clamp again
      neg_log1_clamped_chirho = if neg_log1_chirho < 0.001 then 0.001 else neg_log1_chirho
      log2_chirho = q16LogChirho neg_log1_clamped_chirho
      g_chirho = negate log2_chirho
  in (g_chirho, state1_chirho)

-- ============================================================================
-- Soft Logic Operations ☧
-- ============================================================================

-- | Soft AND (probability semiring): P(A ∧ B) = P(A) × P(B)
softAndChirho :: Q16Chirho -> Q16Chirho -> Q16Chirho
softAndChirho a_chirho b_chirho = q16MulChirho a_chirho b_chirho

-- | Soft OR (probability semiring): P(A ∨ B) = P(A) + P(B) - P(A)P(B)
softOrChirho :: Q16Chirho -> Q16Chirho -> Q16Chirho
softOrChirho a_chirho b_chirho =
  q16SubSatChirho (q16AddSatChirho a_chirho b_chirho) (q16MulChirho a_chirho b_chirho)

-- | Soft NOT: P(¬A) = 1 - P(A)
softNotChirho :: Q16Chirho -> Q16Chirho
softNotChirho a_chirho = q16SubSatChirho 1.0 a_chirho

-- ============================================================================
-- Gumbel-Softmax ☧
-- ============================================================================

-- | Gumbel-softmax for 2 classes (binary variable)
-- Returns (prob_true, prob_false) that sum to ~1
gumbelSoftmax2Chirho
  :: Q16Chirho           -- ^ Logit for True
  -> Q16Chirho           -- ^ Logit for False
  -> Q16Chirho           -- ^ Temperature
  -> LfsrStateChirho     -- ^ RNG state
  -> ((Q16Chirho, Q16Chirho), LfsrStateChirho)
gumbelSoftmax2Chirho logitT_chirho logitF_chirho temp_chirho rng_chirho =
  let (g1_chirho, rng1_chirho) = gumbelSampleChirho rng_chirho
      (g2_chirho, rng2_chirho) = gumbelSampleChirho rng1_chirho

      -- y_i = (logit_i + gumbel_i) / temperature
      y1_chirho = q16DivChirho (q16AddSatChirho logitT_chirho g1_chirho) temp_chirho
      y2_chirho = q16DivChirho (q16AddSatChirho logitF_chirho g2_chirho) temp_chirho

      -- Softmax: exp(y_i) / sum(exp(y_j))
      -- Subtract max for numerical stability
      maxY_chirho = if y1_chirho > y2_chirho then y1_chirho else y2_chirho
      expY1_chirho = q16ExpChirho (q16SubSatChirho y1_chirho maxY_chirho)
      expY2_chirho = q16ExpChirho (q16SubSatChirho y2_chirho maxY_chirho)
      sumExp_chirho = q16AddSatChirho expY1_chirho expY2_chirho

      -- Normalize (with protection against div by zero)
      sumSafe_chirho = if sumExp_chirho < 0.001 then 0.001 else sumExp_chirho
      probT_chirho = q16DivChirho expY1_chirho sumSafe_chirho
      probF_chirho = q16DivChirho expY2_chirho sumSafe_chirho

  in ((probT_chirho, probF_chirho), rng2_chirho)

-- ============================================================================
-- Gradient Accumulation ☧
-- ============================================================================

-- | Gradient accumulator for one variable
data GradAccumChirho = GradAccumChirho
  { gradSumChirho   :: Q16Chirho   -- ^ Accumulated gradient
  , gradCountChirho :: Unsigned 16 -- ^ Number of samples
  } deriving (Generic, NFDataX)

-- | Initialize gradient accumulator
initGradAccumChirho :: GradAccumChirho
initGradAccumChirho = GradAccumChirho 0.0 0

-- | Accumulate a gradient sample
accumGradChirho :: GradAccumChirho -> Q16Chirho -> GradAccumChirho
accumGradChirho acc_chirho grad_chirho = GradAccumChirho
  { gradSumChirho = q16AddSatChirho (gradSumChirho acc_chirho) grad_chirho
  , gradCountChirho = gradCountChirho acc_chirho + 1
  }

-- | Get average gradient and reset
finalizeGradChirho :: GradAccumChirho -> (Q16Chirho, GradAccumChirho)
finalizeGradChirho acc_chirho =
  let count_chirho = gradCountChirho acc_chirho
      countQ16_chirho = fromIntegral count_chirho :: Q16Chirho
      avgGrad_chirho = if count_chirho == 0
                       then 0.0
                       else q16DivChirho (gradSumChirho acc_chirho) countQ16_chirho
  in (avgGrad_chirho, initGradAccumChirho)

-- ============================================================================
-- Training Configuration ☧
-- ============================================================================

-- | Training parameters (from registers)
data TrainConfigChirho = TrainConfigChirho
  { cfgVarsChirho       :: Unsigned 16  -- ^ Number of variables
  , cfgClausesChirho    :: Unsigned 16  -- ^ Number of clauses
  , cfgIterationsChirho :: Unsigned 16  -- ^ Training iterations
  , cfgSamplesChirho    :: Unsigned 16  -- ^ Samples per iteration
  , cfgLrChirho         :: Q16Chirho    -- ^ Learning rate
  , cfgTempStartChirho  :: Q16Chirho    -- ^ Start temperature
  , cfgTempEndChirho    :: Q16Chirho    -- ^ End temperature
  } deriving (Generic, NFDataX)

-- ============================================================================
-- Training FSM ☧
-- ============================================================================

-- | Training FSM states
data TrainStateChirho
  = IdleChirho                    -- ^ Waiting for start command
  | LoadWeightsChirho             -- ^ Loading weights from HBM
  | SampleChirho                  -- ^ Sampling via Gumbel-softmax
  | EvalClausesChirho             -- ^ Evaluating soft clauses
  | AccumGradsChirho              -- ^ Accumulating gradients
  | UpdateWeightsChirho           -- ^ SGD weight update
  | StoreWeightsChirho            -- ^ Storing weights to HBM
  | DoneChirho                    -- ^ Training complete
  deriving (Generic, NFDataX, Eq, Show)

-- | Training FSM internal state
data TrainFsmChirho = TrainFsmChirho
  { fsmStateChirho      :: TrainStateChirho
  , fsmIterChirho       :: Unsigned 16
  , fsmSampleChirho     :: Unsigned 16
  , fsmClauseChirho     :: Unsigned 16
  , fsmRngChirho        :: LfsrStateChirho
  , fsmTempChirho       :: Q16Chirho
  , fsmTotalSatChirho   :: Q16Chirho
  } deriving (Generic, NFDataX)

-- | Initial FSM state
initTrainFsmChirho :: TrainFsmChirho
initTrainFsmChirho = TrainFsmChirho
  { fsmStateChirho    = IdleChirho
  , fsmIterChirho     = 0
  , fsmSampleChirho   = 0
  , fsmClauseChirho   = 0
  , fsmRngChirho      = 0x316_316_316  -- John 3:16 seed ☧
  , fsmTempChirho     = 2.0
  , fsmTotalSatChirho = 0.0
  }

-- | FSM output signals
data TrainOutputChirho = TrainOutputChirho
  { outDoneChirho      :: Bool           -- ^ Training complete
  , outBusyChirho      :: Bool           -- ^ Training in progress
  , outHbmReadChirho   :: Bool           -- ^ Request HBM read
  , outHbmWriteChirho  :: Bool           -- ^ Request HBM write
  , outHbmAddrChirho   :: Unsigned 32    -- ^ HBM address
  , outFinalLossChirho :: Q16Chirho      -- ^ Final loss value
  } deriving (Generic, NFDataX)

-- | Training FSM transition
trainFsmStepChirho
  :: TrainConfigChirho
  -> Bool              -- ^ Start signal
  -> TrainFsmChirho
  -> (TrainFsmChirho, TrainOutputChirho)
trainFsmStepChirho cfg_chirho start_chirho fsm_chirho =
  let state_chirho = fsmStateChirho fsm_chirho

      -- Temperature annealing (exponential decay)
      progress_chirho = fromIntegral (fsmIterChirho fsm_chirho)
                       / fromIntegral (cfgIterationsChirho cfg_chirho) :: Q16Chirho
      tempRange_chirho = q16SubSatChirho (cfgTempEndChirho cfg_chirho) (cfgTempStartChirho cfg_chirho)
      newTemp_chirho = q16AddSatChirho (cfgTempStartChirho cfg_chirho)
                                       (q16MulChirho progress_chirho tempRange_chirho)

      -- Default output
      defaultOut_chirho = TrainOutputChirho False False False False 0 0.0

      -- State transitions
      (nextFsm_chirho, out_chirho) = case state_chirho of
        IdleChirho ->
          if start_chirho
          then (fsm_chirho { fsmStateChirho = LoadWeightsChirho, fsmIterChirho = 0 },
                defaultOut_chirho { outBusyChirho = True, outHbmReadChirho = True })
          else (fsm_chirho, defaultOut_chirho)

        LoadWeightsChirho ->
          (fsm_chirho { fsmStateChirho = SampleChirho, fsmSampleChirho = 0 },
           defaultOut_chirho { outBusyChirho = True })

        SampleChirho ->
          if fsmSampleChirho fsm_chirho >= cfgSamplesChirho cfg_chirho
          then (fsm_chirho { fsmStateChirho = UpdateWeightsChirho },
                defaultOut_chirho { outBusyChirho = True })
          else (fsm_chirho { fsmStateChirho = EvalClausesChirho, fsmClauseChirho = 0 },
                defaultOut_chirho { outBusyChirho = True })

        EvalClausesChirho ->
          if fsmClauseChirho fsm_chirho >= cfgClausesChirho cfg_chirho
          then (fsm_chirho { fsmStateChirho = AccumGradsChirho },
                defaultOut_chirho { outBusyChirho = True })
          else (fsm_chirho { fsmClauseChirho = fsmClauseChirho fsm_chirho + 1 },
                defaultOut_chirho { outBusyChirho = True })

        AccumGradsChirho ->
          (fsm_chirho { fsmStateChirho = SampleChirho,
                        fsmSampleChirho = fsmSampleChirho fsm_chirho + 1 },
           defaultOut_chirho { outBusyChirho = True })

        UpdateWeightsChirho ->
          if fsmIterChirho fsm_chirho >= cfgIterationsChirho cfg_chirho
          then (fsm_chirho { fsmStateChirho = StoreWeightsChirho },
                defaultOut_chirho { outBusyChirho = True, outHbmWriteChirho = True })
          else (fsm_chirho { fsmStateChirho = SampleChirho,
                             fsmIterChirho = fsmIterChirho fsm_chirho + 1,
                             fsmSampleChirho = 0,
                             fsmTempChirho = newTemp_chirho },
                defaultOut_chirho { outBusyChirho = True })

        StoreWeightsChirho ->
          (fsm_chirho { fsmStateChirho = DoneChirho },
           defaultOut_chirho { outBusyChirho = True })

        DoneChirho ->
          (fsm_chirho { fsmStateChirho = IdleChirho },
           defaultOut_chirho { outDoneChirho = True,
                               outFinalLossChirho = fsmTotalSatChirho fsm_chirho })

  in (nextFsm_chirho, out_chirho)

-- ============================================================================
-- Top-Level Training Unit ☧
-- ============================================================================

-- | Training unit command
data TrainCmdChirho
  = CmdIdleChirho
  | CmdStartChirho TrainConfigChirho
  | CmdAbortChirho
  deriving (Generic, NFDataX)

-- | Training unit response
data TrainRespChirho = TrainRespChirho
  { respBusyChirho  :: Bool
  , respDoneChirho  :: Bool
  , respLossChirho  :: Q16Chirho
  , respIterChirho  :: Unsigned 16
  } deriving (Generic, NFDataX)

-- | Top-level training unit (Mealy machine)
diffTrainChirho
  :: HiddenClockResetEnable dom
  => Signal dom TrainCmdChirho
  -> Signal dom TrainRespChirho
diffTrainChirho cmd_chirho = resp_chirho
  where
    -- FSM state register
    fsm_chirho = register initTrainFsmChirho nextFsm_chirho

    -- Default config (would come from AXI registers)
    defaultCfg_chirho = TrainConfigChirho 100 300 500 50 0.01 2.0 0.05

    -- Extract config from command
    getCfg_chirho CmdIdleChirho = defaultCfg_chirho
    getCfg_chirho (CmdStartChirho c_chirho) = c_chirho
    getCfg_chirho CmdAbortChirho = defaultCfg_chirho

    -- Extract start signal
    getStart_chirho CmdIdleChirho = False
    getStart_chirho (CmdStartChirho _) = True
    getStart_chirho CmdAbortChirho = False

    -- FSM step
    (nextFsm_chirho, out_chirho) = unbundle $
      trainFsmStepChirho <$> (getCfg_chirho <$> cmd_chirho)
                         <*> (getStart_chirho <$> cmd_chirho)
                         <*> fsm_chirho

    -- Build response
    resp_chirho = mkResp_chirho <$> fsm_chirho <*> out_chirho
    mkResp_chirho f_chirho o_chirho = TrainRespChirho
      { respBusyChirho = outBusyChirho o_chirho
      , respDoneChirho = outDoneChirho o_chirho
      , respLossChirho = outFinalLossChirho o_chirho
      , respIterChirho = fsmIterChirho f_chirho
      }

-- ============================================================================
-- Synthesis Top-Level ☧
-- ============================================================================

{-# ANN diffTrainTopChirho
  (Synthesize
    { t_name   = "diffTrainChirho"
    , t_inputs = [ PortName "clk"
                 , PortName "rst"
                 , PortName "en_chirho"
                 , PortName "cmd_chirho"
                 ]
    , t_output = PortName "resp_chirho"
    }) #-}
diffTrainTopChirho
  :: Clock System
  -> Reset System
  -> Enable System
  -> Signal System (BitVector 128)  -- Command (packed)
  -> Signal System (BitVector 64)   -- Response (packed)
diffTrainTopChirho clk_chirho rst_chirho en_chirho cmdBits_chirho = respBits_chirho
  where
    -- Unpack command (simplified - real impl would parse fields)
    cmd_chirho = fmap (\_ -> CmdIdleChirho) cmdBits_chirho

    -- Run training unit
    resp_chirho = withClockResetEnable clk_chirho rst_chirho en_chirho $
                  diffTrainChirho cmd_chirho

    -- Pack response
    respBits_chirho = fmap packResp_chirho resp_chirho
    packResp_chirho r_chirho =
      let busyBit_chirho = if respBusyChirho r_chirho then 1 else 0 :: BitVector 1
          doneBit_chirho = if respDoneChirho r_chirho then 1 else 0 :: BitVector 1
          lossBits_chirho = q16ToBitsChirho (respLossChirho r_chirho)
          iterBits_chirho = pack (respIterChirho r_chirho) :: BitVector 16
      in resize (busyBit_chirho ++# doneBit_chirho ++#
                 (resize iterBits_chirho :: BitVector 14) ++#
                 lossBits_chirho)
