# Plan: Address Gemini Paper Review Feedback ☧

## Overview

Gemini reviewed an earlier version of the paper and provided detailed technical feedback.
This plan tracks addressing each point in both code and paper.

---

## Weaknesses to Address

### W1. The 64-Bit Horizon ✅ ALREADY ADDRESSED
**Concern:** Performance cliff when universe exceeds 64 bits.

**Our solution (already implemented):**
- `BitVec256Chirho` - 256-bit domains (hardware_chirho)
- `Hierarchical4kChirho` - 4,096 values (64 × 64 two-level)
- `Hierarchical256kChirho` - 262,144 values (64 × 64 × 64 three-level)
- GPU `Vec<u32>` - Unlimited size domains

**Action:** Add section to paper explaining multi-word scaling.

### W2. FPGA Resource Exhaustion
**Concern:** CAM resources are constrained. When do we exhaust registers?

**Action:**
- [ ] Add benchmark showing LUT usage vs number of variables/domain size
- [ ] Document CAM capacity limits in paper

### W3. Unfair Solver Comparisons ✅ ACKNOWLEDGED
**Concern:** Comparing to Z3 (general SMT) and Python is skewed.

**Our response:**
- OCanren comparison in Table 7 IS the fair comparison
- Add explicit acknowledgment in paper that Z3/clingo solve more general problems

### W4. Differentiable Evaluation
**Concern:** Family relations is trivial. Need MNIST-Sudoku or similar.

**Action:**
- [ ] Implement MNIST-Sudoku benchmark (neural digit recognition + logic solving)
- [ ] Or: implement gradient stability test for deeper search trees

---

## Detailed Technical Feedback

### A. Contraction Order Overhead
**Concern:** If solver takes 3μs but planner takes 50μs, quantify this.

**Action:**
- [ ] Add ablation study: heuristic_time + execution_time
- [ ] Add table showing breakdown for each benchmark

### B. Handling Dynamic Word Sizes ✅ ALREADY IMPLEMENTED
**Concern:** What happens when terms > 64?

**Our solution:**
```rust
// Automatic scaling via Hierarchical domains
if domain_size <= 64 {
    BitVec64Chirho
} else if domain_size <= 4096 {
    Hierarchical4kChirho
} else {
    Hierarchical256kChirho or GPU Vec<u32>
}
```

**Action:** Document this in paper Section 4.

### C. Neurosymbolic Validation
**Concern:** Family relations doesn't prove gradient stability.

**Options:**
1. MNIST-Sudoku (standard benchmark)
2. Gradient norm tracking over depth
3. Variance analysis of Gumbel-Softmax

**Action:**
- [ ] Pick one and implement
- [ ] Add results to Section 9

### D. Minor Errata

#### D1. Email ✅ INTENTIONAL
The email `loveJesus@loveJes.us` is intentional (author's faith statement).

#### D2. GPU 0.3× speedup explanation
**Concern:** Bulk AND (1K) is slower on GPU - explain why.

**Action:**
- [ ] Add sentence explaining memory transfer bottleneck
- [ ] GPU only wins when batch size amortizes PCIe latency

---

## Implementation Checklist

### Paper Updates

- [ ] Section 3: Add "Scaling Beyond 64 Bits" subsection
  - Explain Hierarchical domains
  - Explain GPU unlimited domains

- [ ] Section 5: Add solver comparison caveat
  - "Z3 solves general SMT; our comparison highlights domain-specific speedup"

- [ ] Section 8 (GPU): Add memory bottleneck explanation
  - "For small batches (<10K), PCIe transfer dominates"

- [ ] Section 9 (Differentiable): Add gradient stability analysis
  - Either MNIST-Sudoku or variance tracking

- [ ] Add contraction order ablation table

### Code Updates

- [ ] Add `benchmark_contraction_overhead_chirho()` test
- [ ] Add FPGA resource usage benchmark (or document from Calyx output)
- [ ] Consider MNIST-Sudoku integration test

---

## Priority Order

1. **Paper: Scaling section** (addresses W1, B) - EASY, we have the code
2. **Paper: GPU explanation** (addresses D2) - EASY, one sentence
3. **Paper: Solver caveat** (addresses W3) - EASY, one paragraph
4. **Code: Contraction ablation** (addresses A) - MEDIUM
5. **Code/Paper: Neurosymbolic** (addresses W4, C) - HARD, requires new benchmark

---

*Soli Deo Gloria* ☧
