# Build History - miniKanren FPGA ☧

## Version Summary

| Version | Directory | Target | Status | Notes |
|---------|-----------|--------|--------|-------|
| v0 | `v0_f1_synth_chirho/` | F1 | Exploratory | Early Yosys experiments |
| v1 | `v1_aws_f1_chirho/` | F1 | Incomplete | F1 instances scarce |
| v2 | `v2_aws_f2_chirho/` | F2 | ✅ Working | Basic 64-bit domains |
| v3 | `v3_aws_f2_chirho_cl/` | F2 | ✅ Working | CL wrapper, HBM integration |
| v4 | `v4_aws_f2_hier_ns_chirho_cl/` | F2 | ❌ Failed | Routing congestion |
| v5 | `v5_aws_f2_floorplan_chirho_cl/` | F2 | 🔄 In Progress | No pblocks + 200MHz |
| v5.1 | `v5_aws_f2_floorplan_chirho_cl/` | F2 | ❌ Failed | HBM pblock_CL conflict |
| v5.2 | `v5_aws_f2_floorplan_chirho_cl/` | F2 | ❌ Failed | CLB packing overflow |
| v5.3 | `v5_aws_f2_floorplan_chirho_cl/` | F2 | 🔄 Pending | Sparse streaming |

---

## v5.2 Post-Mortem: CLB Packing Overflow

### Build Details
- **Date:** 2026-01-30
- **Instance:** c5.9xlarge (72GB RAM)
- **Duration:** ~1h 13m (failed during placement)
- **Clock:** 250MHz (A2 recipe)

### What Failed
```
ERROR: [Place 30-487] The packing of instances into a set of CLBs defined by
       a pblock constraint could not be obeyed. There are a total of 142215
       CLBs in the pblock, of which 33562 CLBs are available, however, the
       unplaced instances require 35437 CLBs.
```

**Root Cause:** Parallel generate blocks force arrays to be registers, not BRAM.

```systemverilog
// This reads ALL 256 words simultaneously - forces register implementation!
for (int gi = 0; gi < 256; gi++) begin
    hier_65k_result_level1_chirho[gi] <=
        hier_65k_1_level1_chirho[gi] & hier_65k_2_level1_chirho[gi];
end
```

**Resource consumption:**
- 65K engine: 256 × 256 bits × 3 arrays = **196,608 FFs**
- 262K engine: 512 × 512 bits × 3 arrays = **786,432 FFs**
- Total hierarchical buffers: **~983,000 FFs** (vs ~1.2M available in pblock_CL)
- Plus 65,536 fanout on CE signals causing routing congestion

### Fix for v5.3: Sparse Streaming

**Key insight:** Use level0 summary to skip zero blocks.

```
1. Load level0_A and level0_B (256 or 512 bits each)
2. Compute level0_result = level0_A & level0_B
3. For each bit i where level0_result[i] == 1:
   - Stream level1_A[i] from HBM channel 0
   - Stream level1_B[i] from HBM channel 1
   - Compute level1_result[i] = level1_A[i] & level1_B[i]
   - Stream result to HBM channel 2
4. Skip all zero blocks (no HBM access needed)
```

**Expected resource reduction:**

| Component | V5.2 (parallel) | V5.3 (streaming) |
|-----------|-----------------|------------------|
| 65K level1 buffers | 196,608 FFs | 768 FFs |
| 262K level1 buffers | 786,432 FFs | 1,536 FFs |
| Index FIFO | 0 | ~512 FFs |
| **Total** | **~983,000 FFs** | **~3,000 FFs** |

**Performance (typical sparse domain, 10 bits set in level0):**

| Metric | V5.2 (parallel) | V5.3 (streaming) |
|--------|-----------------|------------------|
| HBM reads | 1026 beats | 22 beats |
| Compute cycles | 1 | 10 |
| Total latency | ~7μs | **~50ns** |
| Speedup | 1× | **140×** |

---

## v5.1 Post-Mortem: HBM Pblock Conflict

### Build Details
- **Date:** 2026-01-29
- **Instance:** c5.9xlarge (72GB RAM)
- **Duration:** ~4 minutes (failed during placement)
- **Clock:** 200MHz (A1 recipe)

### What Failed
```
ERROR: [Place 30-1093] Failed to place WRAPPER/CL/HBM_ENABLED.HBM_AXI4_CHIRHO/
       HBM_PRESENT_EQ_1.HBM_WRAPPER_I/HBM_CORE_I/inst/TWO_STACK.u_hbm_top/
       TWO_STACK_HBM.hbm_two_stack_intf/HBM_ONE_STACK_INTF<0>_INST
       on device because ... placed on site BLI_HBM_APB_INTF_X8Y0 ...
       is outside its area constraints. Inst PBlock: pblock_CL.
```

**Root Cause:** AWS HDK shell creates `pblock_CL` for the reconfigurable CL region.
HBM IP has **fixed physical locations** at `BLI_HBM_APB_INTF` sites at chip edges.
These sites are outside `pblock_CL` bounds.

When HBM is instantiated inside the CL hierarchy (`WRAPPER/CL/HBM_ENABLED...`),
Vivado tries to place it within `pblock_CL`, which conflicts with HBM's fixed sites.

### Fix for v5.2
Add constraints to exclude HBM cells from `pblock_CL`:
```tcl
set hbm_cells [get_cells -hierarchical -filter {NAME =~ *HBM*} -quiet]
if {[llength $hbm_cells] > 0} {
    foreach cell $hbm_cells {
        set pblock [get_pblocks -of_objects $cell -quiet]
        if {[llength $pblock] > 0} {
            remove_cells_from_pblock $pblock $cell
        }
    }
}
```

