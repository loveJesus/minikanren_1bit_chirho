# Plan: miniKanren as 1-Bit Matrix Operations ☧

> **For active sprint tracking, see [`SPRINT_CHIRHO.md`](./SPRINT_CHIRHO.md)**

This document captures the short-term and long-term roadmap for the project.

---

## 📋 Short-Term Plan (Current Focus)

**Duration:** Until V5 FPGA synthesis completes and AFI is validated

### Immediate Tasks

1. **V5 HDK Build Completion**
   - [ ] Monitor V5 synthesis on AWS (200MHz, SSI_SpreadLogic_high)
   - [ ] Download artifacts when complete
   - [ ] Create AFI and validate on f2.6xlarge
   - [ ] Update BUILD_HISTORY_CHIRHO.md with results

2. **Hierarchical Domain Deployment**
   - [ ] Compile `Hierarchical512Chirho.hs` to Verilog via Clash
   - [ ] Integrate into V5 design for next synthesis iteration
   - [ ] Benchmark 512² vs 64³ (target: 1.5× speedup)

3. **Documentation Sync**
   - [x] Articulate Theorem 5 (tensor isomorphism) in papers
   - [x] Generate PDFs for all subpapers
   - [x] Symlink README.md → README_CHIRHO.md
   - [ ] This plan file created and linked from AGENTS.md

### ⏹️ Short-Term End Marker

**When to stop short-term work:**
- V5 AFI is validated on physical F2 hardware, OR
- Build fails and requires architectural changes

**Instructions at short-term end:**
1. Update `SPRINT_CHIRHO.md` with build results
2. Update `BUILD_HISTORY_CHIRHO.md` with V5 outcome
3. Commit status: `git commit -m "checkpoint: V5 synthesis [success|failure]"`
4. Read Long-Term Plan below before starting next phase

---

## 🗺️ Long-Term Plan

### Phase 1: Theoretical Foundation (1-3 months)

**Goal:** Strengthen Theorem 5 to a publishable proof

1. **Formalize the Isomorphism**
   - Rigorous definition of miniKanren operational semantics
   - Formal tensor algebra over Boolean semiring
   - Prove structure-preserving bijection (not just behavioral equivalence)
   - Connect to existing work: linear logic, proof nets, game semantics

2. **Extend to Infinite Domains**
   - Hash consing as demand-driven dimension extension
   - Lazy tensor construction semantics
   - Prove termination conditions (mode analysis → finite approximation)

3. **Write Cross-Disciplinary Paper**
   - Target: POPL, PLDI, or ICFP
   - Angle: "Hidden tensor structure in relational programming"
   - Contribution: Mathematical identity, not engineering optimization

### Phase 2: Hardware Verification (3-6 months)

**Goal:** Full hierarchical domains on FPGA with verified correctness

1. **Deploy All Clash Modules**
   - `Hierarchical512Chirho` (262K domains, 2-level)
   - `HbmEngineChirho` (batch processing with HBM)
   - `DiffTrainChirho` (differentiable training on FPGA)
   - `AdaptiveHbmChirho` (auto-select domain size)

2. **Verification**
   - Property-based testing (QuickCheck/proptest)
   - Formal verification via Clash's type system
   - Simulation coverage: >95% toggle coverage

3. **Benchmark Against Claims**
   - Verify all README benchmark numbers on V5 hardware
   - Document any discrepancies
   - Update papers with verified results

### Phase 3: Learning Integration (6-12 months)

**Goal:** Differentiable logic programs learning from data

1. **Gumbel-Softmax Training**
   - Implement end-to-end differentiable search
   - Train relation weights from examples
   - Benchmark: program synthesis from I/O pairs

2. **HMC for Probabilistic Inference**
   - Hamiltonian Monte Carlo over soft domains
   - Sample from posterior over program structures
   - Application: probabilistic type inference

3. **Neurosymbolic Applications**
   - Knowledge graph completion
   - Inductive logic programming
   - Constraint satisfaction learning

### Phase 4: Ecosystem & Tooling (12+ months)

**Goal:** Make the tensor view accessible to practitioners

1. **Language Integration**
   - Python bindings (PyO3)
   - Haskell DSL for relational programs
   - JavaScript/WASM for web deployment

2. **Visual Tools**
   - WebGPU visualization of tensor contractions
   - Interactive domain exploration
   - Debugging interface for search states

3. **Industrial Applications**
   - Database query optimization
   - Configuration management (ConfigGuard)
   - Test data generation (TestForge)

---

## 📖 How to Use This Plan

### Starting a Session

1. Check current sprint: `cat SPRINT_CHIRHO.md`
2. Check this plan for context
3. Check build status if FPGA work is ongoing

### During Work

- Update `SPRINT_CHIRHO.md` with progress
- Commit frequently with descriptive messages
- Run `/audit-chirho` before major commits

### Transitioning Phases

When short-term plan ends:
1. Document outcome in SPRINT_CHIRHO.md
2. Review Long-Term Plan for next phase
3. Create new short-term tasks from next phase
4. Update this file with new short-term section

---

## 🔗 Related Documents

| Document | Purpose |
|----------|---------|
| [`AGENTS.md`](./AGENTS.md) | Naming conventions, framework rules |
| [`SPRINT_CHIRHO.md`](./SPRINT_CHIRHO.md) | Current sprint tracking |
| [`BUILD_HISTORY_CHIRHO.md`](./synth_chirho/BUILD_HISTORY_CHIRHO.md) | FPGA synthesis history |
| [`README_CHIRHO.md`](./README_CHIRHO.md) | Project overview |
| [`paper_chirho/`](./paper_chirho/) | Academic papers |

---

*Soli Deo Gloria* ☧
