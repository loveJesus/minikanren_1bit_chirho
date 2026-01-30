# Sparse Hierarchical Domain Storage - Findings ☧

For God so loved the world - John 3:16

---

## Discovery: Implicit Indexing via Popcount

**Date:** 2026-01-30
**Context:** V5.3 sparse streaming success → exploring billion-scale domains

### Key Insight

No explicit pointer tables needed. The summary bits at each level implicitly encode
the index via popcount (rank).

```
offset(bit_i) = popcount(summary & ((1 << i) - 1)) × BLOCK_SIZE
```

This is the **rank** operation from succinct data structures.

---

## Computational Efficiency Analysis

### Per-Level Operations

| Operation | Hardware Cost | Latency |
|-----------|---------------|---------|
| `mask = (1 << i) - 1` | Barrel shifter | 1 cycle |
| `masked = summary & mask` | 512-bit AND | 1 cycle |
| `rank = popcount(masked)` | Adder tree | 3-4 cycles |
| `offset = rank × BLOCK_SIZE` | Shift (power of 2) | 1 cycle |
| **Total per level** | | **~6 cycles** |

For 4-level hierarchy (512⁴ = 68B values): **~24 cycles** to compute any address.

### Comparison to Pointer-Based

| Approach | Memory Overhead | Lookup Latency |
|----------|-----------------|----------------|
| Explicit pointers | 64 bits per child | HBM read (~100ns) per level |
| Implicit (popcount) | 0 | ~6 cycles (~30ns) per level |

**Speedup:** 3-4× faster lookups, zero memory overhead.

### Streaming Efficiency

When iterating through `level0_A & level0_B`:
- We process set bits in order (priority encoder: `find_next_set_bit_chirho`)
- For each bit, rank computation is independent
- **Pipelining:** While computing rank for bit i, can fetch data for bit i-1

---

## Tracking State Across Levels

### The Problem

When descending from level N to level N+1, we need to know:
1. Which child index we're at in level N
2. Cumulative count of children processed before this subtree

### Solution: Cumulative Rank Stack

```
For variable A with 4-level hierarchy:

level0_summary = 0b...01010010  (bits 1, 4, 6 set → 3 children)
                       ↑
                    current_bit = 4, rank_in_level0 = 1

level1_base = A_base + sizeof(level0)
level1[1]_addr = level1_base + 1 × L1_BLOCK_SIZE

level1[1]_summary = 0b...00110001  (bits 0, 4, 5 set → 3 children)
                           ↑
                        current_bit = 4, rank_in_level1 = 1

level2_base = level1_base + popcount(level0) × L1_BLOCK_SIZE
            = level1_base + 3 × L1_BLOCK_SIZE

// But which level2 block corresponds to level1[1]'s child 1?
// Need cumulative children from level1[0] first!

cumulative_before_level1[1] = popcount(level1[0]_summary)  // Must read!
level2[child]_addr = level2_base + (cumulative + 1) × L2_BLOCK_SIZE
```

### The Cumulative Sum Problem

To find a leaf at path (i₀, i₁, i₂, i₃):

```
leaf_offset = Σ(j<i₀) popcount(level1[j]) × L2_SIZE × L3_SIZE
            + Σ(k<i₁) popcount(level2[...][k]) × L3_SIZE
            + ...
```

This requires reading intermediate summaries to compute cumulative counts.

### Three Approaches

#### 1. Streaming Order (Current V5.3)

Process in depth-first order, track cumulative count as we go:

```systemverilog
// State maintained during traversal
logic [31:0] cumulative_l1_children_chirho;  // Children seen so far in level1
logic [31:0] cumulative_l2_children_chirho;  // Children seen so far in level2
logic [31:0] cumulative_l3_children_chirho;  // Children seen so far in level3

// When we finish processing a level1 block:
cumulative_l2_children_chirho += popcount(current_l1_summary_chirho);
```

**Pro:** Simple, matches streaming access pattern.
**Con:** Must process in order; no random access.

#### 2. Precomputed Prefix Sums

Store cumulative child counts at each level:

```
level0_summary: 512 bits
level0_prefix: [0, popcount(l1[0]), popcount(l1[0])+popcount(l1[1]), ...]
```

**Pro:** O(1) random access to any subtree.
**Con:** Extra storage (512 × 32 bits = 2KB per level).

#### 3. Hybrid: Cache Recent Prefix Sums

Small on-chip cache for recently computed prefix sums:

```systemverilog
// Cache: maps (level, block_index) → cumulative_children_before
logic [31:0] prefix_cache_chirho [0:15];  // 16-entry cache
logic [15:0] prefix_cache_valid_chirho;
```

