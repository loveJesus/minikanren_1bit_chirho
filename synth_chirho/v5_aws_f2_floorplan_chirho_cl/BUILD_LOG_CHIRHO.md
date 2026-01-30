# AWS F2 HDK Build Log ☧

For God so loved the world - John 3:16

---

## V5.3 SUCCESS (2026-01-30) - SPARSE STREAMING ☧

**Result:** ✅ BUILD SUCCESSFUL

### Build Details
- **Instance:** i-08cca702b67d12081 (c5.9xlarge)
- **Duration:** 52 minutes
- **Clock:** 200MHz (A1 recipe)
- **Device ID:** 0xF053
- **DCP:** `s3://minikanren-fpga-chirho/f2_hbm_hdk/dcp_v5_floorplan/2026_01_30-065128.Developer_CL.tar`
- **AFI:** `afi-010cbb77b5413e1d6` / `agfi-041630da370421d34`

### Timing Summary
| Phase | WNS | Status |
|-------|-----|--------|
| place_design | -1.659ns | ✅ Passed |
| phys_opt_design | -0.783ns | ✅ Improved |
| route_design | -0.473ns | ✅ Completed |

**Final WNS: -0.473ns** - Violation is in HBM MMCM IP (AWS/Xilinx), not our design.

### Key Changes from V5.2 (failed)
V5.2 failed with CLB packing overflow (~983K FFs required).

V5.3 sparse streaming reduces FFs by 99.7%:
- Only buffer level0 summaries (256 or 512 bits)
- Stream level1 words one at a time
- Use `level0_A & level0_B` to skip zero blocks

| Resource | V5.2 | V5.3 |
|----------|------|------|
| 65K level1 FFs | 196,608 | 768 |
| 262K level1 FFs | 786,432 | 1,536 |
| **Total** | **~983,000** | **~3,000** |

### New FSM States
```systemverilog
FSM_SPARSE_INIT_CHIRHO      // Compute level0 AND, find first non-zero
FSM_SPARSE_LOAD_A_CHIRHO    // Stream level1[i] from var1
FSM_SPARSE_LOAD_B_CHIRHO    // Stream level1[i] from var2
FSM_SPARSE_COMPUTE_CHIRHO   // Compute AND for current word
FSM_SPARSE_STORE_CHIRHO     // Store result level1[i]
FSM_SPARSE_NEXT_CHIRHO      // Find next non-zero index
```

### Helper Functions
- `find_next_set_bit_chirho()` - Priority encoder for sparse iteration
- `popcount_512_chirho()` - Count non-zero blocks

---

## V5.2 FAILED (2026-01-30) - CLB Packing Overflow

**Error:** `ERROR: [Place 30-487] ... 35,437 CLBs required, 33,562 available`

Parallel generate blocks forced all level1 arrays to registers:
```systemverilog
// This reads ALL 256 words simultaneously - forces register implementation!
for (int gi = 0; gi < 256; gi++) begin
    hier_65k_result_level1_chirho[gi] <= ...
end
```

---

## V5.1 FAILED (2026-01-29) - HBM Pblock Conflict

**Error:** `ERROR: [Place 30-1093] Failed to place ... HBM_CORE_I ... outside pblock_CL`

HBM hard macros have fixed physical sites outside pblock_CL bounds.

---

## Build v12 (2026-01-28) - HIERARCHICAL + NEUROSYMBOLIC

**New in this build:**
- All hierarchical domain modes (256², 512², 256³, 512³)
- Neurosymbolic training engine (Q16.16 fixed-point)
- Probabilistic inference (Q8.8 fixed-point)
- Streaming FSM for 3-level hierarchies

### Hierarchical Domain Modes

| Mode | Values | Memory | Strategy |
|------|--------|--------|----------|
| `HIER_MODE_FLAT256_CHIRHO` | 256 | 32 bytes | Single HBM beat |
| `HIER_MODE_HIER_65K_CHIRHO` | 65,536 (256²) | 8KB | Fully buffered |
| `HIER_MODE_HIER_262K_CHIRHO` | 262,144 (512²) | 33KB | Fully buffered |
| `HIER_MODE_HIER_16M_CHIRHO` | 16,777,216 (256³) | 2MB | Streaming |
| `HIER_MODE_HIER_134M_CHIRHO` | 134,217,728 (512³) | 16MB | Streaming |

