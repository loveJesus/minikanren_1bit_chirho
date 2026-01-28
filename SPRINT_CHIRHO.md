# Sprint: Neurosymbolic Architecture + Hierarchical Domains ☧

> *For God so loved the world... - John 3:16*

**Status:** IN PROGRESS
**Started:** 2026-01-28
**Goal:** Deploy 256² and 512² hierarchical domains + neurosymbolic training to F2 FPGA

---

## Current State (What's Deployed)

| Component | Status | Notes |
|-----------|--------|-------|
| `searchEngineChirho` | ✅ Deployed | **64-bit domains only** (from MiniKanrenChirho.hs) |
| HBM FSM Engine | ✅ Deployed | 256-bit flat domains (not hierarchical) |
| `DiffTrainChirho` | ✅ Compiled | Q16.16 fixed-point, benchmarked |
| `Hierarchical512Chirho` | ❌ NOT deployed | Exists in Clash, not compiled to Verilog |
| `HbmEngineChirho` | ❌ NOT deployed | Batch processing FSM |
| `AdaptiveHbmChirho` | ❌ NOT deployed | Auto-select 64/4k/256k domains |

**Critical Gap:** Papers and README claim hierarchical domain support, but only 64-bit is synthesized!

---

## Sprint Goals

### Goal 1: Deploy Hierarchical Domains (262K to 134M values)
- [x] Compile `Hierarchical512Chirho.hs` to Verilog with Clash ✅
  - `intersect_hier_262k_chirho.v` - 512² = 262K values, 512K vars
  - `intersect_hier_65k_chirho.v` - 256² = 65K values, 2M vars
  - **`intersect_hier_16m_chirho.v`** - **256³ = 16.7M values, ~8K vars** ← SWEET SPOT
  - `intersect_hier_134m_chirho.v` - 512³ = 134M values, ~950 vars
- [x] Integrate into `cl_minikanren_chirho.sv` ✅
  - Added `ctrl_hier_mode_chirho` register (3 bits)
  - Extended FSM with `FSM_LOAD_VAR1_BURST_CHIRHO`, `FSM_LOAD_VAR2_BURST_CHIRHO`, etc.
  - Implemented 256² (65K) hierarchical intersection with 256 parallel ANDs
  - Level0 summary gating: skip level1 blocks where either input is zero
- [x] Add register interface for hierarchical ops ✅
  - `REG_HIER_MODE_CHIRHO` (0x40) - Mode selection (0-4)
  - `REG_HIER_LEVEL_CHIRHO` (0x44) - Debug: current level
  - `REG_BEAT_COUNT_CHIRHO` (0x48) - Debug: HBM beat counter
- [x] Extend FSM for all hierarchy modes ✅
  - Mode 0: Flat 256-bit (1 beat)
  - Mode 1: 256² = 65K (257 beats, fully buffered)
  - Mode 2: 512² = 262K (513 beats, fully buffered)
  - Mode 3: 256³ = 16.7M (streaming: summaries buffered, level2 streamed)
  - Mode 4: 512³ = 134M (streaming: summaries buffered, level2 streamed)
- [x] Add streaming FSM states for 3-level hierarchies ✅
  - `FSM_LOAD_SUMMARIES_CHIRHO` - Load level0 + level1 for both vars
  - `FSM_STREAM_LOAD_L2_V1/V2_CHIRHO` - Stream level2 blocks
  - `FSM_STREAM_COMPUTE_CHIRHO` - 256 parallel ANDs on current block
  - `FSM_STREAM_STORE_CHIRHO` - Write result back to HBM
  - `FSM_STREAM_NEXT_BLOCK_CHIRHO` - Advance with sparse skipping
- [ ] Synthesize and create new AFI
- [ ] Benchmark hierarchies on FPGA

### Goal 2: Deploy Full HBM Batch Engine
- [ ] Compile `HbmEngineChirho.hs` to Verilog
- [ ] Connect to AWS F2 HBM AXI4 interface
- [ ] Implement problem queue in HBM Channel 0
- [ ] Implement result queue in HBM Channel 1
- [ ] Benchmark: Batch 1000 problems (target: 80K solves/sec)

### Goal 3: Neurosymbolic Training Hardware
- [x] Verify `DiffTrainChirho.hs` generates correct Verilog ✅
  - Q16.16 fixed-point (32-bit with 16 fractional bits)
  - LFSR random number generator with Gumbel sampling
  - Gumbel-softmax reparameterization
  - Training FSM: Idle → LoadWeights → Sample → EvalClauses → AccumGrads → UpdateWeights → Done
  - Temperature annealing (exponential decay)
