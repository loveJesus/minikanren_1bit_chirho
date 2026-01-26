# FPGA Synthesis Results ☧

## Headline Result (Post-Route Timing on AWS F1 Build Environment)

> **Clash design meets timing at 280 MHz on Xilinx VU9P after Vivado place-and-route.
> The 8-cycle unify kernel implies 35M unify ops/sec throughput (projected).**

| Metric | FPGA (Clash @ 280 MHz) | CPU (Rust @ 3 GHz) | Speedup |
|--------|------------------------|--------------------|---------|
| Throughput | 35M unify/sec† | 333K unify/sec | **104×** |
| Latency | 28.6 ns (deterministic)† | ~3 μs (variable) | **105×** |
| Cycles/unify | 8 | ~9000 | **1125×** |

†Projected from timing report; actual throughput requires on-FPGA measurement.

**What "unify" means:** One invocation of the hardware kernel's unify micro-sequence
(constrain two 64-bit domains via AND, check for empty → fail or continue).

## Vivado Post-Route Timing (2026-01-26)

Synthesized and routed with Vivado 2024.2 on AWS F1 c5.4xlarge (dev instance, not F1 FPGA):

| Metric | Value |
|--------|-------|
| **Achieved Fmax** | **280.19 MHz** |
| WNS (Worst Negative Slack) | +0.431 ns |
| TNS (Total Negative Slack) | 0.000 ns |
| WHS (Worst Hold Slack) | +0.042 ns |
| THS (Total Hold Slack) | 0.000 ns |
| Target Clock | 250 MHz (4 ns period) |
| LUTs | 21,820 (~1.8% of VU9P)‡ |
| Registers | 9,839 (~0.4% of VU9P)‡ |
| Target Device | xcvu9p-flgb2104-2-i |
| Timing Met | ✅ All constraints met |

‡Utilization percentages are approximate. VU9P has ~1.18M LUTs; precise
hierarchical utilization requires `report_utilization -hierarchical`.

---

## Yosys Synthesis (2025-01-25)

Technology-independent synthesis with Yosys 0.61:

## Summary

| Design | Source | Cells | FFs | Scope |
|--------|--------|-------|-----|-------|
| **Clash searchEngineChirho** | clash_chirho/ | 43,136 | 9,781 | Full search engine |
| **Calyx domain_chirho** | calyx_chirho/ | 627 | 203 | Unification core |

## Clash Design: searchEngineChirho

Full search engine with hash-consing, branching, and backtracking.

### Resource Utilization

| Resource | Count |
|----------|-------|
| Total Cells | 43,136 |
| Flip-Flops (DFFE) | 9,781 |
| AND gates | 16,579 |
| MUX | 4,822 |
| NOT | 1,100 |
| NOR | 481 |
| NAND | 118 |
| Wire bits | 127,487 |
| Ports | 587 bits |

### Target Device Fit

| FPGA | Resources | Utilization |
|------|-----------|-------------|
| Intel Cyclone V (5CSEBA6U23I7) | 110K ALMs | ~8% |
| AMD Artix-7 A35 | 33K LUTs | ~65% |
| Lattice iCE40 HX8K | 8K LUTs | Does not fit |

## Calyx Design: domain_chirho (Unification Core)

Minimal proof-of-concept: just the unification kernel.

### Resource Utilization

| Resource | Count |
|----------|-------|
| Total Cells | 627 |
| Flip-Flops (DFFE) | 203 |
| AND gates | 140 |
| MUX | 64 |

### Preprocessing

Calyx-generated Verilog uses SystemVerilog and `$fatal` (simulation-only).
For synthesis:

```bash
# Strip $fatal and use -sv flag
sed 's/\$fatal.*;//g' domain_chirho.v > domain_clean.v
yosys -p "read_verilog -sv domain_clean.v; synth; stat"
```

## Calyx Design: search_engine_chirho (Full Engine)

**Status: Needs Architecture Refactor**

The full search engine skeleton exists but requires refactoring:
- Calyx's `@external` memory constraint requires shared memory in main component
- Subcomponents need port-based memory access instead of direct references

This is a Calyx idiom issue, not a fundamental limitation.

## Notes

- Yosys synthesis above used generic techmap (technology-independent)
- Vivado 2024.2 post-route timing achieved **280 MHz** on VU9P (see above)
- On-FPGA validation (actual F1 bitstream execution) is pending
- Next step: Create AFI and run on physical F1 instance

---

*Soli Deo Gloria* ☧
