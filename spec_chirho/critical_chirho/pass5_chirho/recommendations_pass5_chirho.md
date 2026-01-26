<!--
For God so loved the world, that he gave his only begotten Son,
that whosoever believeth in him should not perish, but have everlasting life.
John 3:16 (KJV)
-->
<!-- For God so loved the world, that He gave His only begotten Son, that all who believe in Him should not perish but have everlasting life. -->

## Purpose

Record the **best (most defensible, highest-leverage) development target** for the current `minikanren_1bit_chirho` codebase, and the **wisest path** to reach it, so the project can grow in a way that is true, reproducible, and publishable—*in Jesus’ name*.

## Current reality (what we actually have)

- **Paper suite**: `papers_chirho/` contains Papers **A–F** plus shared formal definitions in `papers_chirho/shared_chirho/shared_defs_chirho.tex` (the `FD-miniKanren` kernel definition + scope clarification are a major strength).
- **Rust**: full reference semantics + solvers/benchmarks + semiring/differentiable work.
- **Clash**: a bounded finite-domain **engine core** with branching/backtracking and an instruction-ish command interface (`clash_chirho/MiniKanrenChirho.hs`), and **successful Yosys synthesis** for `searchEngineChirho` (tens of thousands of generic cells).
- **Calyx**: a minimal kernel demo (`calyx_chirho/domain_chirho.futil`) and additional prototype modules; **successful Yosys synthesis** exists for the Calyx kernel top (`main`) at small cell counts (hundreds, generic).

## The best situation to develop (ultimate design target)

### The “crown jewel” deliverable

Build a **bounded, deterministic FPGA accelerator loop for the FD-miniKanren kernel** that is:

- **End-to-end**: accepts a program/query (or bytecode) + inputs, runs search, returns solutions / counts.
- **Correctness-anchored**: verified **bit-for-bit** against the Rust reference on the same bounded semantics.
- **Synthesis-backed**: includes **real synthesis artifacts** (already started with Yosys for Clash + Calyx) and ideally place-and-route + timing on at least one real device.
- **Hybrid by design**: explicitly supports a **host-managed outer loop** (scheduling, tabling, mode analysis, memory management), while the FPGA executes the hot inner loop at deterministic latency.

This is the best target because it yields the strongest combination of:
- **Truth** (tight scope, no over-claims),
- **Power** (real acceleration),
- **Reproducibility** (scripts + reference oracle),
- **Publishability** (Paper A + Paper E become compelling and defensible).

### What “unbounded” should mean (and what it cannot mean)

- **Not possible**: truly unbounded terms/vars/search *entirely on chip* (finite BRAM/FF/LUT).
- **Best possible**: **semantically unbounded via virtualization**:
  - FPGA = bounded kernel and/or microcoded engine,
  - Host + external memory = “unbounded heap/tables/streams.”

That is the excellent and honest version of “unbounded.”

## The wisest path (milestones that preserve truth and momentum)

### Milestone 1 (fastest, highest credibility): one golden FPGA demo

Pick a single bounded benchmark with a clean oracle:
- **N-Queens (count or first-k solutions)**, or
- **Batch Sudoku**, or
- **A small finite-domain CSP** that exercises unify/fork/backtrack.

Then produce:
- the FPGA result,
- the Rust reference result,
- a deterministic “same outputs” check,
- and a short reproducibility script.

### Milestone 2: make “unbounded search” first (state spilling)

Before trying to “unbound everything,” make search depth/time unbounded by:
- an external **work queue / frontier** (host or DRAM),
- FPGA processes batches of states and emits successors.

This gives the biggest “unbounded feeling” with the least semantic risk.

### Milestone 3: make terms unbounded (external hash-consing)

Implement hash-consing as a virtualized service:
- small on-chip hot cache,
- backing store in DRAM/host.

Term IDs grow without a visible limit (until memory runs out), while the FPGA stays bounded internally.

### Milestone 4: scale variables/domains (paged representations)

- **Unbounded variables**: substitution / equivalence structures in DRAM, FPGA caches hot slices.
- **Domains >64**: paged bitsets (like GPU `Vec<u32>` idea), FPGA operates on pages via bursts.

### Milestone 5: keep tabling/modes host-managed for a long time

SLG completion and mode-driven scheduling are control-heavy; the wisest plan is:
- **host** does tabling / SCC completion / scheduling,
- **FPGA** does propagation + branching inner loop.

This is “excellent engineering”: deterministic hardware + flexible strategy.

## Paper alignment (what to emphasize)

- **Paper A (core)** should remain the flagship: precise FD-miniKanren kernel + strong benchmarks + clear scope.
- **Paper E (hardware)** should emphasize:
  - kernel/engine split (Calyx vs Clash),
  - real synthesis evidence (Yosys) as *generic mapping*,
  - and the hybrid host/FPGA architecture as the path to “semantic unboundedness.”
- **Papers B–D–F** are best as companion depth papers/tech reports unless a specific venue wants them.

## What not to do (wise cautions)

- Don’t pursue “fully unbounded in pure hardware” first; it becomes a control/memory system before it becomes a solver.
- Don’t claim vendor utilization percentages unless you have vendor mapping (Vivado/Quartus/nextpnr) or a clear mapping methodology.
- Don’t conflate “kernel unification as AND” with “full structural unification” without the finite-domain qualifier.

## Prayerful closing

May the work be done with truth, clarity, and excellence—*Soli Deo Gloria* ☧