- [x] Verify `DiffFixedChirho.hs` generates correct Verilog ✅
  - `soft_and_32_chirho.v` - Q16.16 soft AND (probabilistic)
  - `soft_and_16_chirho.v` - Q8.8 soft AND (inference)
  - `intersect_prob_domain_64_chirho.v` - Probabilistic domain intersection
- [x] Integrate `diffTrainChirho` into `cl_minikanren_chirho.sv` ✅
  - Copied `diffTrainChirho.v` to synthesis directory
  - Added training registers: `REG_TRAIN_MODE_CHIRHO`, `REG_TRAIN_CMD_*_CHIRHO`, `REG_TRAIN_RESP_*_CHIRHO`
  - Instantiated as parallel engine (runs alongside search)
  - Training cmd: learning_rate, temperature, epochs, samples, clause_count
  - Training resp: final_loss, current_epoch, valid, done
- [x] Add probabilistic inference modules ✅
  - `soft_and_32_chirho` - Q16.16 soft AND (scalar)
  - `soft_and_16_chirho` - Q8.8 soft AND (inference)
  - `intersect_prob_domain_64_chirho` - 64-element probabilistic intersection
  - `REG_INFER_MODE_CHIRHO` (0x70) - Boolean vs probabilistic mode
- [ ] Add attention mechanism (scaled dot-product)
- [ ] Connect to HBM for weight storage
- [ ] Benchmark: SAT training throughput

### Goal 4: Fix Naming Convention Issues
- [x] Rename `searchEngineChirho` → `searchEngine64BitChirho` ✅
  - Updated MiniKanrenChirho.hs TopEntity annotation
  - Regenerated searchEngine64BitChirho.v
  - Updated cl_minikanren_chirho.sv module instantiation
- [ ] Add `_CHIRHO` suffix to register constants
- [ ] Update papers to clarify deployed vs. planned features
- [ ] Update README with accurate capability matrix

---

## Clash Modules to Compile

| Module | Size | Purpose | Priority |
|--------|------|---------|----------|
| `Hierarchical512Chirho.hs` | 30 KB | 512² domains (262K values) | **P0** |
| `HbmEngineChirho.hs` | 22 KB | HBM batch processing FSM | **P0** |
| `DiffTrainChirho.hs` | 19 KB | Differentiable training | **P1** |
| `AdaptiveHbmChirho.hs` | 14 KB | Auto domain selection | **P1** |
| `HierarchicalChirho.hs` | 14 KB | 64² (4K) domains | P2 |
| `DiffFixedChirho.hs` | 13 KB | Q16.16/Q8.8 fixed-point | P2 |
| `TermStoreHbmChirho.hs` | 10 KB | Hash-consed terms in HBM | P2 |

---

## Build Scripts (Working)

```bash
# Local Clash compilation
cd clash_chirho
clash --verilog Hierarchical512Chirho.hs

# AWS F2 HDK build
cd synth_chirho/aws_f2_chirho_cl
./prepare_hdk_build_chirho.sh    # Package design for AWS
./launch_hdk_build_chirho.sh     # Run Vivado synthesis on c5.9xlarge

# Create AFI
./scripts/create_afi_chirho.sh   # Submit to AWS for AFI creation
./scripts/check_afi_status_chirho.sh  # Monitor AFI status
```

---

## Register Map Updates Needed

Current (unnamed offsets):
```c
fpga_pci_poke(bar, 0x200, ...);  // BAD: magic number
fpga_pci_poke(bar, 0x500, ...);  // BAD: magic number
```

Target (named constants with _CHIRHO suffix):
```c
#define REG_CMD_LO_CHIRHO      0x010
#define REG_CMD_MID_CHIRHO     0x014
#define REG_CMD_HI_CHIRHO      0x018
#define REG_CTRL_CHIRHO        0x004
#define REG_STATUS_CHIRHO      0x008
#define REG_HIER_MODE_CHIRHO   0x100  // NEW: Select domain type
#define REG_HIER_LEVEL_CHIRHO  0x104  // NEW: Hierarchy depth
```

---

## Architecture: Neurosymbolic Training

