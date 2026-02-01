# V6 Architecture Analysis: Dual-Pathway Sparse Hierarchies ☧

For God so loved the world - John 3:16

---

## Requirements

1. **Dual word sizes:** 256-bit (native HBM) and 512-bit (proven in V5.3)
2. **Variable depth:** 2, 3, or 4 levels per word size
3. **Sparse streaming** (implicit indexing via popcount)
4. **HBM-friendly** (sequential access patterns)
5. **V5.4 fixes included:** Register map, AXI handshake, debug registers

---

## V6 Domain Modes (6 Total)

| Mode | Word Size | Depth | Domain Size | HBM beats/word | Latency (sparse) |
|------|-----------|-------|-------------|----------------|------------------|
| 256² | 256-bit | 2 | 65K | 1 | ~100ns |
| 256³ | 256-bit | 3 | 16.7M | 1 | ~150ns |
| 256⁴ | 256-bit | 4 | 4.3B | 1 | ~200ns |
| 512² | 512-bit | 2 | 262K | 2 | ~200ns |
| 512³ | 512-bit | 3 | 134M | 2 | ~300ns |
| 512⁴ | 512-bit | 4 | 68B | 2 | ~400ns |

**Mode encoding:**
```systemverilog
// {word_size[0], depth[1:0]} - 3 bits total
`define HIER_MODE_256_2_CHIRHO  3'b0_01  // 256², 65K values
`define HIER_MODE_256_3_CHIRHO  3'b0_10  // 256³, 16.7M values
`define HIER_MODE_256_4_CHIRHO  3'b0_11  // 256⁴, 4.3B values
`define HIER_MODE_512_2_CHIRHO  3'b1_01  // 512², 262K values
`define HIER_MODE_512_3_CHIRHO  3'b1_10  // 512³, 134M values
`define HIER_MODE_512_4_CHIRHO  3'b1_11  // 512⁴, 68B values
```

### Memory Requirements (Sparse 0.001%)

| Mode | Dense Storage | Sparse Storage |
|------|---------------|----------------|
| 256² | 8 KB | ~70 bytes |
| 256³ | 2 MB | ~2 KB |
| 256⁴ | 512 MB | ~500 KB |
| 512² | 33 KB | ~270 bytes |
| 512³ | 16 MB | ~16 KB |
| 512⁴ | 8.5 GB | ~8 MB |

---

## V5.4 Fixes Carried to V6

### 1. Register Map (DONE)
```
HIER/TRAIN registers moved from 0x40-0x70 to 0x80+
No conflicts with RESP registers (0x20-0x5C)
```

### 2. AXI Handshake (REQUIRED)
```systemverilog
// V5.3 BUG: Set arvalid, immediately wait for rvalid
// V6 FIX: Wait for arready before expecting rvalid

FSM_WAIT_ARREADY_CHIRHO: begin
    if (hbm_axi4_bus_chirho.arready) begin
        axi_read_req_chirho <= 1'b0;  // Clear after accepted
        fsm_state_chirho <= FSM_LOAD_DATA_CHIRHO;
    end
end
```

### 3. Debug Registers (REQUIRED)
```
0x0C0: FSM_STATE   - Current FSM state (read-only)
0x0C4: AXI_STATUS  - {arready, arvalid, rvalid, rready, awready, awvalid, bvalid, bready}
0x0C8: AXI_ADDR_LO - Current axi_addr_chirho[31:0]
0x0CC: AXI_ADDR_HI - Current axi_addr_chirho[47:32]
0x0D0: BEAT_COUNT  - Current beat_counter_chirho
0x0D4: LEVEL_INFO  - {current_level[3:0], max_depth[3:0], word_size[7:0]}
```

---

## Hardware Resource Analysis

### Per-Level Resources (Dual Pathway)

| Component | 256-bit | 512-bit | Notes |
|-----------|---------|---------|-------|
| Summary register | 256 FF | 512 FF | Current word being processed |
| Popcount circuit | ~450 LUT | ~900 LUT | Adder tree for rank |
| Priority encoder | ~400 LUT | ~500 LUT | find_next_set_bit |
| AND gate array | 256 LUT | 512 LUT | Intersection |
| Address calculator | ~200 LUT | ~200 LUT | Multiply/shift |
| Word assembly | 0 | ~100 LUT | 2-beat assembly for 512-bit |
| **Total per level** | **~1300 LUT, 256 FF** | **~2200 LUT, 512 FF** |

### Shared Resources (Both Pathways)

| Component | Cost | Notes |
|-----------|------|-------|
| Depth counter | ~50 LUT | Compare against max_depth |
| Mode decoder | ~30 LUT | Select 256/512 path |
| Cumulative trackers | ~200 LUT, 128 FF | 4 levels × 32-bit counters |
| Debug registers | ~100 LUT, 256 FF | V5.4 fix: visibility |
| **Total shared** | **~380 LUT, 384 FF** |

### Configurable vs Hardcoded Depth

#### Option A: Hardcoded Depth (Recommended)

Separate FSMs for each supported depth:

