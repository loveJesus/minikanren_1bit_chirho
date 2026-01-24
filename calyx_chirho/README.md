# Calyx Hardware Implementation ☧

> *"Whether therefore ye eat, or drink, or whatsoever ye do, do all to the glory of God."*
> — 1 Corinthians 10:31

## Overview

[Calyx](https://calyxir.org/) is an intermediate language for building compilers that generate hardware accelerators. This directory contains Calyx IR (.futil) files implementing miniKanren's core operations for FPGA synthesis.

## Files

| File | Description |
|------|-------------|
| `domain_chirho.futil` | 64-bit domain registers, unification (AND), disjunction (OR) |
| `cam_chirho.futil` | Content-Addressable Memory for relation lookup |
| `search_engine_chirho.futil` | Complete search engine with branching and backtracking |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Search Engine                          │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐   │
│  │  Domain Registers (N × 64-bit)                   │   │
│  │  var0: [████████████████████████████████████████]│   │
│  │  var1: [████░░░░████░░░░████░░░░████░░░░████░░░░]│   │
│  │  var2: [██████████████████░░░░░░░░░░░░░░░░░░░░░░]│   │
│  └─────────────────────────────────────────────────┘   │
│                         │                                │
│  ┌──────────────────────▼──────────────────────────┐   │
│  │  Parallel Unification Unit                       │   │
│  │  domain1 AND domain2 → result (single cycle)    │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                                │
│  ┌──────────────────────▼──────────────────────────┐   │
│  │  Branch Unit (Fork)                              │   │
│  │  Split on lowest bit: x & (-x), x & (x-1)       │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                                │
│  ┌──────────────────────▼──────────────────────────┐   │
│  │  CAM (Content-Addressable Memory)                │   │
│  │  Parallel relation lookup in single cycle        │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                                │
│  ┌──────────────────────▼──────────────────────────┐   │
│  │  Backtrack Stack                                 │   │
│  │  Save/restore alternative branches               │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Key Hardware Operations

### Unification (Single Cycle)
```
domain1:  0b11110000  // {4,5,6,7}
domain2:  0b00111100  // {2,3,4,5}
result:   0b00110000  // {4,5} = domain1 AND domain2
```

### Branching (Fork)
```
domain:       0b11110000  // {4,5,6,7}
lowest_bit:   0b00010000  // {4} = x & (-x)
rest:         0b11100000  // {5,6,7} = x & (x-1)
```

### Failure Detection
```
if (result == 0) {
    // Empty domain = no possible values = failure
    backtrack();
}
```

## Building

Prerequisites:
- [Calyx](https://calyxir.org/) toolchain
- [Verilator](https://www.veripool.org/verilator/) (for simulation)
- [Yosys](https://yosyshq.net/yosys/) + [nextpnr](https://github.com/YosysHQ/nextpnr) (for FPGA synthesis)

```bash
# Install Calyx
cargo install calyx

# Compile to Verilog
calyx domain_chirho.futil -b verilog > domain_chirho.v

# Simulate with Verilator
calyx domain_chirho.futil -b verilog-refmem | verilator --binary -

# Synthesize for iCE40 FPGA
yosys -p "read_verilog domain_chirho.v; synth_ice40 -top main -json out.json"
nextpnr-ice40 --hx8k --json out.json --asc out.asc
```

## Performance Targets

| Operation | Cycles | Notes |
|-----------|--------|-------|
| Unify two vars | 1 | Parallel 64-bit AND |
| Check singleton | 1 | x & (x-1) == 0 |
| Branch (fork) | 2 | Compute both alternatives |
| CAM lookup | 1 | All entries compared in parallel |
| Backtrack | 2 | Pop stack, restore state |

## Resource Estimates (iCE40 HX8K)

| Component | LUTs | FFs | BRAMs |
|-----------|------|-----|-------|
| 8-var state | ~520 | 520 | 0 |
| Unify unit | ~130 | 130 | 0 |
| 16-entry CAM | ~800 | 560 | 0 |
| Branch unit | ~200 | 200 | 0 |
| Stack (16 deep) | ~300 | 1100 | 1 |
| **Total** | ~1950 | 2510 | 1 |

iCE40 HX8K has 7680 LUTs, so this fits comfortably with room for expansion.

## Future Work

1. **Pipelining**: Overlap unification with branching
2. **Multiple search states**: Parallel exploration of search tree
3. **Larger CAM**: More relation entries for complex queries
4. **Memory interface**: External SRAM for larger domains/relations

---

*Soli Deo Gloria* ☧