```
┌─────────────────────────────────────────────────────────────────┐
│                    Host (CPU)                                    │
│  Upload weights/clauses → Start training → Download results      │
└─────────────────────────────────────────────────────────────────┘
                    │ PCIe/DMA (batch, one-time)
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                  FPGA Fabric (VU47P-HBM)                        │
│                                                                  │
│  ┌────────────┐    ┌───────────────────────────────────────┐   │
│  │ AXI-Lite   │───▶│ Training FSM (DiffTrainChirho)        │   │
│  │ Control    │    │ - Q16.16 fixed-point arithmetic       │   │
│  └────────────┘    │ - Gumbel-softmax (LFSR noise)         │   │
│                    │ - Gradient accumulation               │   │
│  ┌────────────┐    │ - Temperature annealing               │   │
│  │ HBM Ch 0-3 │◀──▶│                                       │   │
│  │ Weights    │    └───────────────────────────────────────┘   │
│  ├────────────┤                    │                           │
│  │ HBM Ch 4-7 │◀──▶┌───────────────┴───────────────────┐      │
│  │ Domains    │    │ Hierarchical Engine (512²)         │      │
│  │ (512² per  │    │ - 512-bit wide operations          │      │
│  │  variable) │    │ - 2-level hierarchy (faster)       │      │
│  └────────────┘    │ - Sparse skip optimization          │      │
│                    └───────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Success Criteria

| Metric | Current | Target | How to Verify |
|--------|---------|--------|---------------|
| Domain size | 64 values | **262,144 values** | Benchmark Hierarchical512 |
| Hierarchy | None | **512² (2-level)** | Compare 512² vs 64³ |
| Batch throughput | 40.5M unify/sec | **200M unify/sec** | HBM batch benchmark |
| Training speedup | 3.3M× (simulated) | **3.3M× (verified)** | Run SAT training on HBM |
| Papers accuracy | Misleading | **Accurate** | Review claims vs. deployed |

---

## Files to Modify

| File | Change |
|------|--------|
| `clash_chirho/Hierarchical512Chirho.hs` | Ensure TopEntity annotation |
| `synth_chirho/aws_f2_chirho_cl/design/cl_minikanren_chirho.sv` | Add hierarchical engine instance |
| `synth_chirho/aws_f2_chirho_cl/benchmarks_chirho/*.c` | Add `_CHIRHO` register constants |
| `README.md` | Clarify deployed vs. planned |
| `README_CHIRHO.md` | Add capability matrix |
| `paper_chirho/paper_chirho.tex` | Add "Current Limitations" subsection |
| `AGENTS.md` | Add sprint tracking section |

---

## Daily Log

### 2026-01-28
- [x] Identified gap: 512² not deployed, only 64-bit
- [x] Stopped F2 instances to save costs
- [x] Created SPRINT_CHIRHO.md
- [x] **Compile Hierarchical512Chirho.hs to Verilog** ✅
  - Fixed type errors (toList → fold + fmap for Vec compatibility)
  - Generated 8 Verilog modules (486 lines total):
    - `intersect_hier_262k_chirho.v` (83 lines) - 512² = 262K values
    - `intersect_hier_65k_chirho.v` (83 lines) - 256² = 65K values
    - `intersect_hier_4k_chirho.v` (83 lines) - 64² = 4K values
    - `intersect_hier_262k_64_chirho.v` (154 lines) - 64³ sparse optimized
    - `intersect_packed_64_chirho.v` (23 lines) - batch 4 domains per beat
    - `intersect_512_chirho.v` (20 lines) - flat 512-bit
    - `intersect_256_chirho.v` (20 lines) - flat 256-bit
    - `intersect_64_chirho.v` (20 lines) - flat 64-bit
  - Copied key modules to `synth_chirho/aws_f2_chirho_cl/design/`
- [x] Integrate into cl_minikanren_chirho.sv ✅
  - Extended FSM to support multi-beat HBM reads for hierarchical domains
  - Added states: `FSM_LOAD_VAR1_BURST_CHIRHO`, `FSM_LOAD_VAR2_BURST_CHIRHO`,
    `FSM_COMPUTE_HIER_CHIRHO`, `FSM_STORE_BURST_CHIRHO`
  - Implemented 256² (65K) domain buffers using BRAM arrays
  - 256 parallel 256-bit AND operations for level1 intersection
  - Level0 gating: if summary bit is 0, skip the level1 block entirely
  - Updated `cl_minikanren_chirho_defines.vh` with:
    - `HIER_MODE_*_CHIRHO` constants for mode selection
    - `REG_*_CHIRHO` register address constants
    - `DOMAIN_SIZE_*_CHIRHO` and `BEATS_*_CHIRHO` for each hierarchy level

---

## References

- `spec_chirho/hardware_synthesis_chirho/HBM_INTEGRATION_PLAN_CHIRHO.md` - HBM architecture
- `clash_chirho/Hierarchical512Chirho.hs` - 512² implementation
- `synth_chirho/aws_f2_chirho_cl/launch_hdk_build_chirho.sh` - Build script
- `synth_chirho/RESULTS_CHIRHO.md` - Current benchmark results

---

*Soli Deo Gloria* ☧
