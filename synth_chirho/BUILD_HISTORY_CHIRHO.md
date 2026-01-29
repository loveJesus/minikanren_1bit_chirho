# Build History - miniKanren FPGA ☧

## Version Summary

| Version | Directory | Target | Status | Notes |
|---------|-----------|--------|--------|-------|
| v0 | `v0_f1_synth_chirho/` | F1 | Exploratory | Early Yosys experiments |
| v1 | `v1_aws_f1_chirho/` | F1 | Incomplete | F1 instances scarce |
| v2 | `v2_aws_f2_chirho/` | F2 | ✅ Working | Basic 64-bit domains |
| v3 | `v3_aws_f2_chirho_cl/` | F2 | ✅ Working | CL wrapper, HBM integration |
| v4 | `v4_aws_f2_hier_ns_chirho_cl/` | F2 | ❌ Failed | Routing congestion |
| v5 | `v5_aws_f2_floorplan_chirho_cl/` | F2 | 🔄 Planned | Floorplanning + 200MHz |

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

## v5 Plan: Floorplanning + 200MHz

### Changes from v4

1. **Clock Frequency:** 250MHz → 200MHz
   - Relaxes timing, allows longer routes around congestion
   - Low-risk change

2. **Floorplanning Constraints:** Spread logic across SLRs
   ```
   ┌─────────────────────────────────┐
   │            SLR2                 │
   │   512³ (134M) - isolated        │
   ├─────────────────────────────────┤
   │            SLR1                 │
   │   64-bit + 512² (262K)          │
   ├─────────────────────────────────┤
   │            SLR0                 │
   │   256² + Neurosym + HBM         │  ← Primary MCMC use case
   └─────────────────────────────────┘
   ```

3. **Simplified Design:** Drop 256³ (16M) for now
   - Reduces routing pressure
   - 512³ covers "millions" use case
   - Can add back in v6 if v5 succeeds

### Rationale for Floorplan

- **SLR0:** 256² (65K) pairs with Neurosymbolic for MCMC workloads
  - Weight storage fits in 256KB BRAM
  - Primary use case per Ed Kmett feedback
  - Close to HBM for weight persistence

- **SLR1:** 64-bit (tiny) + 512² (262K)
  - Pure logic domains without probabilistic weights
  - Medium density, won't cause congestion

- **SLR2:** 512³ (134M) alone
  - Largest module isolated
  - HBM-backed anyway (crosses SLR via AXI)

### Expected Outcome

| Metric | v4 | v5 (Expected) |
|--------|-----|---------------|
| Congestion | 7/8 | 4-5/8 |
| Timing Slack | Failed | Met @ 200MHz |
| Build Success | ❌ | ✅ |

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
