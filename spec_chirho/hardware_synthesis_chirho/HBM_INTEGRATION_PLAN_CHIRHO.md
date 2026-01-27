# HBM Integration Plan ☧

*For God so loved the world... - John 3:16*

## Problem Statement

Current benchmark shows **843 cycles/op** instead of expected **8 cycles/op**.

**Root cause:** PCIe round-trip latency (~2-3 μs) per register access.

```
Current data flow (SLOW):
  CPU → PCIe → AXI-Lite Register → FPGA Logic → Register → PCIe → CPU
  Latency: ~3 μs per operation

Target data flow (FAST):
  CPU → PCIe/DMA → HBM ← FPGA Logic (local HBM access)
  Setup: ~10 μs (one-time DMA)
  Compute: ~80 ns per HBM access (20 cycles @ 250 MHz)
```

---

## AWS F2 HBM Specifications

| Spec | Value |
|------|-------|
| HBM Channels | 32 |
| Total Bandwidth | 460 GB/s |
| Per-Channel BW | 14.4 GB/s |
| Per-Channel Size | 512 MB |
| Total HBM | 16 GB |
| Access Latency | ~80 ns |

---

## Architecture Redesign

### Current Architecture (Register-Based)

```
┌──────────────────────────────────────────────────────────┐
│                    Host (CPU)                             │
│  write CMD registers → wait → read RESP registers         │
└──────────────────────────────────────────────────────────┘
                    │ PCIe (slow, per-op)
                    ▼
┌──────────────────────────────────────────────────────────┐
│                  FPGA Fabric                              │
│  ┌────────────┐    ┌──────────────────────────────────┐  │
│  │ AXI-Lite   │───▶│ Search Engine (8 vars × 64 bits) │  │
│  │ Registers  │◀───│ State in flip-flops              │  │
│  └────────────┘    └──────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### Target Architecture (HBM-Based)

```
┌──────────────────────────────────────────────────────────┐
│                    Host (CPU)                             │
│  DMA problem batch to HBM → start → DMA results back      │
└──────────────────────────────────────────────────────────┘
                    │ PCIe/DMA (batch, amortized)
                    ▼
┌──────────────────────────────────────────────────────────┐
│                  FPGA Fabric                              │
│  ┌────────────┐    ┌──────────────────────────────────┐  │
│  │ AXI-Lite   │───▶│ Control FSM                      │  │
│  │ (control)  │    │ - Start/stop                     │  │
│  └────────────┘    │ - Problem count                  │  │
│                    │ - Status/done                    │  │
│  ┌────────────┐    └──────────────────────────────────┘  │
│  │            │              │                           │
│  │  HBM Ch 0  │◀────────────▶│ Search Engine            │
│  │  Problems  │              │ - Read problem from HBM  │
│  │            │              │ - Compute (8 cycles)     │
│  ├────────────┤              │ - Write result to HBM    │
│  │  HBM Ch 1  │◀────────────▶│                          │
│  │  Results   │              │ Pipelined: 1 result/cycle│
│  │            │              │ after initial latency    │
│  └────────────┘              └──────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

---

## Data Layout in HBM

### Channel 0: Problem Queue (Input)

```
Offset 0x0000_0000: Problem 0
  [0x00] var0_domain (64 bits)
  [0x08] var1_domain (64 bits)
  ...
  [0x38] var7_domain (64 bits)
  [0x40] constraint_mask (64 bits)  // Which constraints to apply
  [0x48] reserved (64 bits)

Offset 0x0000_0080: Problem 1
  ... (same layout)

Problem size: 128 bytes (0x80)
Problems per channel: 512 MB / 128 B = 4M problems
```

### Channel 1: Result Queue (Output)

```
Offset 0x0000_0000: Result 0
  [0x00] var0_domain (64 bits)  // After constraint propagation
  [0x08] var1_domain (64 bits)
  ...
  [0x38] var7_domain (64 bits)
  [0x40] status (64 bits)       // valid, solution_found, etc.
  [0x48] reserved (64 bits)

Result size: 128 bytes (0x80)
Results per channel: 512 MB / 128 B = 4M results
```

### Channel 2-7: Term Store (Optional, Future)

For large problems with hash-consed terms:
```
Channel 2: Terms (cons cells, atoms)
Channel 3: Hash table
Channel 4-7: Reserved for scaling
```

