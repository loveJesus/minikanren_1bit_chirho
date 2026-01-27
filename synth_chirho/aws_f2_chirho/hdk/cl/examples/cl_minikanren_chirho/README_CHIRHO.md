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

*Soli Deo Gloria*