```systemverilog
parameter MAX_LEVELS_CHIRHO = 4;

// Dedicated state machines, parallel instances
fsm_level1_chirho u_l1 (...);  // 512-bit, 1 level = 512 values
fsm_level2_chirho u_l2 (...);  // 512-bit, 2 levels = 262K values
fsm_level3_chirho u_l3 (...);  // 512-bit, 3 levels = 134M values
fsm_level4_chirho u_l4 (...);  // 512-bit, 4 levels = 68B values
```

**Pros:**
- No runtime overhead for depth dispatch
- Each level FSM optimized for its specific path
- Parallel processing: level 1 of variable A while level 2 of variable B
- Timing closure easier (known path depths)

**Cons:**
- More total resources (~4× for 4 levels)
- Fixed set of supported depths

#### Option B: Configurable Depth (Stack-based)

Single FSM with level stack:

```systemverilog
typedef struct packed {
    logic [10:0] word_size_chirho;     // 64 to 2048
    logic [3:0]  current_level_chirho; // 0-15
    logic [3:0]  max_level_chirho;     // Depth for this variable
    logic [2047:0] summary_stack_chirho [0:15]; // One per level
    logic [47:0] base_addr_stack_chirho [0:15];
    logic [31:0] cumulative_stack_chirho [0:15];
} traversal_state_t_chirho;
```

**Pros:**
- Arbitrary depth (up to 16 levels = astronomical domains)
- Single FSM, less duplicated logic
- Dynamic: same hardware handles any hierarchy

**Cons:**
- Stack access adds latency (register file vs direct)
- Complex state machine with level transitions
- Harder to pipeline (variable path length)

#### Option C: Hybrid (Recommended for V6)

Hardcode common depths (1-4), configurable extension:

```systemverilog
// Fast path: hardcoded for levels 1-4
always_comb begin
    case (hier_mode_chirho)
        HIER_1_CHIRHO: use_fast_l1_chirho = 1'b1;
        HIER_2_CHIRHO: use_fast_l2_chirho = 1'b1;
        HIER_3_CHIRHO: use_fast_l3_chirho = 1'b1;
        HIER_4_CHIRHO: use_fast_l4_chirho = 1'b1;
        default:       use_configurable_chirho = 1'b1; // Stack-based
    endcase
end
```

---

## Word Size Analysis: 256 vs 512 Bits

### HBM Alignment

| Word Size | HBM Beats | Latency | Complexity |
|-----------|-----------|---------|------------|
| 256-bit | 1 | Lowest | Simplest (native) |
| 512-bit | 2 | +1 cycle | Word assembly FSM |

### Popcount Circuit Comparison

```
256-bit popcount:
  8 levels: 128→64→32→16→8→4→2→1
  Total: ~450 LUT, 3 cycles (pipelined)

512-bit popcount:
  9 levels: 256→128→64→32→16→8→4→2→1
  Total: ~900 LUT, 4 cycles (pipelined)
```

### Trade-offs

| Use Case | Best Choice | Why |
|----------|-------------|-----|
| Lowest latency | 256-bit | 1 HBM beat, simpler FSM |
| Larger domains | 512-bit | 512⁴ = 68B vs 256⁴ = 4.3B |
| Shallower trees | 512-bit | 512³ = 134M vs 256³ = 16.7M |
| Resource-constrained | 256-bit | ~60% the LUT cost |

### Recommendation

**Support both 256 and 512-bit** in V6:
- 256-bit for latency-sensitive workloads (4.3B domain max)
- 512-bit for large domains (68B max, proven in V5.3)
- Mode register selects pathway at runtime
- Shared FSM structure, parameterized circuits

---

## CL (Compute Logic) Count Analysis

### VU47P-HBM FPGA Resources (AWS F2)

| Resource | Available | V5.3 Used | V6 (4-level 512-bit) |
|----------|-----------|-----------|----------------------|
| LUT | 1,044,000 | ~5,000 | ~30,000 (3%) |
| FF | 2,088,000 | ~3,000 | ~15,000 (0.7%) |
| BRAM | 1,080 | ~10 | ~50 (4.6%) |
| HBM Channels | 16 | 3 | 6 (37.5%) |

**Conclusion:** Single CL is sufficient. We're using <5% of FPGA resources.

### Multi-CL Options

If we wanted parallel intersection engines:

| Config | Engines | Throughput | Resources |
|--------|---------|------------|-----------|
| 1 CL | 1 | Baseline | ~30K LUT |
| 2 CL | 2 | 2× | ~60K LUT |
| 8 CL | 8 | 8× | ~240K LUT (23%) |
| 16 CL | 16 | 16× | ~480K LUT (46%) |

**Recommendation:** Start with 1 CL in V6. Add parallelism in V7 if needed.

---

## V6 FSM Design: 4-Level Sparse Streaming

### State Machine