---

## Implementation Steps

### Phase 1: AXI4 HBM Interface (Clash)

Add HBM read/write ports to the search engine:

```haskell
-- New: HBM interface signals
data HbmPortChirho = HbmPortChirho
  { hbmAddrChirho  :: BitVector 34   -- 16GB address space
  , hbmWdataChirho :: BitVector 256  -- 256-bit data bus
  , hbmRdataChirho :: BitVector 256
  , hbmWenChirho   :: Bool
  , hbmRenChirho   :: Bool
  , hbmReadyChirho :: Bool
  } deriving (Generic, NFDataX)

-- Modified search engine with HBM access
searchEngineHbmChirho
  :: Clock XilinxSystem
  -> Reset XilinxSystem
  -> Signal XilinxSystem ControlChirho
  -> Signal XilinxSystem HbmPortChirho  -- NEW: HBM interface
  -> Signal XilinxSystem StatusChirho
```

### Phase 2: FSM for Batch Processing

```haskell
data BatchStateChirho
  = IdleChirho
  | LoadProblemChirho (Index 4194304)  -- Problem index
  | ComputeChirho
  | StoreResultChirho
  | NextProblemChirho
  | DoneChirho

-- Process problems in batch
batchFsmChirho
  :: Signal dom BatchStateChirho
  -> Signal dom (BitVector 34)          -- HBM address
  -> Signal dom Bool                    -- HBM read/write
  -> Signal dom (SearchStateChirho 8)   -- Current computation
```

### Phase 3: Shell Integration

Modify `cl_minikanren_chirho.sv` to connect HBM:

```systemverilog
// Add HBM AXI4 interface
wire [33:0] hbm_axi_awaddr;
wire [255:0] hbm_axi_wdata;
// ... full AXI4 signals

// Connect to AWS F2 shell HBM ports
assign cl_sh_hbm_awaddr = hbm_axi_awaddr;
// ...
```

### Phase 4: Host Driver Update

```python
# New host workflow
def solve_batch_chirho(problems_chirho: List[Problem]) -> List[Result]:
    # 1. DMA problems to HBM channel 0
    dma_write_chirho(HBM_CH0_ADDR, encode_problems_chirho(problems_chirho))

    # 2. Start batch processing
    fpga.pci_poke(REG_CONTROL, START_BATCH | len(problems_chirho))

    # 3. Poll for completion
    while not (fpga.pci_peek(REG_STATUS) & DONE_BIT):
        time.sleep(0.001)

    # 4. DMA results from HBM channel 1
    results_chirho = dma_read_chirho(HBM_CH1_ADDR, len(problems_chirho) * 128)

    return decode_results_chirho(results_chirho)
```

---

## Expected Performance

| Metric | Current | With HBM |
|--------|---------|----------|
| **Setup latency** | 0 | ~10 μs (DMA) |
| **Per-op latency** | 3.37 μs | 80 ns |
| **Cycles/op** | 843 | ~20 |
| **Ops for N-Queens** | 153 | 153 |
| **Total time** | 516 μs | 10 + 12 = **22 μs** |
| **Speedup** | 1.32× | **~30×** |

For batched workloads:
- 1000 problems: 10 μs setup + 12 μs × 1000 = **12 ms** (vs 516 ms current)
- Effective throughput: **~80,000 solves/sec**

---

## Files to Modify

| File | Change |
|------|--------|
| `clash_chirho/MiniKanrenChirho.hs` | Add HBM interface, batch FSM |
| `synth_chirho/aws_f2_chirho/hdk/.../cl_minikanren_chirho.sv` | Connect HBM AXI4 ports |
| `spec_chirho/.../benchmark_nqueens_chirho.py` | Add DMA-based batch mode |
| `synth_chirho/RESULTS_CHIRHO.md` | Update with HBM benchmarks |

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| HBM timing closure | Start with 200 MHz, optimize |
| AXI4 complexity | Use AWS shell IP for HBM interface |
| DMA setup overhead | Batch ≥100 problems to amortize |
| Memory conflicts | Ping-pong buffers for continuous flow |

---

## Success Criteria

1. HBM read/write working from FPGA logic
2. Batch of 1000 N-Queens in <20 ms
3. Speedup ≥20× over current register-based approach
4. Stable operation under continuous load

---

*Soli Deo Gloria* ☧