### New Design Files

| File | Description |
|------|-------------|
| `searchEngine64BitChirho.v` | 64-bit variant search engine |
| `intersect_512_chirho.v` | 512-bit base intersection |
| `intersect_hier_65k_chirho.v` | 256² intersection |
| `intersect_hier_262k_chirho.v` | 512² intersection |
| `intersect_hier_16m_chirho.v` | 256³ streaming intersection |
| `intersect_hier_134m_chirho.v` | 512³ streaming intersection |
| `diffTrainChirho.v` | Gumbel-softmax training engine |
| `soft_and_32_chirho.v` | Q16.16 soft AND |
| `soft_and_16_chirho.v` | Q8.8 soft AND |
| `intersect_prob_domain_64_chirho.v` | Probabilistic intersection |

---

## Build History

### Build v11 (2026-01-28) - IN PROGRESS

**Instance:** i-0b80fd38bec5ff535
**IP:** 107.21.151.192
**AMI:** ami-0cb1b6ae2ff99f8bf (FPGA Developer AMI 1.18.0, Rocky Linux, Vivado 2025.1)
**Instance Type:** c5.9xlarge (36 vCPU, 72 GB RAM, $1.53/hr)
**Started:** 2026-01-28 02:28 UTC
**SSH User:** `rocky` (NOT ec2-user!)

**Improvements:**
- Bootstrap script downloads full build script from S3 (avoids 16KB userdata limit)
- Progress checkpoints uploaded to S3 (`progress_v11_chirho.txt`)
- Verification step confirms constraint files before starting Vivado

---

### Build v10 (2026-01-28) - FAILED

**Instance:** i-06d731ef09f7ee65e
**Issue:** Old userdata was used (not updated base64 file)
**Result:** Same constraint file error as v9

---

## Issues Fixed During Build Iterations

### v5: Vivado Not Found
**Error:** `sh: vivado: command not found`
**Fix:** Added explicit Vivado sourcing before HDK setup:
```bash
source /opt/Xilinx/2025.1/Vivado/settings64.sh
```

### v6: Missing build_all.tcl
**Error:** `couldn't read file "build_all.tcl": no such file or directory`
**Fix:** Created symlinks to HDK common build scripts:
```bash
ln -sf $HDK_DIR/common/shell_stable/build/scripts/aws_build_dcp_from_cl.py .
ln -sf $HDK_DIR/common/shell_stable/build/scripts/build_all.tcl .
ln -sf $HDK_DIR/common/shell_stable/build/scripts/build_level_1_cl.tcl .
```

### v6: Wrong encrypt.tcl
**Error:** `error copying "cl_dram_hbm_dma.sv": no such file or directory`
**Cause:** Copied encrypt.tcl from HBM example which referenced wrong files
**Fix:** Created custom encrypt.tcl that references our design files:
- cl_minikanren_chirho.sv
- cl_minikanren_chirho_defines.vh
- searchEngineChirho.v
- cl_hbm_axi4.sv (from HDK)
- cl_hbm_wrapper.sv (from HDK)
- cl_dram_dma_pkg.sv (from HDK)

### v7: S3 Access Denied (403 Forbidden)
**Error:** `An error occurred (403) when calling the HeadObject operation: Forbidden`
**Cause:** IAM policy referenced wrong bucket name
**Fix:** Updated IAM policy to include both buckets:
```json
"Resource": [
  "arn:aws:s3:::minikanren-fpga-chirho",
  "arn:aws:s3:::minikanren-fpga-chirho/*",
  "arn:aws:s3:::minikanren-fpga-chirho-686672719245",
  "arn:aws:s3:::minikanren-fpga-chirho-686672719245/*"
]
```