**Pro:** Good for workloads with locality.
**Con:** Cache misses require re-traversal.

---

## Inverse Operation: Index → Value Reconstruction

Given a leaf's storage index, reconstruct the original value.

### The Select Operation

Inverse of rank: `select(summary, i) = position of i-th set bit`

```
summary = 0b01010010
select(summary, 0) = 1  (first set bit at position 1)
select(summary, 1) = 4  (second set bit at position 4)
select(summary, 2) = 6  (third set bit at position 6)
```

### Hardware Implementation

```systemverilog
function automatic logic [8:0] select_512_chirho(
    input logic [511:0] summary_chirho,
    input logic [8:0] rank_chirho
);
    // Priority encoder with rank counter
    logic [8:0] count_chirho = 0;
    for (int i = 0; i < 512; i++) begin
        if (summary_chirho[i]) begin
            if (count_chirho == rank_chirho) return i;
            count_chirho++;
        end
    end
    return 9'h1FF;  // Not found
endfunction
```

For hardware: Use parallel prefix + binary search (log₂(512) = 9 steps).

### Value Reconstruction

```
Given leaf at storage index L in 512⁴ hierarchy:

1. Find which level0 child contains L:
   Scan level0 prefix sums to find i₀ where cumsum[i₀] ≤ L < cumsum[i₀+1]

2. L' = L - cumsum[i₀]  // Offset within this subtree
   Repeat for level1, level2, level3

3. Original value = i₀×512³ + i₁×512² + i₂×512 + i₃
```

This is essentially a **B-tree search** with implicit structure.

---

## RLE Consideration: Encoding 0s vs 1s

### Standard RLE (on 1s)

```
summary = 0b00001111000000001111111100000001
RLE:     [(4,4), (8,8), (1,1)]  // (start, length) pairs
```

Good for runs of consecutive 1s (interval domains).

### Inverted RLE (on 0s)

```
summary = 0b00001111000000001111111100000001
0-runs:  [(0,4), (8,8), (24,7)]  // Where the 0s are
```

Equivalent information, different representation.

### When Each Helps

| Domain Pattern | Best Encoding |
|----------------|---------------|
| Dense clusters: {0..100, 500..600} | RLE on 1s |
| Sparse exclusions: all except {50, 100, 150} | RLE on 0s |
| Random sparse | Neither (use hierarchy) |
| Consecutive values | RLE on 1s (most compact) |

### For miniKanren Domains

Typical patterns:
- **Type domains:** Dense (all integers, all symbols) → RLE on 1s
- **Computed domains:** Sparse, scattered → Hierarchical (no RLE)
- **Range constraints:** Intervals → RLE on 1s
- **Exclusion constraints:** Dense with holes → RLE on 0s

### Hybrid Representation

```
Domain encoding tag (2 bits):
  00 = Dense (raw bits)
  01 = Hierarchical sparse (implicit indexing)
  10 = RLE on 1s (interval list)
  11 = RLE on 0s (exclusion list)
```

---

## Recommended Architecture for V6

### Memory Layout

```
Variable header (64 bytes, one HBM beat):
├── encoding_type: 2 bits
├── num_levels: 3 bits (1-4 for hierarchies)
├── base_word_size: 2 bits (64/128/256/512)
├── total_popcount: 32 bits (for result sizing)
├── level_bases[4]: 4 × 48-bit HBM addresses
└── level0_summary: 512 bits (inline for fast access)
```

### FSM State for Traversal

```systemverilog
typedef struct packed {
    logic [47:0] base_addr_chirho;      // HBM base for this level
    logic [511:0] summary_chirho;       // Current block's summary
    logic [8:0] current_bit_chirho;     // Which bit we're processing
    logic [31:0] cumulative_chirho;     // Children before current subtree
} level_state_t_chirho;

level_state_t_chirho level_stack_chirho [0:3];  // 4 levels max
logic [1:0] current_level_chirho;
```

### Complexity Summary

| Operation | Time | Space |
|-----------|------|-------|
| Lookup (4 levels) | O(1) = ~24 cycles + 4 HBM reads | O(1) registers |
| Streaming intersection | O(k) where k = result size | O(depth) = 4 words |
| Full enumeration | O(n) where n = domain size | O(depth) |
| Random access | O(depth × HBM latency) | O(depth) cache |

---

## Conclusion

The implicit indexing via popcount is:
1. **Computationally efficient:** O(1) per level, pipelineable
2. **Memory efficient:** Zero overhead vs pointer tables
3. **Streaming-friendly:** Natural fit for HBM sequential access

The main complexity is tracking cumulative child counts across levels,
solved by maintaining a small state stack during traversal.

---

*Soli Deo Gloria* ☧
