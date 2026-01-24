{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE TemplateHaskell #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}

{- |
Module      : MiniKanren_chirho
Description : miniKanren as 1-bit matrix operations for FPGA ☧
Copyright   : (c) 2024
License     : MIT

Clash implementation of miniKanren search primitives.
Compiles to Verilog/VHDL for FPGA synthesis.

"Whether therefore ye eat, or drink, or whatsoever ye do, 
 do all to the glory of God." — 1 Corinthians 10:31
-}
module MiniKanren_chirho where

import Clash.Prelude
import GHC.Generics (Generic)
import Control.DeepSeq (NFData)

-- | 64-bit domain representing possible values for a variable
-- Bit i = 1 means value i is possible
type Domain_chirho = BitVector 64

-- | Variable index (supports up to 8 variables)
type VarIdx_chirho = Index 8

-- | Search state: N variables with 64-bit domains
data SearchState_chirho (n :: Nat) = SearchState_chirho
  { domains_chirho :: Vec n Domain_chirho
  , valid_chirho   :: Bool
  } deriving (Generic, NFData, Show, Eq, Bundle)

-- | Full domain: all values possible
fullDomain_chirho :: Domain_chirho
fullDomain_chirho = maxBound  -- All 1s

-- | Empty domain: no values possible (failure)
emptyDomain_chirho :: Domain_chirho
emptyDomain_chirho = 0

-- | Initialize search state with all full domains
initState_chirho :: KnownNat n => SearchState_chirho n
initState_chirho = SearchState_chirho
  { domains_chirho = repeat fullDomain_chirho
  , valid_chirho = True
  }

-- | Unification: AND two domains together (single cycle)
-- Core operation: domain1 ∩ domain2
unify_chirho :: Domain_chirho -> Domain_chirho -> Domain_chirho
unify_chirho d1_chirho d2_chirho = d1_chirho .&. d2_chirho

-- | Check if domain is empty (failure)
isEmpty_chirho :: Domain_chirho -> Bool
isEmpty_chirho d_chirho = d_chirho == 0

-- | Check if domain is singleton (exactly one value)
isSingleton_chirho :: Domain_chirho -> Bool
isSingleton_chirho x_chirho = 
  x_chirho /= 0 && (x_chirho .&. (x_chirho - 1)) == 0

-- | Get lowest set bit (isolate one possible value)
lowestBit_chirho :: Domain_chirho -> Domain_chirho
lowestBit_chirho x_chirho = x_chirho .&. negate x_chirho

-- | Clear lowest set bit (remaining possibilities)
clearLowest_chirho :: Domain_chirho -> Domain_chirho
clearLowest_chirho x_chirho = x_chirho .&. (x_chirho - 1)

-- | Fork: split domain into two branches
-- Returns (lowest single value, remaining values)
fork_chirho :: Domain_chirho -> (Domain_chirho, Domain_chirho)
fork_chirho x_chirho = (lowestBit_chirho x_chirho, clearLowest_chirho x_chirho)

-- | Disjunction: OR two domains (for conde)
disj_chirho :: Domain_chirho -> Domain_chirho -> Domain_chirho
disj_chirho d1_chirho d2_chirho = d1_chirho .|. d2_chirho

-- | Population count: number of possible values
popCount_chirho :: Domain_chirho -> Index 65
popCount_chirho = fromIntegral . popCount

-- | Unify two variables in a search state
unifyVars_chirho 
  :: KnownNat n 
  => Index n 
  -> Index n 
  -> SearchState_chirho n 
  -> SearchState_chirho n
unifyVars_chirho v1_chirho v2_chirho state_chirho
  | not (valid_chirho state_chirho) = state_chirho  -- Already failed
  | otherwise = 
      let d1_chirho = (domains_chirho state_chirho) !! v1_chirho
          d2_chirho = (domains_chirho state_chirho) !! v2_chirho
          result_chirho = unify_chirho d1_chirho d2_chirho
          newDomains_chirho = replace v1_chirho result_chirho 
                           $ replace v2_chirho result_chirho 
                           $ domains_chirho state_chirho
      in SearchState_chirho
          { domains_chirho = newDomains_chirho
          , valid_chirho = not (isEmpty_chirho result_chirho)
          }

-- | Constrain a variable to a specific value
constrain_chirho 
  :: KnownNat n 
  => Index n 
  -> Domain_chirho 
  -> SearchState_chirho n 
  -> SearchState_chirho n
constrain_chirho v_chirho mask_chirho state_chirho
  | not (valid_chirho state_chirho) = state_chirho
  | otherwise =
      let d_chirho = (domains_chirho state_chirho) !! v_chirho
          result_chirho = unify_chirho d_chirho mask_chirho
          newDomains_chirho = replace v_chirho result_chirho (domains_chirho state_chirho)
      in SearchState_chirho
          { domains_chirho = newDomains_chirho
          , valid_chirho = not (isEmpty_chirho result_chirho)
          }

-- | Branch on a variable: create two alternative states
branch_chirho 
  :: KnownNat n 
  => Index n 
  -> SearchState_chirho n 
  -> (SearchState_chirho n, SearchState_chirho n)
branch_chirho v_chirho state_chirho
  | not (valid_chirho state_chirho) = (state_chirho, state_chirho)
  | otherwise =
      let d_chirho = (domains_chirho state_chirho) !! v_chirho
          (lo_chirho, hi_chirho) = fork_chirho d_chirho
          state1_chirho = constrain_chirho v_chirho lo_chirho state_chirho
          state2_chirho = constrain_chirho v_chirho hi_chirho state_chirho
      in (state1_chirho, state2_chirho)

-- | Check if state represents a complete solution (all singletons)
isSolution_chirho :: KnownNat n => SearchState_chirho n -> Bool
isSolution_chirho state_chirho = 
  valid_chirho state_chirho && 
  all isSingleton_chirho (domains_chirho state_chirho)

-- | Extract solution values (assumes isSolution_chirho is True)
extractSolution_chirho :: KnownNat n => SearchState_chirho n -> Vec n (Index 64)
extractSolution_chirho state_chirho = 
  map (fromIntegral . countTrailingZeros) (domains_chirho state_chirho)

-------------------------------------------------------------------------------
-- Hardware: Mealy machine for search engine
-------------------------------------------------------------------------------

-- | Search engine command
data SearchCmd_chirho
  = Init_chirho                           -- Reset to initial state
  | Unify_chirho VarIdx_chirho VarIdx_chirho  -- Unify two variables
  | Constrain_chirho VarIdx_chirho Domain_chirho  -- Constrain variable to domain
  | Branch_chirho VarIdx_chirho           -- Branch on variable (push to stack)
  | Backtrack_chirho                      -- Pop from stack and continue
  | Nop_chirho                            -- No operation
  deriving (Generic, NFData, Show, Eq, Bundle)

-- | Search engine response  
data SearchResp_chirho = SearchResp_chirho
  { respValid_chirho    :: Bool
  , respSolution_chirho :: Bool
  , respDomains_chirho  :: Vec 8 Domain_chirho
  } deriving (Generic, NFData, Show, Eq, Bundle)

-- | Stack entry for backtracking
data StackEntry_chirho = StackEntry_chirho
  { stackVar_chirho    :: VarIdx_chirho
  , stackDomain_chirho :: Domain_chirho
  , stackDomains_chirho :: Vec 8 Domain_chirho
  } deriving (Generic, NFData, Show, Eq, Bundle)

-- | Search engine state
data EngineState_chirho = EngineState_chirho
  { engDomains_chirho :: Vec 8 Domain_chirho
  , engValid_chirho   :: Bool
  , engStack_chirho   :: Vec 16 (Maybe StackEntry_chirho)
  , engSP_chirho      :: Index 16
  } deriving (Generic, NFData, Show, Eq, Bundle)

-- | Initial engine state
initEngine_chirho :: EngineState_chirho
initEngine_chirho = EngineState_chirho
  { engDomains_chirho = repeat fullDomain_chirho
  , engValid_chirho = True
  , engStack_chirho = repeat Nothing
  , engSP_chirho = 0
  }

-- | Search engine transition function (Mealy machine)
engineStep_chirho :: EngineState_chirho -> SearchCmd_chirho -> (EngineState_chirho, SearchResp_chirho)
engineStep_chirho st_chirho cmd_chirho = (st'_chirho, resp_chirho)
  where
    -- Process command
    st'_chirho = case cmd_chirho of
      Init_chirho -> initEngine_chirho
      
      Unify_chirho v1_chirho v2_chirho ->
        let d1_chirho = (engDomains_chirho st_chirho) !! v1_chirho
            d2_chirho = (engDomains_chirho st_chirho) !! v2_chirho
            result_chirho = unify_chirho d1_chirho d2_chirho
            newDomains_chirho = replace v1_chirho result_chirho 
                              $ replace v2_chirho result_chirho 
                              $ engDomains_chirho st_chirho
        in st_chirho 
            { engDomains_chirho = newDomains_chirho
            , engValid_chirho = engValid_chirho st_chirho && not (isEmpty_chirho result_chirho)
            }
      
      Constrain_chirho v_chirho mask_chirho ->
        let d_chirho = (engDomains_chirho st_chirho) !! v_chirho
            result_chirho = unify_chirho d_chirho mask_chirho
            newDomains_chirho = replace v_chirho result_chirho (engDomains_chirho st_chirho)
        in st_chirho
            { engDomains_chirho = newDomains_chirho
            , engValid_chirho = engValid_chirho st_chirho && not (isEmpty_chirho result_chirho)
            }
      
      Branch_chirho v_chirho ->
        let d_chirho = (engDomains_chirho st_chirho) !! v_chirho
            (lo_chirho, hi_chirho) = fork_chirho d_chirho
            -- Push alternative to stack
            entry_chirho = StackEntry_chirho v_chirho hi_chirho (engDomains_chirho st_chirho)
            newStack_chirho = replace (engSP_chirho st_chirho) (Just entry_chirho) (engStack_chirho st_chirho)
            newSP_chirho = engSP_chirho st_chirho + 1
            -- Continue with first choice
            newDomains_chirho = replace v_chirho lo_chirho (engDomains_chirho st_chirho)
        in st_chirho
            { engDomains_chirho = newDomains_chirho
            , engStack_chirho = newStack_chirho
            , engSP_chirho = newSP_chirho
            , engValid_chirho = engValid_chirho st_chirho && not (isEmpty_chirho lo_chirho)
            }
      
      Backtrack_chirho ->
        if engSP_chirho st_chirho == 0
          then st_chirho { engValid_chirho = False }  -- No more alternatives
          else 
            let newSP_chirho = engSP_chirho st_chirho - 1
                entry_chirho = (engStack_chirho st_chirho) !! newSP_chirho
            in case entry_chirho of
                Nothing -> st_chirho { engValid_chirho = False }
                Just e_chirho -> st_chirho
                    { engDomains_chirho = replace (stackVar_chirho e_chirho) 
                                                  (stackDomain_chirho e_chirho) 
                                                  (stackDomains_chirho e_chirho)
                    , engSP_chirho = newSP_chirho
                    , engValid_chirho = not (isEmpty_chirho (stackDomain_chirho e_chirho))
                    }
      
      Nop_chirho -> st_chirho
    
    -- Build response
    resp_chirho = SearchResp_chirho
      { respValid_chirho = engValid_chirho st'_chirho
      , respSolution_chirho = engValid_chirho st'_chirho && 
                              all isSingleton_chirho (engDomains_chirho st'_chirho)
      , respDomains_chirho = engDomains_chirho st'_chirho
      }

-- | Top-level search engine (synthesizable)
{-# ANN searchEngine_chirho
  (Synthesize
    { t_name   = "search_engine_chirho"
    , t_inputs = [PortName "clk", PortName "rst", PortName "cmd_chirho"]
    , t_output = PortName "resp_chirho"
    }) #-}
searchEngine_chirho 
  :: Clock System
  -> Reset System
  -> Signal System SearchCmd_chirho
  -> Signal System SearchResp_chirho
searchEngine_chirho clk_chirho rst_chirho cmd_chirho = 
  mealy engineStep_chirho initEngine_chirho cmd_chirho

-------------------------------------------------------------------------------
-- Soli Deo Gloria ☧
-------------------------------------------------------------------------------
