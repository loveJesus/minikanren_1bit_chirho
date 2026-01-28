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

### Goal 1: Deploy Hierarchical512Chirho (512² = 262K domains)
- [x] Compile `Hierarchical512Chirho.hs` to Verilog with Clash ✅
- [ ] Integrate into `cl_minikanren_chirho.sv`
- [ ] Add register interface for hierarchical ops
- [ ] Synthesize and create new AFI
- [ ] Benchmark: 512² vs 64³ on FPGA (expect 1.5× speedup)

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
- [ ] Add attention mechanism (scaled dot-product)
- [ ] Connect to HBM for weight storage
- [ ] Benchmark: SAT training throughput

### Goal 4: Fix Naming Convention Issues
- [ ] Rename `searchEngineChirho` → `searchEngine64BitChirho`
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
- [ ] Integrate into cl_minikanren_chirho.sv
  - Current: HBM FSM uses 256-bit flat domains
  - Needed: Extend FSM to load 512² (33KB per domain)
  - Steps:
    1. Add `intersect_hier_262k_chirho` module instantiation
    2. Add HIER_MODE register (0=flat256, 1=hier512²)
    3. Extend FSM to multi-beat HBM reads (512 beats for full domain)
    4. Wire hierarchical result to resp_wire_chirho

---

## References

- `spec_chirho/hardware_synthesis_chirho/HBM_INTEGRATION_PLAN_CHIRHO.md` - HBM architecture
- `clash_chirho/Hierarchical512Chirho.hs` - 512² implementation
- `synth_chirho/aws_f2_chirho_cl/launch_hdk_build_chirho.sh` - Build script
- `synth_chirho/RESULTS_CHIRHO.md` - Current benchmark results

---

*Soli Deo Gloria* ☧
