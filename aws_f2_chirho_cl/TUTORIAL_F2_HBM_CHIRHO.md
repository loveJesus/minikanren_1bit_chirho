# F2 HBM Integration Tutorial ☧

## Overview

This tutorial documents the AWS F2 FPGA HBM integration for miniKanren 1-bit domain operations.

**Goal:** Store variable domains in HBM (High Bandwidth Memory) instead of PCIe registers to eliminate the 843 cycles/operation bottleneck.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     AWS F2 Shell                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                  cl_minikanren_chirho                      │  │
│  │                                                            │  │
│  │   ┌──────────────┐    ┌──────────────┐    ┌────────────┐  │  │
│  │   │  PCIe/OCL    │───▶│   HBM FSM    │───▶│   HBM IP   │  │  │
│  │   │  Interface   │    │  Controller  │    │  (8GB)     │  │  │
│  │   └──────────────┘    └──────────────┘    └────────────┘  │  │
│  │          │                   │                   │         │  │
│  │          ▼                   ▼                   ▼         │  │
│  │   ┌──────────────┐    ┌──────────────┐    ┌────────────┐  │  │
│  │   │   Control    │    │   Domain     │    │   256-bit  │  │  │
│  │   │   Registers  │    │   Operations │    │   AXI Bus  │  │  │
│  │   └──────────────┘    └──────────────┘    └────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## HBM Memory Layout

| Region | Base Address | Size | Purpose |
|--------|--------------|------|---------|
| Variable Headers | `0x0_0000_0000` | 512 MB | Metadata per variable |
| Variable Domains | `0x0_2000_0000` | 8 GB | 256-bit domain bitvectors |
| Term Store | `0x1_0000_0000` | 4 GB | Hash-consed terms |
| Hash Table | `0x2_0000_0000` | 256 MB | Term lookup table |
| Tabling Cache | `0x2_1000_0000` | 2 GB | Memoization cache |

---

## HBM FSM States

```
IDLE_CHIRHO ──▶ LOAD_VAR1_CHIRHO ──▶ LOAD_VAR2_CHIRHO ──▶ COMPUTE_CHIRHO ──▶ STORE_RESULT_CHIRHO
     ▲                                                                              │
     └──────────────────────── BATCH_NEXT_CHIRHO ◀──────────────────────────────────┘
```

### State Descriptions

1. **IDLE_CHIRHO**: Wait for operation request via control register
2. **LOAD_VAR1_CHIRHO**: Issue AXI read for first variable's domain
3. **LOAD_VAR2_CHIRHO**: Issue AXI read for second variable's domain
4. **COMPUTE_CHIRHO**: Perform domain operation (AND/OR/NOT/IS_GROUND)
5. **STORE_RESULT_CHIRHO**: Write result back to HBM
6. **BATCH_NEXT_CHIRHO**: Increment to next operation in batch

---

## Domain Operations

| Op Code | Name | Operation | miniKanren Equivalent |
|---------|------|-----------|----------------------|
| `0x0` | INTERSECT_CHIRHO | `D1 & D2` | Unification (==) |
| `0x1` | UNION_CHIRHO | `D1 \| D2` | Disjunction (conde) |
| `0x2` | COMPLEMENT_CHIRHO | `~D1` | Negation (not) |
| `0x3` | IS_GROUND_CHIRHO | `popcount(D)==1` | Ground check |

---

## Files Structure

```
aws_f2_chirho_cl/
├── design/
│   ├── cl_minikanren_chirho.sv      # Main shell with HBM FSM
│   ├── cl_minikanren_chirho_defines.vh  # Memory layout defines
│   ├── cl_id_defines.vh             # PCIe IDs (0xF216_1D0F)
│   ├── cl_dram_dma_defines.vh       # AXI defaults
│   ├── cl_hbm_axi4.sv               # AXI4→AXI3 conversion
│   ├── cl_hbm_wrapper.sv            # HBM IP wrapper
│   ├── searchEngineChirho.v         # Clash-generated search engine
│   └── stubs_chirho.sv              # Simulation stubs
└── build/
    └── scripts/
        └── synth_cl_minikanren_chirho.tcl  # Vivado synthesis
```

---

## RTL Verification Status

### Verilator Lint Check ✅

```bash
cd aws_f2_chirho_cl/design
verilator --lint-only --top-module cl_minikanren_chirho \
  -I. \
  -I$HDK_DIR/common/shell_stable/design/interfaces \
  -I$HDK_DIR/common/lib \
  $HDK_DIR/common/lib/interfaces.sv \
  $HDK_DIR/cl/examples/cl_dram_hbm_dma/design/cl_dram_dma_pkg.sv \
  cl_minikanren_chirho.sv cl_hbm_axi4.sv cl_hbm_wrapper.sv
```

**Result:** Syntax verified. Missing modules are Xilinx IP (available in Vivado).

---

## AWS Synthesis Launch

### Instance Selection

| Instance | vCPUs | Memory | Cost/hr | Use Case |
|----------|-------|--------|---------|----------|
| c5.4xlarge | 16 | 32 GB | $0.68 | Small designs |
| **c5.9xlarge** | 36 | **72 GB** | $1.53 | **HBM designs (recommended)** |
| c5.12xlarge | 48 | 96 GB | $2.04 | Large designs |

### Launch Command

```bash
# From aws_f2_chirho_cl/
./launch_synth_f2_chirho.sh
```

### Expected Timeline

1. Instance launch: ~2 minutes
2. Design upload: ~1 minute
3. Synthesis: ~2-3 hours
4. Place & Route: ~1-2 hours
5. Total: **3-5 hours**

---

## Control Register Map

| Offset | Name | Bits | Description |
|--------|------|------|-------------|
| 0x00 | CONTROL_CHIRHO | [0] | Start operation |
| | | [1] | Batch mode enable |
| | | [2] | HBM mode (vs register mode) |
| | | [7:4] | Operation code |
| 0x04 | STATUS_CHIRHO | [0] | Busy |
| | | [1] | Done |
| | | [2] | Error |
| | | [3] | HBM ready |
| 0x08 | VAR1_ID_CHIRHO | [15:0] | First variable ID |
| 0x0C | VAR2_ID_CHIRHO | [15:0] | Second variable ID |
| 0x10 | RESULT_ID_CHIRHO | [15:0] | Result variable ID |
| 0x14 | BATCH_COUNT_CHIRHO | [31:0] | Number of operations |

---

## Performance Targets

| Metric | PCIe Mode | HBM Mode | Improvement |
|--------|-----------|----------|-------------|
| Latency/op | 843 cycles | ~10 cycles | **84x** |
| Throughput | 297K ops/s | 25M ops/s | **84x** |
| Bandwidth | 2.4 GB/s | 460 GB/s | **192x** |

---

## Next Steps

1. ☐ Launch c5.9xlarge synthesis
2. ☐ Verify timing at 250 MHz
3. ☐ Create AFI (Amazon FPGA Image)
4. ☐ Test on f2.2xlarge instance
5. ☐ Benchmark HBM vs PCIe performance

---

*Soli Deo Gloria* ☧
