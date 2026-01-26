<!--
For God so loved the world, that he gave his only begotten Son,
that whosoever believeth in him should not perish, but have everlasting life.
John 3:16 (KJV)
-->
<!-- For God so loved the world, that He gave His only begotten Son, that all who believe in Him should not perish but have everlasting life. -->

## Measured Performance (On-FPGA) ☧

This file records **measured** (not projected) throughput and latency once the design runs on real FPGA hardware (e.g. AWS F1 AFI or a local board).

### Measurement environment

- **Platform**: (AWS F1 AFI / Board model)
- **FPGA device**: (e.g. VU9P)
- **Build toolchain**: (Vivado version, build settings)
- **Host**: (instance type / CPU / OS)
- **Interface**: (PCIe / AXI-Lite / DMA protocol)
- **Clock constraints**: (target period, achieved Fmax, I/O constraints status)

### Workload (Golden Demo)

- **Benchmark**: (N-Queens / Sudoku / other)
- **Problem size**: (e.g. N=12)
- **Kernel semantics**: FD-miniKanren bounded kernel (domain intersection + branch/backtrack)
- **Correctness oracle**: Rust `SearchStateHwChirho` + Verilator co-sim (bit-identical)

### Results

| Metric | Value | Notes |
|--------|-------|------|
| End-to-end latency (single query) | TBD | Host → FPGA → Host |
| Sustained throughput | TBD | Queries/sec or kernel-ops/sec |
| FPGA utilization | TBD | vendor report |
| Power (optional) | TBD | board/instance dependent |

### Repro steps

- **Commands**: (exact commands to run the benchmark and collect timings)
- **Artifacts**: (links/paths to AFI build outputs and logs)

