# AWS F2 HDK Build Log ☧

For God so loved the world - John 3:16

---

## V5.4 BUILD COMPLETE (2026-01-30) - DEBUG REGISTERS + WIDTH FIX ☧

**Result:** ⚠️ BUILD WITH TIMING WARNING

### Build Details
- **Instance:** i-08cca702b67d12081 (c5.9xlarge) at 98.87.148.68
- **Duration:** 58 minutes
- **Clock:** 200MHz (A1 recipe)
- **Device ID:** 0xF054
- **Version:** 0xF2540001
- **DCP:** `s3://minikanren-fpga-chirho/f2_hbm_hdk/dcp_v5_floorplan/2026_01_30-150751.Developer_CL.tar`
- **AFI:** `afi-07fd1ddc4e2c1615c` / `agfi-0d5a0236dacd50a9e` (pending)

### Timing Warning
```
CRITICAL WARNING: Found {WRAPPER/CL/HBM_ENGINE.sparse_idx_chirho_reg[0]/C -->
                        WRAPPER/CL/HBM_ENGINE.sparse_idx_chirho_reg[1]/D}
                  negative setup slack paths
```
The sparse_idx register has a timing violation. May affect functionality - needs testing.

### Changes from V5.3
1. **Register map fix**: HIER/TRAIN registers moved to 0x80+ to avoid overlap with RESP (0x20-0x5C)
2. **Width bug fix**: `find_next_set_bit_chirho` uses `<=` with inclusive bounds (9'd255, 9'd511)
3. **Debug registers**: Added read-only registers at 0xC0-0xD4:
   - 0xC0: FSM_STATE (current FSM state, 4 bits)
   - 0xC4: AXI_STATUS {bready,bvalid,awvalid,awready,rready,rvalid,arvalid,arready}
   - 0xC8: AXI_ADDR_LO
   - 0xCC: AXI_ADDR_HI
   - 0xD0: BEAT_COUNT
   - 0xD4: SPARSE_IDX
4. **Version update**: 0xF2540001
5. **Forward declarations**: Moved FSM typedef + debug signals earlier for OCL visibility

### V5.4 Test Results (2026-01-30)

**AFI Status:** ✅ Available and loaded on F2 (52.54.165.179)

**Verified Working:**
- VERSION register returns 0xF2540001 ✓
- Debug registers readable at 0xC0-0xD4 ✓
- FSM responds to CONTROL register ✓
- FSM executes through states: IDLE → LOAD_VAR1 → LOAD_VAR2 → COMPUTE → STORE_RESULT → BATCH_NEXT → IDLE ✓
- Address calculation working: var_id=1 maps to HBM_BASE + 0x20 = 0x20000020 ✓

**Issues Found:**
1. **STATUS register hardcoded** (line 583): `done=1, valid=1` always - doesn't reflect actual state
2. **FSM auto-restarts**: After reaching IDLE, if CONTROL=0x05 still set, operation restarts
3. **Command encoding was wrong**: Initial tests used wrong field layout (fixed below)

**Fixes Applied:**
1. Command encoding corrected (see "Command Format" section below)
2. Register offset map clarified (OCL uses 6-bit word addresses)

**Next Steps for V5.5:**
1. Fix STATUS register to show actual `op_done_chirho` state
2. Consider adding "one-shot" mode that auto-clears enable after completion
3. Pipeline sparse_idx to fix timing violation

---

## V5.4 Register Interface Reference ☧

**CRITICAL:** The FSM requires BOTH `ctrl_enable_chirho` AND `ctrl_hbm_mode_chirho` to start!

### OCL Register Map (AXI-Lite, BAR0)

OCL uses 6-bit word addresses. Byte address = word_addr × 4.

| Byte Addr | Word | R/W | Name | Description |
|-----------|------|-----|------|-------------|
| 0x00 | 6'h00 | R | VERSION | 0xF2540001 for V5.4 |
| 0x04 | 6'h01 | R/W | CONTROL | bit0=enable, bit1=reset, bit2=hbm_mode |
| 0x08 | 6'h02 | R | STATUS | bit0=done, bit1=valid, bit2=hbm_ready |
| 0x0C | 6'h03 | R/W | MODE | Operation mode flags |
| 0x10 | 6'h04 | R/W | CMD_LO | {var_id_1[15:0], opcode[3:0]} |
| 0x14 | 6'h05 | R/W | CMD_MID | {batch_count[15:0], var_id_2[15:0]} |
| 0x18 | 6'h06 | R/W | CMD_HI | {unused[25:0], batch_count[17:16]} |
| 0x20-0x5C | 6'h08-17 | R | RESP[0-15] | Response data (512-bit + flags) |
| 0x80 | 6'h20 | R/W | HIER_MODE | Hierarchy mode (0=flat256, 1=65k, 2=262k) |
| 0xC0 | 6'h30 | R | FSM_STATE | Current FSM state (4 bits) |
| 0xC4 | 6'h31 | R | AXI_STATUS | {bready,bvalid,awvalid,awready,rready,rvalid,arvalid,arready} |
| 0xC8 | 6'h32 | R | AXI_ADDR_LO | Current HBM address [31:0] |
| 0xCC | 6'h33 | R | AXI_ADDR_HI | Current HBM address [33:32] |
| 0xD0 | 6'h34 | R | BEAT_COUNT | AXI beat counter |
| 0xD4 | 6'h35 | R | SPARSE_IDX | Current sparse level1 index |

