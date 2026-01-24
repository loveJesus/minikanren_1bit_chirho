# Practical Benchmarks ☧

> *"Whether therefore ye eat, or drink, or whatsoever ye do,
>  do all to the glory of God."* — 1 Corinthians 10:31

## Overview

These benchmarks test miniKanren's 1-bit matrix operations at realistic scales,
targeting workloads similar to production AI reasoning systems.

## Benchmarks

| Benchmark | Domain | Variables | Constraints | Target |
|-----------|--------|-----------|-------------|--------|
| `synth_chirho.py` | Program synthesis | 50-500 | 100-1000 | Generate code from specs |
| `type_scale_chirho.py` | Type inference | 1000+ | 5000+ | Typecheck real codebases |
| `graph_reason_chirho.py` | Knowledge graphs | 10K+ nodes | Path queries | Graph traversal at scale |
| `schedule_chirho.py` | Scheduling | 100+ tasks | Temporal constraints | Resource allocation |

## Running

```bash
cd benchmarks_chirho
source ../venv_chirho/bin/activate

# Quick smoke test
python3 synth_chirho.py --size small

# Full benchmark
python3 synth_chirho.py --size large --profile
```

## Metrics

Each benchmark reports:
- **Solutions found**: Count and first N solutions
- **Time**: Wall clock and per-operation breakdown
- **Memory**: Peak usage and domain sizes
- **Branches**: Search tree statistics
- **Propagation**: Constraint propagation rounds

## Hardware Projection

For each benchmark, we project FPGA performance based on:
- Domain operations → AND gates (1 cycle)
- Branching → Stack operations (2 cycles)
- Propagation → Iterative refinement (N cycles)

---

*Soli Deo Gloria* ☧