```systemverilog
typedef enum logic [4:0] {
    // Idle/Init
    FSM_IDLE_CHIRHO,
    FSM_LOAD_HEADER_CHIRHO,       // Read variable header (encoding, levels, base addrs)

    // Level 0 (always in-register after header load)
    FSM_L0_COMPUTE_CHIRHO,        // AND level0 summaries
    FSM_L0_FIRST_BIT_CHIRHO,      // Find first set bit

    // Level 1
    FSM_L1_LOAD_A_CHIRHO,         // Stream L1 block from var A
    FSM_L1_LOAD_B_CHIRHO,         // Stream L1 block from var B
    FSM_L1_COMPUTE_CHIRHO,        // AND, update cumulative
    FSM_L1_STORE_CHIRHO,          // Store result if needed
    FSM_L1_NEXT_CHIRHO,           // Find next set bit in L0

    // Level 2
    FSM_L2_LOAD_A_CHIRHO,
    FSM_L2_LOAD_B_CHIRHO,
    FSM_L2_COMPUTE_CHIRHO,
    FSM_L2_STORE_CHIRHO,
    FSM_L2_NEXT_CHIRHO,

    // Level 3 (Leaf)
    FSM_L3_LOAD_A_CHIRHO,
    FSM_L3_LOAD_B_CHIRHO,
    FSM_L3_COMPUTE_CHIRHO,
    FSM_L3_STORE_CHIRHO,
    FSM_L3_NEXT_CHIRHO,

    // Completion
    FSM_WRITE_RESULT_HEADER_CHIRHO,
    FSM_DONE_CHIRHO
} hier_fsm_state_t_chirho;
```

### Cumulative Tracking

```systemverilog
// Per-level state (cumulative child counts)
logic [31:0] cum_l1_children_chirho;  // Children before current L1 subtree
logic [31:0] cum_l2_children_chirho;  // Children before current L2 subtree
logic [31:0] cum_l3_children_chirho;  // Children before current L3 subtree

// When we finish an L1 block, update L2 cumulative
always_ff @(posedge clk_chirho) begin
    if (state_chirho == FSM_L1_NEXT_CHIRHO && l1_done_chirho) begin
        cum_l2_children_chirho <= cum_l2_children_chirho +
                                   popcount_512(current_l1_summary_chirho);
    end
end
```

### Address Calculation (Implicit Indexing)

```systemverilog
// Example: Finding L2 block address
// base_l2 = base_l1 + (total L1 blocks) × L1_BLOCK_SIZE
// offset_in_l2 = cum_l2_children + rank(current_l1_summary, current_l1_bit)

logic [47:0] l2_addr_chirho;
assign l2_addr_chirho = base_l2_chirho +
                        (cum_l2_children_chirho +
                         popcount_masked(current_l1_summary_chirho, current_l1_bit_chirho))
                        * L2_BLOCK_SIZE_CHIRHO;
```

---

## Memory Layout (V6)

```
Variable in HBM:
┌────────────────────────────────────────────┐
│ Header (256 bits = 1 HBM beat)             │
│   [7:0]   encoding_chirho (hier depth)     │
│   [15:8]  word_size_log2_chirho            │
│   [47:16] total_popcount_chirho            │
│   [95:48] base_l1_addr_chirho              │
│   [143:96] base_l2_addr_chirho             │
│   [191:144] base_l3_addr_chirho            │
│   [255:192] reserved                       │
├────────────────────────────────────────────┤
│ Level0 Summary (512 bits = 2 HBM beats)    │
│   Inline in header area for fast access    │
├────────────────────────────────────────────┤
│ Level1 Blocks (compacted)                  │
│   Only blocks where L0[i] == 1             │
│   Each block: 512 bits = 2 HBM beats       │
├────────────────────────────────────────────┤
│ Level2 Blocks (compacted)                  │
│   Only blocks where L1[j] == 1             │
├────────────────────────────────────────────┤
│ Level3 Blocks (leaf data, compacted)       │
└────────────────────────────────────────────┘
```

---

## Build Complexity Estimate

| Component | Effort | Notes |
|-----------|--------|-------|
| FSM redesign | Medium | 20 states vs 14 in V5.3 |
| Cumulative tracking | Low | 3 counters + popcount |
| Address calculation | Low | Shift + add |
| Header parsing | Low | New register fields |
| HBM layout change | Medium | New memory map |
| Host driver update | Medium | New commands, header format |
| Testing | High | 4 depth levels × 3 word sizes |

**Estimated timeline:** 2-3 days for RTL, 1 day for host driver, 1 build cycle.

---

## Recommendation for V6

### Phase 1: Core Changes
1. Add level 3 and 4 FSM states to sparse streaming engine
2. Implement cumulative tracking for all levels
3. Update HBM memory layout with new header format
4. Keep 512-bit word size (proven in V5.3)

### Phase 2: Word Size Flexibility
1. Parameterize word size (512, 1024, 2048)
2. Scale popcount/priority encoder circuits
3. Test with 2K words for extreme domains

### Phase 3: Multiple Engines
1. Instantiate 2-8 parallel intersection engines
2. Arbitrate HBM access across engines
3. Target: 100K+ intersections/sec

---

## Success Metrics

| Metric | V5.3 | V6 Target |
|--------|------|-----------|
| Max domain size | 262K | 68B (260,000× larger) |
| Intersection throughput | ~10K/sec | ~20K/sec |
| Resource usage | 3% | <10% |
| Build time | 52 min | <90 min |

---

*Soli Deo Gloria* ☧
