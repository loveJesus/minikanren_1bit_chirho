# miniKanren F2 FPGA Port

Ported from F1 to F2 for better capacity availability.

## Key Differences from F1

| Aspect | F1 | F2 |
|--------|----|----|
| FPGA | Xilinx UltraScale+ | AMD Virtex UltraScale+ HBM |
| Clocks | 8 available | 2 (clk_main_a0, clk_hbm_ref) |
| Memory | DDR4 | DDR4 + HBM option |
| Price | $1.65/hr | $1.98/hr |
| OCL ports | sh_ocl_*, ocl_sh_* | ocl_cl_*, cl_ocl_* |

## Files

- `design/cl_minikanren_chirho.sv` - F2 shell wrapper
- `design/searchEngineChirho.v` - Core logic (unchanged from F1)
- `design/cl_minikanren_chirho_defines.vh` - Module defines
- `build/scripts/synth_cl_minikanren_chirho.tcl` - Synthesis script

## Build

```bash
# Source F2 HDK
source /path/to/synth_chirho/aws_f2_chirho/hdk_setup.sh

# Run synthesis on F2 build instance
cd build
vivado -mode batch -source scripts/synth_cl_minikanren_chirho.tcl
```

## AFI Creation

After synthesis, create AFI same as F1 but specify the F2 shell.

## ⚠️ PCI Device ID Requirements

**CRITICAL:** AWS F2 requires specific PCI Device ID ranges or AFI creation will fail.

For Amazon Vendor ID `0x1D0F`:
- **Valid range:** `0xF000` - `0xF0FF` only
- **Our ID:** `0xF016` (F0xx valid + tribute to John 3:16 ☧)

**Forbidden IDs:**
- `0x1042` — Reserved for F1
- `0xF200+` — Reserved by AWS shell
- `0x0000` — Invalid

Example `cl_id_defines.vh`:
```verilog
// For God so loved the world - John 3:16 ☧
`define CL_SH_ID0 32'hF016_1D0F  // DeviceID=0xF016, VendorID=0x1D0F
`define CL_SH_ID1 32'h1D51_F016  // SubsystemVID=0x1D51, SubsystemID=0xF016
```

If you see `PCIID_FORBIDDEN: PCI ID value used is reserved`, check your device ID.

*Soli Deo Gloria* ☧
