# Open Problems ☧

## 1. Theoretical Challenges

### Problem 5.1: Optimal Contraction Order

**Status:** Open (NP-hard)

Tensor network contraction order is equivalent to tree decomposition.
Finding optimal order is NP-hard (same as in quantum simulation).

**Current approach:** Greedy heuristics based on:
- Minimize intermediate tensor size
- Contract sparse tensors first
- Use structure (e.g., Sudoku's regular constraints)

**Research directions:**
- Machine learning for order prediction
- Approximate methods with bounds
- Exploit problem-specific structure

### Problem 5.2: Infinite Domains Without Hash Consing

**Status:** Partially solved

For truly infinite domains (integers, strings), we need:
1. Symbolic representation
2. Lazy enumeration
3. Constraint propagation

**Current solution:** Hash consing creates terms on-demand.
**Limitation:** Still finite (bounded by term store size).

**Open question:** Can we represent infinite domains compactly
for hardware without hash consing?

### Problem 5.3: Occurs Check Complexity

**Status:** Open

Occurs check via transitive closure is O(n³) naively, O(n^2.37)
with fast matrix multiplication.

**Current approach:** Incremental updates to reachability matrix.

**Open questions:**
- Can hardware parallelism reduce this to O(n²) or better?
- Is there a lazy occurs check that avoids full closure?
- Can we use e-graph union-find structure?

### Problem 5.4: Differentiable Relaxation

**Status:** Research frontier

Soft unification using probability semiring:
- Domains as probability distributions
- AND → product (normalized)
- OR → sum (normalized)

**Open questions:**
- How to make gradients flow through branching?
- Connection to Scallop / DeepProbLog?
- Can we learn relation tensors from examples?

## 2. Implementation Challenges

### Problem 5.5: CAM Scalability

**Status:** Engineering challenge

Content-Addressable Memory for hash consing:
- Current: 64-entry direct-mapped (high collision rate)
- Needed: 10K+ entries for practical problems

**Solutions being explored:**
- Set-associative CAM (4-way, 8-way)
- Cuckoo hashing in hardware
- Hierarchical CAM (L1/L2 cache)

### Problem 5.6: Variable Ordering Heuristics

**Status:** Partially solved

Fail-first (smallest domain first) is implemented.

**Open questions:**
- Can we learn good orderings?
- Problem-specific orderings (e.g., Sudoku patterns)?
- Dynamic reordering during search?

### Problem 5.7: Sparse Tensor Representation

**Status:** Engineering challenge

Relations like `appendo` are very sparse. Current approach:
- COO format (coordinate list of tuples)

**Challenges:**
- Hardware-friendly sparse format
- Efficient slicing for constraint propagation
- Incremental updates (tabling)

### Problem 5.8: Multi-FPGA Scaling

**Status:** Future work

For large problems (1000+ variables), single FPGA insufficient.

**Open questions:**
- Partitioning strategy (variables vs constraints)?
- Communication protocol between FPGAs?
- Load balancing for parallel branches?

## 3. Language Design Challenges

### Problem 5.9: Embedding in Host Languages

**Status:** Partially solved

Current: Python/Rust DSL.
Desired: Seamless integration.

**Open questions:**
- Macro-based embedding (Scheme-style)?
- Type-level embedding (Haskell-style)?
- Compiler integration for optimization?

### Problem 5.10: Debugging and Visualization

**Status:** Minimal support

Users need to understand:
- Why did search fail?
- Why is it slow?
- What is the current state?

**Open questions:**
- Visualization of tensor networks?
- Step-through debugger for hardware?
- Performance profiler for constraints?

## 4. Connections to Other Fields

### Problem 5.11: SMT Solver Integration

**Status:** Unexplored

Z3 and other SMT solvers have sophisticated theory solvers.

**Open questions:**
- Can we use SMT for integer constraints?
- Bit-parallel → DPLL connection?
- Hybrid symbolic/bit-parallel approach?

### Problem 5.12: E-Graph Unification

**Status:** Partially explored

E-graphs (egg) and miniKanren both do unification.

**Open questions:**
- Shared representation for terms?
- E-matching as specialized search?
- Bidirectional integration?

### Problem 5.13: Probabilistic Programming

**Status:** Research frontier

Connection to probabilistic inference:
- Domains as distributions
- Relations as conditional probabilities
- Search as marginal computation

**Open questions:**
- Exact vs approximate inference?
- Sampling in hardware?
- Variational relaxations?

## 5. Hardware-Specific Challenges

### Problem 5.14: ASIC vs FPGA Trade-offs

**Status:** Open

FPGA: Flexible, expensive per unit
ASIC: Fixed, cheap at scale

**Open questions:**
- Minimum viable ASIC specification?
- Reconfigurability requirements?
- Hybrid approach (ASIC core + FPGA flexibility)?

### Problem 5.15: Memory Technology

**Status:** Future work

New memory technologies:
- ReRAM for CAM (lower power)
- 3D stacking (higher bandwidth)
- Near-memory compute

**Open questions:**
- Which memory technology best fits?
- Processing-in-memory for relation tensors?

### Problem 5.16: Quantum Acceleration

**Status:** Speculative

Grover's algorithm could accelerate search O(√N).

**Open questions:**
- Suitable problem formulation?
- Hybrid classical-quantum approach?
- Near-term quantum device feasibility?

## 6. Practical Application Challenges

### Problem 5.17: Real-World Benchmarks

**Status:** Limited

Current benchmarks are toy problems.

**Needed:**
- Compiler type inference at scale
- Realistic constraint satisfaction
- Database query optimization

### Problem 5.18: Integration with Existing Tools

**Status:** Minimal

Desired integrations:
- Compiler backends
- Formal verification tools
- Database query planners

**Open questions:**
- API design for hardware accelerator?
- Latency vs throughput trade-offs?
- Fallback for unsupported features?

---

*Soli Deo Gloria* ☧
