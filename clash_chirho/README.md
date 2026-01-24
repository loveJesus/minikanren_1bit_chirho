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
| `MiniKanrenChirho.hs` | Core operations and search engine |
| `HashConsChirho.hs` | Hardware hash consing for infinite domains |

## Key Operations

### Unification (1 cycle)
```haskell
unifyChirho :: DomainChirho -> DomainChirho -> DomainChirho
unifyChirho d1Chirho d2Chirho = d1Chirho .&. d2Chirho
```

### Fork/Branch (1 cycle)
```haskell
forkChirho :: DomainChirho -> (DomainChirho, DomainChirho)
forkChirho xChirho = (lowestBitChirho xChirho, clearLowestChirho xChirho)
  where
    lowestBitChirho x = x .&. negate x      -- isolate lowest set bit
    clearLowestChirho x = x .&. (x - 1)     -- clear lowest set bit
```

### Singleton Check (1 cycle)
```haskell
isSingletonChirho :: DomainChirho -> Bool
isSingletonChirho xChirho =
  xChirho /= 0 && (xChirho .&. (xChirho - 1)) == 0
```

## Building

### Prerequisites

**GHC Version:** Clash requires GHC 9.2-9.8 (not 9.10+).
Use ghcup to manage versions:

```bash
# Install ghcup
curl --proto '=https' --tlsv1.2 -sSf https://get-ghcup.haskell.org | sh

# Install compatible GHC
ghcup install ghc 9.6.4
ghcup set ghc 9.6.4

# Install Clash
cabal update
cabal install clash-ghc

# Or with Stack
stack install clash-ghc
```

### Compile to Verilog
```bash
cd clash_chirho
clash --verilog MiniKanrenChirho.hs

# Output in verilog/MiniKanrenChirho/searchEngineChirho.v
```

### Compile to VHDL
```bash
clash --vhdl MiniKanrenChirho.hs

# Output in vhdl/MiniKanrenChirho/searchEngineChirho.vhdl
```

### Simulate
```bash
# Interactive testing
clashi MiniKanrenChirho.hs

# In GHCi:
> let s0 = initStateChirho :: SearchStateChirho 4
> let s1 = unifyVarsChirho 0 1 s0
> domainsChirho s1
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
