# Practical Examples ☧

> *"Whether therefore ye eat, or drink, or whatsoever ye do, 
>  do all to the glory of God."* — 1 Corinthians 10:31

## Overview

These examples demonstrate practical applications of miniKanren's 
1-bit matrix operations for real constraint solving problems.

## Examples

### 1. Sudoku Solver (`sudoku_chirho.py`)

Solves Sudoku puzzles using bit-parallel domain propagation.

**Key techniques:**
- 9-bit domains (each bit = one possible value 1-9)
- Constraint propagation via bitwise AND
- Fail-first branching heuristic
- No hash consing needed (finite domain)

```bash
python3 sudoku_chirho.py
```

**Performance:**
- Easy puzzles: 0.1-0.2ms (pure propagation, no search)
- Hard puzzles: 1-5ms (propagation + backtracking)

### 2. Type Inference (`type_infer_chirho.py`)

Hindley-Milner style type inference for a lambda calculus.

**Key techniques:**
- Hash-consed type terms (structural sharing)
- Unification for type equations
- Occurs check prevents infinite types
- Substitution chains for variable bindings

```bash
python3 type_infer_chirho.py
```

**Features:**
- Infers polymorphic types (`'t0 -> 't0` for identity)
- Handles higher-order functions
- Detects type errors

## How This Relates to the Main Project

| Example | Domain Size | Hash Consing | Unification |
|---------|-------------|--------------|-------------|
| Sudoku | 9 values (9-bit) | No | Bitwise AND |
| Type Inference | Infinite (types) | Yes | Structural |

Both use the same core insight:
- **Constraints narrow possibilities** (domain shrinking)
- **Unification = intersection** (AND for bits, structural match for terms)
- **Failure = empty domain** (0 bits set, or unification fails)

## Running on FPGA

The Sudoku solver's domain operations map directly to hardware:

```
Cell domain:  9-bit register
Unify:        AND gate (1 cycle)
Fork:         x & (-x), x & (x-1) (1 cycle)
Failure:      domain == 0 (zero-detect)
```

A 9×9 Sudoku board = 81 cells × 9 bits = 729 bits of state.
This fits easily in any FPGA with room for parallel constraint propagation.

---

*Soli Deo Gloria* ☧
