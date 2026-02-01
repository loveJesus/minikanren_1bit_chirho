# HBM Investigation Summary - 2026-01-31 ☧

## For God so loved the world - John 3:16

---

## Executive Summary

**Finding:** HBM has never actually worked in any version (V2, V3, V4, V5.x). All "HBM benchmarks" were measuring PCIe register throughput, not HBM operations.

| What We Thought | Reality |
|-----------------|---------|
| "V3 has working HBM integration" | HBM IP instantiated but never connected to host |
| "V5.5 HBM batches at 1.78M ops/sec" | PCIe register writes to BAR0, not HBM |
| "BAR4 provides direct HBM access" | PCIS interface tied off in all versions |
| "hbm_ready=1 means HBM works" | Only means HBM controller initialized |

---

## Root Cause: PCIS Tied Off

In `cl_minikanren_chirho.sv` (all versions V2-V5.5):

```systemverilog
// Lines 122-139 in V5.5 (similar in V3)
// PCIS (DMA Slave) Tie-offs
always_comb begin
    cl_sh_dma_pcis_awready = 1'b0;  // NOT accepting writes!
    cl_sh_dma_pcis_wready  = 1'b0;  // NOT accepting write data!
    cl_sh_dma_pcis_arready = 1'b0;  // NOT accepting reads!
    cl_sh_dma_pcis_bid     = 6'b0;
    cl_sh_dma_pcis_bresp   = 2'b0;
    cl_sh_dma_pcis_bvalid  = 1'b0;
    cl_sh_dma_pcis_rid     = 6'b0;
    cl_sh_dma_pcis_rdata   = 512'b0;
    cl_sh_dma_pcis_rresp   = 2'b0;
    cl_sh_dma_pcis_rlast   = 1'b0;
    cl_sh_dma_pcis_ruser   = 1'b0;
    cl_sh_dma_pcis_rvalid  = 1'b0;
end
```

**Impact:** All BAR4 (128GB address space) traffic is silently dropped by the shell due to timeout.

---

## Architecture Comparison

### What We Built (V3-V5.5)

```
Host CPU ──PCIe──┬─► BAR0 (OCL) ──► Registers ──► FSM ──► hbm_axi4_bus ──┐
                 │                                                        │
                 │                                                        ▼
                 │                                               ┌───────────────┐
                 │                                               │    HBM IP     │
                 │                                               │  (16GB @450MHz)│
                 │                                               └───────────────┘
                 │
                 └─► BAR4 (PCIS) ──► TIED OFF (awready=0, arready=0)
```

**Problems:**
1. BAR4 writes go nowhere (PCIS tied off)
2. Even internal FSM path fails (HBM returns arready=0)

### What AWS Examples Do (cl_dram_hbm_dma)

```
Host CPU ──PCIe──┬─► BAR0 (OCL) ──► Control registers
                 │
                 └─► BAR4 (PCIS) ──► cl_dma_pcis_slv ──► AXI Crossbar ──┬─► DDR (0x00_0000_0000)
                                                                         │
                                                                         └─► HBM (0x10_0000_0000)
```

**Key difference:** PCIS is actually connected and routes to memory.

---

## What The Benchmarks Actually Measured

### V5.5 Comprehensive Benchmark

```c
// v55_comprehensive_benchmark_chirho.c
void run_massive_batch_chirho(void) {
    for (int i = 0; i < batch; i++) {
        // This writes to registers at 0x100/0x108 (BAR0)
        // NOT to HBM via BAR4
        fpga_queue_write_chirho(domain_a, domain_b);
    }
}
```

| Claimed | Actual |
|---------|--------|
| "HBM batch throughput" | PCIe register write speed |
| "892M ops/sec for social graph" | Derived from (modeled ops ÷ PCIe time) |
| "35.6B ops/sec intertextual" | Same - modeled, not measured |

### BAR4 Direct Access Test (2026-01-31)

```
BAR4 size: 137438953472 bytes (128GB)
Offset      0: wrote 0xCAFE0000, read 0x00000000 FAIL
Offset     64: wrote 0xCAFE0001, read 0x00000000 FAIL
Offset   256: wrote 0xCAFE0002, read 0x00000000 FAIL
Offset  4096: wrote 0xCAFE0003, read 0x00000000 FAIL

All writes silently dropped - shell times out due to awready=0
```

---

## Two Separate Issues

### Issue 1: PCIS Tied Off (Host → HBM path broken)

- All ready signals hardcoded to 0
- Shell times out on any BAR4 transaction
- No data can flow from host to HBM

### Issue 2: HBM AXI Handshake (Internal FSM → HBM path broken)

Even if triggered via registers, the FSM gets stuck:

```
Debug register 0xC4 (AXI_STATUS):
  arvalid=1 (FSM requesting read)
  arready=0 (HBM not accepting)
```

The `cl_hbm_axi4` wrapper or HBM IP is not responding to AXI requests.

---

## How This Went Undetected

1. **No end-to-end test:** Never verified data flowed through HBM and back
2. **Measuring wrong thing:** PCIe throughput labeled as "HBM performance"
3. **hbm_ready confusion:** Signal = HBM controller initialized, not "HBM usable"
4. **Copy-paste inheritance:** PCIS tie-offs copied from V2 to V3 to V4 to V5
5. **Impressive numbers:** 1.78M ops/sec looked good (but it was just PCIe)

---

## What Actually Works in V5.5

| Component | Status | Notes |
|-----------|--------|-------|
| BAR0 register access | ✅ Works | 0.78M reads/sec, 1.78M writes/sec |
| Legacy 64-bit engine | ✅ Works | Via registers, no HBM |
| HBM IP initialization | ✅ Works | hbm_ready=1 |
| BAR4 (PCIS) access | ❌ Broken | Tied off |
| HBM data path | ❌ Broken | arready=0 |
| Hierarchical domains | ❌ Broken | Depends on HBM |

---

## V6 Requirements

To achieve actual HBM functionality:

1. **Implement PCIS handler**
   - Copy `cl_dma_pcis_slv.sv` from AWS examples
   - Connect to AXI crossbar

2. **Add AXI crossbar**
   - Route BAR4 traffic to HBM
   - HBM at offset 0x10_0000_0000 (64GB mark)

3. **Debug HBM AXI handshake**
   - Why does cl_hbm_axi4 return arready=0?
   - Check clock domain crossing
   - Verify HBM IP configuration

4. **Add verification tests**
   - Write pattern to HBM, read back, compare
   - End-to-end data flow test
   - Not just "hbm_ready=1"

---

## Affected Documentation

Updated files:
- `synth_chirho/BUILD_HISTORY_CHIRHO.md` - Added PCIS discovery
- `benchmarks_chirho/V55_BENCHMARK_REPORT_20260131_CHIRHO.md` - Added warning

Version table correction:
```
| v5.5 | ... | ⚠️ Limited | STATUS fix, BAR0 works, BAR4/PCIS tied off |
```

---

## Lessons Learned

1. **Verify data flow end-to-end**, not just interface initialization
2. **Question inherited code** - tie-offs were copied without understanding
3. **Label metrics accurately** - "PCIe throughput" ≠ "HBM throughput"
4. **Test the actual path** - BAR4 write-read test would have caught this immediately

---

*Soli Deo Gloria* ☧

**Investigation Date:** January 31, 2026
**AFI Tested:** agfi-0261e88151bcb39a5 (V5.5)
**Instance:** i-08666433585fc1187 (terminated)
