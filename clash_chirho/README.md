# Clash Hardware Implementation ☧

> *"Whether therefore ye eat, or drink, or whatsoever ye do, do all to the glory of God."*
> — 1 Corinthians 10:31

## Overview

[Clash](https://clash-lang.org/) is a functional hardware description language that compiles Haskell to Verilog/VHDL. This directory contains a Clash implementation of miniKanren's core operations.

## Why Clash?

| Aspect | Clash | Calyx | Direct Verilog |
|--------|-------|-------|----------------|
| Language | Haskell | Custom IR | Verilog |
| Style | Purely Functional | Compiler IR | Imperative |
| Type Safety | Strong (GHC) | Basic | Weak |
| Abstraction | High | Medium | Low |
| Learning Curve | Steep | Medium | Low |

Clash is ideal for miniKanren because:
- **Pure functions** map directly to combinational logic
- **Algebraic data types** represent hardware states cleanly
- **Pattern matching** synthesizes to efficient muxes
- **Mealy machines** model sequential circuits naturally

## Files

| File | Description |
|------|-------------|
| `MiniKanren_chirho.hs` | Core operations and search engine |
| `minikanren-clash.cabal` | Package definition |

## Key Operations

### Unification (1 cycle)
```haskell
unify_chirho :: Domain_chirho -> Domain_chirho -> Domain_chirho
unify_chirho d1_chirho d2_chirho = d1_chirho .&. d2_chirho
```

### Fork/Branch (1 cycle)
```haskell
fork_chirho :: Domain_chirho -> (Domain_chirho, Domain_chirho)
fork_chirho x_chirho = (lowestBit_chirho x_chirho, clearLowest_chirho x_chirho)
  where
    lowestBit_chirho x = x .&. negate x      -- isolate lowest set bit
    clearLowest_chirho x = x .&. (x - 1)     -- clear lowest set bit
```

### Singleton Check (1 cycle)
```haskell
isSingleton_chirho :: Domain_chirho -> Bool
isSingleton_chirho x_chirho = 
  x_chirho /= 0 && (x_chirho .&. (x_chirho - 1)) == 0
```

## Building

### Prerequisites
```bash
# Install Clash
cabal update
cabal install clash-ghc

# Or with Stack
stack install clash-ghc
```

### Compile to Verilog
```bash
cd clash_chirho
clash --verilog MiniKanren_chirho.hs

# Output in verilog/MiniKanren_chirho/search_engine_chirho.v
```

### Compile to VHDL
```bash
clash --vhdl MiniKanren_chirho.hs

# Output in vhdl/MiniKanren_chirho/search_engine_chirho.vhdl
```

### Simulate
```bash
# Interactive testing
clashi MiniKanren_chirho.hs

# In GHCi:
> let s0 = initState_chirho :: SearchState_chirho 4
> let s1 = unifyVars_chirho 0 1 s0
> domains_chirho s1
```

## Architecture

```
┌─────────────────────────────────────────────────────┐
│              search_engine_chirho                    │
│  ┌───────────────────────────────────────────────┐  │
│  │  SearchCmd_chirho                              │  │
│  │  • Init_chirho                                 │  │
│  │  • Unify_chirho v1 v2                          │  │
│  │  • Constrain_chirho v mask                     │  │
│  │  • Branch_chirho v                             │  │
│  │  • Backtrack_chirho                            │  │
│  └───────────────────────────────────────────────┘  │
│                        │                             │
│                        ▼                             │
│  ┌───────────────────────────────────────────────┐  │
│  │  EngineState_chirho                            │  │
│  │  • domains: Vec 8 (BitVector 64)              │  │
│  │  • valid: Bool                                 │  │
│  │  • stack: Vec 16 StackEntry                   │  │
│  │  • sp: Index 16                                │  │
│  └───────────────────────────────────────────────┘  │
│                        │                             │
│                        ▼                             │
│  ┌───────────────────────────────────────────────┐  │
│  │  SearchResp_chirho                             │  │
│  │  • valid: Bool                                 │  │
│  │  • solution: Bool                              │  │
│  │  • domains: Vec 8 (BitVector 64)              │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Comparison with Calyx

| Feature | Clash | Calyx |
|---------|-------|-------|
| Source | `clash_chirho/` | `calyx_chirho/` |
| Language | Haskell | Calyx IR |
| Abstraction | Types, ADTs, pattern matching | Components, groups, control |
| Testing | GHCi, QuickCheck | Verilator simulation |
| Target | Verilog/VHDL | Verilog |

Both produce synthesizable hardware. Use Clash for functional elegance, Calyx for fine-grained control.

---

*Soli Deo Gloria* ☧