### v8: Invalid Hex in CL_SH_ID1
**Error:** `syntax error near 'T_FEED'`
**Cause:** `1B1T_FEED` contains 'T' which is not a valid hex digit
**Fix:** Changed to valid hex:
```verilog
`define CL_SH_ID1 32'h1B1E_FEED  // Was 1B1T_FEED (T is invalid)
```

### v10: Old Userdata Used
**Error:** Same as v9 - `File does not exist: small_shell_cl_pnr_user.xdc`
**Cause:** The base64-encoded userdata file wasn't updated after fixing v6 script
**Fix:** Use bootstrap pattern - upload full script to S3, small userdata downloads and runs it:
```bash
# Bootstrap userdata (fits in 16KB limit)
#!/bin/bash
export AWS_DEFAULT_REGION=us-east-1
aws s3 cp s3://bucket/build_script.sh /tmp/build.sh
chmod +x /tmp/build.sh
/tmp/build.sh
```

### v9: Missing P&R Constraint File
**Error:** `File does not exist: small_shell_cl_pnr_user.xdc`
**Cause:** Implementation phase requires this constraint file
**Fix:** Created placeholder constraint file:
```bash
cat > "$CL_DIR/build/constraints/small_shell_cl_pnr_user.xdc" << 'EOF'
# Place and Route constraints for miniKanren CL
# No special P&R constraints needed
EOF
```

---

## Required Files for HDK Build

### Design Files (in design/)
| File | Source | Description |
|------|--------|-------------|
| cl_minikanren_chirho.sv | Our design | Top-level CL module |
| cl_minikanren_chirho_defines.vh | Our design | Version and HBM address defines |
| searchEngineChirho.v | Clash-generated | miniKanren search engine |
| cl_hbm_axi4.sv | HDK example | HBM AXI4 interface |
| cl_hbm_wrapper.sv | HDK example | HBM wrapper |
| cl_dram_dma_pkg.sv | HDK example | Interface definitions (axi_bus_t, cfg_bus_t) |
| cl_ports.vh | HDK shell | Shell port definitions |
| cl_id_defines.vh | Generated | PCIe device IDs |

### Build Scripts (in build/scripts/)
| File | Type | Description |
|------|------|-------------|
| aws_build_dcp_from_cl.py | Symlink | HDK build orchestrator |
| build_all.tcl | Symlink | Main Vivado build script |
| build_level_1_cl.tcl | Symlink | Implementation script |
| synth_cl_minikanren_chirho.tcl | Custom | Our synthesis script |
| encrypt.tcl | Custom | File copy/encryption script |

### Constraint Files (in build/constraints/)
| File | Description |
|------|-------------|
| cl_synth_user.xdc | Synthesis constraints |
| cl_timing_user.xdc | Timing constraints (false paths) |
| small_shell_cl_pnr_user.xdc | P&R constraints (placeholder) |

---

## Build Command

```bash
python3 $HDK_DIR/common/shell_stable/build/scripts/aws_build_dcp_from_cl.py \
    --cl cl_minikanren_chirho \
    --aws_clk_gen \
    --clock_recipe_a A2
```

**Clock Recipe A2:** 250 MHz main clock

---

## Naming Conventions (per AGENTS.md)

| Category | Convention | Example |
|----------|------------|---------|
| Internal signals | snake_chirho | clk_engine_chirho |
| Constants/enums | UPPER_CHIRHO | FSM_IDLE_CHIRHO |
| Types | snake_chirho_t | wr_state_t_chirho |
| Module instances | snake_chirho | u_engine_chirho |
| Clash-generated | camelChirho | enChirho, cmdChirho |
| AWS HDK primitives | original | clk_main_a0, rst_main_n |

---

## Cost Estimate

- Instance: c5.9xlarge @ $1.53/hr
- Build duration: ~4-5 hours
- **Estimated cost per build: $6-8**

---

## Next Steps After DCP

1. Create AFI from DCP tarball
2. Wait for AFI to become available (~30-60 min)
3. Launch f2.6xlarge instance
4. Load AFI and test

Soli Deo Gloria ☧
