# Hardware Benchmark Status - TRANSPARENT ASSESSMENT ☧

> **Last Updated:** 2026-02-01
> **Purpose:** Honest accounting of what hardware benchmarks are real vs. projected

---

## Summary

| Category | Status | Notes |
|----------|--------|-------|
| **Formal Proofs** | ✅ Coq: Main theorem proven | Lean 4: Domain ops only |
| **Register Access** | ✅ Verified | BAR0/OCL works reliably |
| **FSM Completion** | ✅ Verified (V5.6) | With batch_count=1 fix |
| **BAR4→HBM Path** | ⚠️ Unstable | Works after reload, then fails |
| **HBM Batch Mode** | ❌ NOT VERIFIED | Timing violation prevents stable operation |
| **Production Training** | ❌ PROJECTED | Based on models, not measured |

---

## Formal Verification Status

### Coq (formal_chirho/coq/)

| Theorem | Status | File |
|---------|--------|------|
| `intersect_sound_chirho` | ✅ Proven | Domain_chirho.v |
| `intersect_complete_chirho` | ✅ Proven | Domain_chirho.v |
| `intersect_comm_chirho` | ✅ Proven | Domain_chirho.v |
| `intersect_assoc_chirho` | ✅ Proven | Domain_chirho.v |
| `member_intersect_iff_chirho` | ✅ Proven | Domain_chirho.v |
| `domains_refine_chirho` | ✅ Proven | MiniKanren_chirho.v |
| `unify_intersects_chirho` | ✅ Proven | MiniKanren_chirho.v |
| **`search_equiv_chirho`** | ✅ **PROVEN** | MiniKanren_chirho.v |
| `hier_intersect_equiv_chirho` | ⬜ Admitted | Hierarchical = flat |

**Main Result:** The core equivalence (miniKanren search = tensor contraction for finite domains) is **formally proven in Coq**.

### Lean 4 (formal_chirho/lean4/)

| Theorem | Status | File |
|---------|--------|------|
| `intersect_sound_chirho` | ✅ Proven | DomainChirho.lean |
| `intersect_complete_chirho` | ✅ Proven | DomainChirho.lean |
| `intersect_comm_chirho` | ✅ Proven | DomainChirho.lean |
| `intersect_assoc_chirho` | ✅ Proven | DomainChirho.lean |
| `search_equiv_chirho` | ⬜ TODO | Not yet ported from Coq |

**Lean 4 Status:** Domain algebra proven. Main theorem not yet ported.

---

## Hardware Benchmark Categories

### ✅ VERIFIED ON HARDWARE

These benchmarks ran on physical AWS F2 hardware with results verified:

| Benchmark | Measured | Notes |
|-----------|----------|-------|
| Register read latency | 1.29 μs | BAR0 mmap, no O_SYNC |
| Register write latency | ~139 ns | Write-and-forget |
| FSM completion time | ~6 polls | With batch_count=1 |
| FSM throughput | 64K ops/sec | PCIe-limited |
| BAR4 read throughput | 868K ops/sec | After AFI reload |
| BAR4 write throughput | 48M ops/sec | After AFI reload |

### ⚠️ UNSTABLE / REQUIRES RELOAD

These benchmarks work immediately after AFI reload but fail later:

| Benchmark | Initial | After Usage | Root Cause |
|-----------|---------|-------------|------------|
| BAR4 write persistence | ✅ | ❌ | -2.5ns timing |
| FSM correct results | ✅ | ❌ | Reads stale 0s |
| HBM domain storage | ✅ | ❌ | PCIS instability |

### ❌ NOT VERIFIED / PROJECTED

These numbers appear in papers but are **modeled, not measured**:

| Claim | Value | Actual Status |
|-------|-------|---------------|
| "HBM batch throughput" | 2.48M ops/sec | PROJECTED from V3 simulation |
| "Production SAT training" | 3.3M× speedup | MODELED (HBM unstable) |
| "Attention training" | 1.5M× speedup | MODELED (HBM unstable) |
| "TestForge 26,288×" | 26K× | PROJECTED (batch not tested) |

---

## Version-by-Version Reality

### V5.6 (Current AFI: agfi-07ee9410bbbd5932c)

**What Works:**
- Register interface (BAR0) - reliable
- Version ID reads correctly (0xF2560001)
- FSM starts and cycles through states
- With batch_count=1 fix, FSM completes

**What Doesn't Work Reliably:**
- BAR4 writes become unstable after ~10-50 accesses
- Timing violation (-2.506ns WNS at 250MHz)
- HBM domain storage not reliable

**Benchmark Validity:**
- Single FSM operation: ✅ Valid (64K ops/sec)
- HBM batch mode: ❌ Invalid (instability prevents measurement)

### V5.7 (Planned: 125MHz)

**Expected Improvements:**
- 125MHz gives +1.5ns positive slack
- PCIS handler should be stable
- Proper HBM batch benchmarks possible

**Trade-off:**
- Half the clock frequency = ~32K ops/sec baseline
- But stable operation is more valuable than unstable fast

---

## Paper Corrections Needed

### Paper E (FPGA Implementation)

**Current claims that need correction:**

1. **Line 454:** "Timing Slack (WNS) +0.159 ns"
   - This was from an earlier build (V5.x at 200MHz)
   - V5.6 at 250MHz has -2.506ns
   - **Fix:** Update to V5.7 results when available

2. **Lines 730-752:** Production-scale training benchmarks
   - These are **MODELED**, not measured on hardware
   - HBM batch mode is unstable in V5.6
   - **Fix:** Label as "projected" or remove until V5.7 verified

3. **Table 7 (tab:verified_results_chirho):** "Verified benchmark results"
   - TestForge/ConfigGuard numbers are PCIe-limited single ops
   - The dramatic speedups require batch mode which isn't stable
   - **Fix:** Distinguish single-op (verified) from batch (projected)

### Honest Claims We CAN Make

1. **Formal verification:** Main theorem proven in Coq ✅
2. **AFI deployment:** Design runs on AWS F2 hardware ✅
3. **Register interface:** 64K ops/sec single operations ✅
4. **Architecture validation:** FSM, hierarchical modes work ✅

### Claims We CANNOT Make Until V5.7

1. Stable HBM batch mode throughput
2. Production-scale training benchmarks
3. Multi-million× speedups

---

## Next Steps

1. **Build V5.7 at 125MHz** - Fix timing violation
2. **Re-run benchmarks** - Get real HBM numbers
3. **Update papers** - Replace projected with measured
4. **Document clearly** - What's proven vs. projected

---

*Soli Deo Gloria* ☧
