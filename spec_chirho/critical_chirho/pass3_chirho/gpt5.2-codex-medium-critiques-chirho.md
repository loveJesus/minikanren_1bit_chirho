# GPT-5.2 Codex Medium Suggestions v1 ☧

## Alignment Quick Check (paper ⇄ codebase)
- Core mapping (bitmask domains, AND unification, tensors) appears implemented in Rust and reflected in README.
- Hierarchical domains and BitVec256 exist in `rust_chirho/src/approaches_chirho/hierarchical_chirho.rs`.
- GPU kernels and benches exist in `rust_chirho/src/hardware_chirho/gpu_chirho.rs` and `rust_chirho/benches/gpu_bench_chirho.rs`.
- Differentiable logic exists in `rust_chirho/src/learn_chirho/` and `rust_chirho/benches/differentiable_bench_chirho.rs`.
- AC-3 implemented in `rust_chirho/src/reference_chirho/constraint_chirho.rs`, referenced in README.

## Medium Suggestions (beyond current PRD items)

### 1) Add a “paper claim → artifact” traceability table
**Why:** Make every paper claim reproducible and auditable.  
**Action:** Add a matrix in `spec_chirho/` or README mapping:
- paper section/table → benchmark script / example / module
- exact command(s) used
- expected output CSV/PNG

### 2) Package benchmark data used in the paper
**Why:** Paper includes numeric tables for GPU, N-Queens, Sudoku, gradients, etc.  
**Action:** Add a `benchmarks_chirho/results_chirho/` folder with CSV/JSON outputs and a `run_all_benchmarks_chirho.sh` that reproduces all paper numbers.

### 3) Clarify GPU “occurs check” and transitive closure scope
**Why:** Paper ties GPU TC to occurs check; code implements graph TC and matmul.  
**Action:** Either:
- integrate GPU TC into occurs-check pipeline, or
- clarify in paper that GPU TC is a general reachability kernel used as a building block, not yet wired into unification/occurs-check.

### 4) Reconcile paper tables with the Rust bench suites
**Why:** Paper lists sizes (e.g., 1K/10K/100K edges) that don’t all appear in `datalog_bench_chirho.rs`.  
**Action:** Align bench configs with paper table rows, or explicitly cite external Soufflé scripts + data files in `benchmarks_chirho/`.

### 5) Add reproducibility scripts for gradient attenuation table
**Why:** Paper’s gradient table is specific; code has differentiable components but no clear script that outputs that exact table.  
**Action:** Add a `benchmarks_chirho/gradients_chirho/` script that reproduces Table “Gradient magnitude vs inference depth”.

### 6) Document GPU memory limits in paper’s “unlimited domain” claim
**Why:** “Unlimited” is practically bounded by VRAM; paper should clarify.  
**Action:** Add a sentence: “Unlimited in principle, bounded by GPU memory; domains scale to billions of values on 8–80GB VRAM.”

### 7) Make `Hierarchical16kChirho` and `BitVec256Chirho` visible in docs
**Why:** Paper highlights these; README’s domain table omits `Hierarchical16kChirho`.  
**Action:** Add to README domain table and add a tiny example or bench showing 16k behavior.

### 8) Tighten hardware claims labeling
**Why:** Paper and README include LUT estimates and FPGA cycle counts.  
**Action:** Mark estimates clearly vs measured, and link to actual synthesis reports once P2-7 finishes.