---

## v4 Post-Mortem: Routing Congestion Failure

### Build Details
- **Date:** 2026-01-28
- **Instance:** r5.8xlarge (256GB RAM)
- **Duration:** ~5 hours
- **Clock:** 250MHz (A2 recipe)

### What Failed
```
ERROR: [Route 35-4445] route_design is terminated due to errors/critical warnings
       issued before and during initial routing.
```

**Root Cause:** Congestion level 7/8 on 128x128 grid
- All hierarchical engines placed in SLR0 (near HBM)
- 256-bit wide HBM data paths created routing bottleneck
- Too much logic density in single SLR

### Phases Completed
| Phase | Time | Status |
|-------|------|--------|
| Synthesis | 1:05:27 | ✅ |
| Link Design | ~35 min | ✅ |
| opt_design | 3:22 | ✅ |
| place_design | 1:00:11 | ✅ |
| phys_opt_design | 1:31:53 | ✅ |
| route_design | 56:26 | ❌ Failed |

### Memory Usage
- Peak: 152 GB
- Instance: 256 GB (r5.8xlarge)
- Memory was NOT the issue

---

## v5 Design: Simplified + Floorplanned (Final 2026-01-29)

### V5 Scope (Final)

**Previous V4 failed due to 3-level streaming FSM complexity:**
- Critical path: `hier_16m_level1_idx_chirho_reg` with 32,775 fanout
- 14 logic levels through mux trees
- WNS: -6.256ns at 200MHz

**V5 implements 2-level hierarchies (fully buffered):**
1. **Flat 256-bit** - 256 values (1 HBM beat) ✓
2. **65K (256²)** - 65,536 values (257 beats) ✓
3. **262K (512²)** - 262,144 values (1026 beats) ✓ **with word assembly**

**Removed from V5:**
- 1M (1024²) - Requires 4× word assembly (TODO: V6)
- 16M (256³) - Streaming FSM caused timing failures
- 134M (512³) - Streaming FSM not implemented

**Key V5 addition:** 262K uses 512-bit words assembled from 2× 256-bit HBM beats

### V5 Floorplan (Final)

```
┌─────────────────────────────────────┐
│       SLR0 (near HBM)               │
│  Everything on one SLR:             │
│  - 65K (256²) buffers: 24KB BRAM    │
│  - 262K (512²) buffers: 99KB BRAM   │
│  - Neurosymbolic training           │
│  - Control FSM + Flat domains       │
│  - Soft AND / prob intersection     │
│  (~8% utilization)                  │
├─────────────────────────────────────┤
│       SLR1 + SLR2                   │
│       (empty/reserved)              │
│       Future: 1M (1024²)            │
└─────────────────────────────────────┘
```

**Rationale:** All MCMC workload on SLR0 for minimum HBM latency.
65K + 262K domains + Gumbel-softmax training iterations stay local.

**Word assembly:** 262K (512²) uses 512-bit words. HBM provides 256-bit beats.
FSM assembles: even beat → low 256 bits, odd beat → high 256 bits.

XDC constraints assign cells to pblocks:
- `small_shell_cl_pnr_user.xdc` contains pblock definitions
- `CONTAIN_ROUTING true` isolates SLRs
- AggressiveExplore directives enabled

### Key Code Changes (V5 Revision)

1. **Removed 16M streaming FSM states:**
   - `FSM_LOAD_SUMMARIES_CHIRHO`
   - `FSM_STREAM_LOAD_L2_V1_CHIRHO`
   - `FSM_STREAM_LOAD_L2_V2_CHIRHO`
   - `FSM_STREAM_COMPUTE_CHIRHO`
   - `FSM_STREAM_STORE_CHIRHO`
   - `FSM_STREAM_NEXT_BLOCK_CHIRHO`

2. **Removed large buffer declarations (3-level hierarchies):**
   - `hier_16m_*` (256³ buffers) - streaming FSM complexity
   - `hier_134m_*` (512³ buffers) - not implemented

3. **Added 262K (512²) with word assembly:**
   - 512-bit words assembled from 2× 256-bit HBM beats
   - Beat 0: low half of level0, Beat 1: high half of level0
   - Beats 2-1025: level1 words (even→low, odd→high)
   - 512 parallel ANDs for level1 intersection

4. **FSM enum reduced from 5 to 4 bits**

### Expected Outcome (Final)

| Metric | v4 | v5 (Expected) |
|--------|-----|---------------|
| Logic Levels | 14 | <8 |
| Max Fanout | 32,775 | <1000 |
| Congestion | 7/8 | 3-4/8 |
| Timing Slack | -6.256ns | Met @ 200MHz |
| Build Success | ❌ | ✅ |
| Max Domain Size | 134M (broken) | 262K (working) |
| BRAM Usage | ~200 | ~30 |
| SLR Distribution | SLR0 only | SLR0 only |

---

## PCI Device IDs

| Version | Device ID | Notes |
|---------|-----------|-------|
| v3 | 0xF003 | Basic CL |
| v4 | 0xF004 | Hier + Neurosym |
| v5 | 0xF005 | Floorplanned |

All use Vendor ID 0x1D0F (Amazon) with valid range 0xF000-0xF0FF.

---

## S3 Artifacts

```
s3://minikanren-fpga-chirho/f2_hbm_hdk/
├── build_v4_chirho.log          # V4 build log (failed)
├── build_v4_status_chirho.txt   # V4 status
├── userdata_v4_chirho.log       # V4 userdata script output
├── design_v4_hier_ns_chirho.tar.gz  # V4 design tarball
└── dcp_v4_hier_ns/              # V4 DCP (if any)
```

---

*Soli Deo Gloria* ☧