### CONTROL Register (0x04)

```
Bit 0: ctrl_enable_chirho    - Enable HBM engine
Bit 1: ctrl_reset_chirho     - Reset FSM
Bit 2: ctrl_hbm_mode_chirho  - HBM operation mode (REQUIRED for FSM to start!)
```

**⚠️ IMPORTANT:** To start an HBM operation, you MUST write:
```python
CONTROL = 0x05  # enable (bit 0) + hbm_mode (bit 2) = 0b0101
```

Writing only `0x01` (enable) will NOT start the FSM!

### FSM Start Condition (from line 832)

```systemverilog
if (ctrl_enable_chirho && ctrl_hbm_mode_chirho && hbm_ready_chirho) begin
    // FSM starts here
end
```

All three conditions must be true:
1. `ctrl_enable_chirho` = CONTROL[0] = 1
2. `ctrl_hbm_mode_chirho` = CONTROL[2] = 1
3. `hbm_ready_chirho` = STATUS[2] = 1 (HBM IP ready)

### Command Format

**⚠️ IMPORTANT:** The cmd_reg field layout spans register boundaries!

```
cmd_reg_chirho bit layout (70 bits total):
  [3:0]   = opcode (4 bits)
  [19:4]  = var_id_1 (16 bits)
  [35:20] = var_id_2 (16 bits) - SPANS CMD_LO[31:20] and CMD_MID[3:0]
  [51:36] = batch_count (16 bits)
  [69:52] = reserved

Register writes:
  CMD_LO (0x10)  -> cmd_reg[31:0]
  CMD_MID (0x14) -> cmd_reg[63:32]
  CMD_HI (0x18)  -> cmd_reg[69:64]
```

**Correct encoding for var_id_1=0, var_id_2=1, batch=1, op=INTERSECT:**

```python
def encode_command(opcode, var_id_1, var_id_2, batch_count):
    # var_id_2 spans CMD_LO[31:20] and CMD_MID[3:0]
    var_id_2_lo = var_id_2 & 0xFFF       # bits [11:0] -> CMD_LO[31:20]
    var_id_2_hi = (var_id_2 >> 12) & 0xF # bits [15:12] -> CMD_MID[3:0]

    cmd_lo = (var_id_2_lo << 20) | (var_id_1 << 4) | opcode
    cmd_mid = (batch_count << 4) | var_id_2_hi
    return cmd_lo, cmd_mid

# Example: var1=0, var2=1, batch=1, op=1
# cmd_lo  = 0x00100001
# cmd_mid = 0x00000010
```

**Address mapping:**
```
var_id -> HBM_BASE (0x2_0000_0000) + var_id * bytes_per_var

Mode         | bytes_per_var
-------------|---------------
FLAT256 (0)  | 32 bytes
HIER_65K (1) | 8,224 bytes
HIER_262K (2)| 33,024 bytes
```

### PCI Device Path on F2

```bash
# Find FPGA device
lspci | grep Amazon  # Should show device ID 0xf054

# BAR0 path (64MB OCL)
/sys/bus/pci/devices/0000:34:00.0/resource0

# BAR4 path (128GB HBM - if needed for direct access)
/sys/bus/pci/devices/0000:34:00.0/resource4
```

### Example Test Script

```python
import mmap, struct, os, time

BAR = "/sys/bus/pci/devices/0000:34:00.0/resource0"
fd = os.open(BAR, os.O_RDWR | os.O_SYNC)
mm = mmap.mmap(fd, 64*1024*1024)

def read_reg(off):
    mm.seek(off); return struct.unpack('<I', mm.read(4))[0]

def write_reg(off, val):
    mm.seek(off); mm.write(struct.pack('<I', val))

def encode_cmd(op, v1, v2, batch):
    v2_lo = v2 & 0xFFF
    v2_hi = (v2 >> 12) & 0xF
    cmd_lo = (v2_lo << 20) | (v1 << 4) | op
    cmd_mid = (batch << 4) | v2_hi
    return cmd_lo, cmd_mid

# Verify V5.4
assert read_reg(0x00) == 0xF2540001, "Not V5.4!"

# Setup command: intersect var0 and var1, batch=1
write_reg(0x04, 0x00)  # Clear CONTROL
write_reg(0x80, 0x00)  # HIER_MODE = FLAT256
cmd_lo, cmd_mid = encode_cmd(1, 0, 1, 1)  # op=INTERSECT, v1=0, v2=1, batch=1
write_reg(0x10, cmd_lo)
write_reg(0x14, cmd_mid)

# Start with BOTH enable and hbm_mode!
write_reg(0x04, 0x05)  # CONTROL = 0x05

# Poll FSM until IDLE (FSM=0)
for i in range(100):
    time.sleep(0.002)
    fsm = read_reg(0xC0)
    if fsm == 0:
        write_reg(0x04, 0x00)  # Clear CONTROL to stop auto-restart
        print(f"Completed in {i*2}ms")
        break
    print(f"FSM={fsm} AXI=0x{read_reg(0xC4):02X} ADDR=0x{read_reg(0xC8):08X}")

mm.close()
```

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
