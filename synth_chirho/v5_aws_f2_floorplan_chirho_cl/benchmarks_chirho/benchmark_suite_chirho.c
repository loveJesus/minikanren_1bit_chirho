// ============================================================================
// For God so loved the world - John 3:16 ☧
// Comprehensive Benchmark Suite for miniKanren FPGA
// v4: Hierarchical Domains (65K-134M) + Neurosymbolic
// ============================================================================

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include <math.h>

// AWS FPGA includes
#include <fpga_pci.h>
#include <fpga_mgmt.h>

// Register addresses
#define REG_VERSION_CHIRHO      0x00
#define REG_CONTROL_CHIRHO      0x04
#define REG_STATUS_CHIRHO       0x08
#define REG_CMD_LO_CHIRHO       0x10
#define REG_CMD_MID_CHIRHO      0x14
#define REG_CMD_HI_CHIRHO       0x18
#define REG_RESP_BASE_CHIRHO    0x20
#define REG_HIER_MODE_CHIRHO    0x40
#define REG_HIER_LEVEL_CHIRHO   0x44
#define REG_BEAT_COUNT_CHIRHO   0x48
#define REG_TRAIN_MODE_CHIRHO   0x50
#define REG_TRAIN_CMD_LO_CHIRHO 0x54
#define REG_TRAIN_CMD_MID_CHIRHO 0x58
#define REG_TRAIN_CMD_HI_CHIRHO 0x5C
#define REG_TRAIN_CMD_TOP_CHIRHO 0x60
#define REG_TRAIN_RESP_LO_CHIRHO 0x64
#define REG_TRAIN_RESP_HI_CHIRHO 0x68
#define REG_INFER_MODE_CHIRHO   0x70

// Control bits
#define CTRL_ENABLE_CHIRHO      (1 << 0)
#define CTRL_RESET_CHIRHO       (1 << 1)
#define CTRL_HBM_MODE_CHIRHO    (1 << 2)

// Status bits
#define STATUS_DONE_CHIRHO      (1 << 0)
#define STATUS_VALID_CHIRHO     (1 << 1)
#define STATUS_HBM_READY_CHIRHO (1 << 2)

// Training mode bits
#define TRAIN_ENABLE_CHIRHO     (1 << 0)
#define TRAIN_RESET_CHIRHO      (1 << 1)

// Hierarchical modes
typedef enum {
    HIER_64_CHIRHO = 0,      // 64 values
    HIER_65K_CHIRHO = 1,     // 256² = 65,536
    HIER_262K_CHIRHO = 2,    // 512² = 262,144
    HIER_16M_CHIRHO = 3,     // 256³ = 16,777,216
    HIER_134M_CHIRHO = 4     // 512³ = 134,217,728
} hier_mode_chirho_t;

// Commands
#define CMD_NOP_CHIRHO          0x00
#define CMD_INTERSECT_CHIRHO    0x01
#define CMD_UNION_CHIRHO        0x02
#define CMD_CONTAINS_CHIRHO     0x03
#define CMD_COUNT_CHIRHO        0x04
#define CMD_INIT_CHIRHO         0x10
#define CMD_LOAD_L0_CHIRHO      0x20
#define CMD_LOAD_L1_CHIRHO      0x21
#define CMD_LOAD_L2_CHIRHO      0x22

static pci_bar_handle_t bar0_chirho = PCI_BAR_HANDLE_INIT;
static pci_bar_handle_t bar4_chirho = PCI_BAR_HANDLE_INIT;  // HBM access

// ============================================================================
// Utility functions
// ============================================================================

static inline int reg_read_chirho(uint32_t addr, uint32_t *val) {
    return fpga_pci_peek(bar0_chirho, addr, val);
}

static inline int reg_write_chirho(uint32_t addr, uint32_t val) {
    return fpga_pci_poke(bar0_chirho, addr, val);
}

static void wait_done_chirho(void) {
    uint32_t status;
    do {
        reg_read_chirho(REG_STATUS_CHIRHO, &status);
    } while (!(status & STATUS_DONE_CHIRHO));
}

static double get_time_ns_chirho(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e9 + ts.tv_nsec;
}

static const char* hier_mode_name_chirho(hier_mode_chirho_t mode) {
    static const char* names[] = {
        "64 (BitVec64)",
        "65K (256²)",
        "262K (512²)",
        "16.7M (256³)",
        "134M (512³)"
    };
    return (mode <= HIER_134M_CHIRHO) ? names[mode] : "Unknown";
}

static uint64_t hier_mode_size_chirho(hier_mode_chirho_t mode) {
    static const uint64_t sizes[] = {64, 65536, 262144, 16777216, 134217728};
    return (mode <= HIER_134M_CHIRHO) ? sizes[mode] : 0;
}

// ============================================================================
// BENCHMARK 1: Hierarchical Domain Intersection (realistic sizes)
// ============================================================================

typedef struct {
    double total_ns;
    double per_op_ns;
    double ops_per_sec;
    uint64_t domain_size;
    int iterations;
} bench_result_chirho_t;

static bench_result_chirho_t bench_intersect_chirho(hier_mode_chirho_t mode, int iterations) {
    bench_result_chirho_t result = {0};
    result.domain_size = hier_mode_size_chirho(mode);
    result.iterations = iterations;

    // Set mode
    reg_write_chirho(REG_HIER_MODE_CHIRHO, mode);

    // Enable HBM for large domains
    if (mode >= HIER_16M_CHIRHO) {
        reg_write_chirho(REG_CONTROL_CHIRHO, CTRL_ENABLE_CHIRHO | CTRL_HBM_MODE_CHIRHO);
        uint32_t status;
        int timeout = 500;
        do {
            reg_read_chirho(REG_STATUS_CHIRHO, &status);
            usleep(10000);
        } while (!(status & STATUS_HBM_READY_CHIRHO) && --timeout > 0);
    }

    // Warm up
    for (int i = 0; i < 10; i++) {
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_INTERSECT_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, 0xAAAAAAAA);
        reg_write_chirho(REG_CMD_HI_CHIRHO, 0x55555555);
        wait_done_chirho();
    }

    // Benchmark: intersect two sparse domains
    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Simulate realistic sparse patterns
        uint32_t pattern_a = (i * 0x12345678) ^ 0xDEADBEEF;
        uint32_t pattern_b = (i * 0x87654321) ^ 0xCAFEBABE;

        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_INTERSECT_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, pattern_a);
        reg_write_chirho(REG_CMD_HI_CHIRHO, pattern_b);
        wait_done_chirho();
    }
    double end = get_time_ns_chirho();

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

// ============================================================================
// BENCHMARK 2: Multi-level Hierarchical Traversal
// Tests the streaming FSM for 3-level hierarchies
// ============================================================================

static bench_result_chirho_t bench_hier_traversal_chirho(hier_mode_chirho_t mode, int iterations) {
    bench_result_chirho_t result = {0};
    result.domain_size = hier_mode_size_chirho(mode);
    result.iterations = iterations;

    reg_write_chirho(REG_HIER_MODE_CHIRHO, mode);

    if (mode >= HIER_16M_CHIRHO) {
        reg_write_chirho(REG_CONTROL_CHIRHO, CTRL_ENABLE_CHIRHO | CTRL_HBM_MODE_CHIRHO);
        uint32_t status;
        int timeout = 500;
        do {
            reg_read_chirho(REG_STATUS_CHIRHO, &status);
            usleep(10000);
        } while (!(status & STATUS_HBM_READY_CHIRHO) && --timeout > 0);
    }

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Load Level 0 (top-level sparse mask)
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_LOAD_L0_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, 0x0000FFFF);  // First 16 blocks active
        wait_done_chirho();

        // Load Level 1 for each active L0 block
        for (int j = 0; j < 16; j++) {
            reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_LOAD_L1_CHIRHO | (j << 8));
            reg_write_chirho(REG_CMD_MID_CHIRHO, 0xAAAAAAAA);  // Sparse pattern
            wait_done_chirho();
        }

        // Do intersection at leaf level
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_INTERSECT_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, 0x12345678);
        reg_write_chirho(REG_CMD_HI_CHIRHO, 0x87654321);
        wait_done_chirho();
    }
    double end = get_time_ns_chirho();

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

// ============================================================================
// BENCHMARK 3: Neurosymbolic Training (weight updates)
// Tests the Q16.16 fixed-point training pipeline
// ============================================================================

static bench_result_chirho_t bench_train_chirho(int iterations, int batch_size) {
    bench_result_chirho_t result = {0};
    result.iterations = iterations;

    // Enable training mode
    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, TRAIN_ENABLE_CHIRHO);

    // Initialize weights (simulated from HBM)
    reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, 0x00010000);   // weight = 1.0 in Q16.16
    reg_write_chirho(REG_TRAIN_CMD_MID_CHIRHO, 0x00008000);  // learning_rate = 0.5
    reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, batch_size);
    wait_done_chirho();

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Training step: forward pass + backward pass + weight update
        // Simulate realistic gradient values
        uint32_t gradient = (uint32_t)(sin(i * 0.1) * 0x1000) & 0xFFFF;

        reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, 0x80000000 | gradient);  // Train command
        reg_write_chirho(REG_TRAIN_CMD_MID_CHIRHO, i % batch_size);        // Sample index
        reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, 0x00000001);             // Feature ID
        wait_done_chirho();
    }
    double end = get_time_ns_chirho();

    // Read final weights
    uint32_t final_weight_lo, final_weight_hi;
    reg_read_chirho(REG_TRAIN_RESP_LO_CHIRHO, &final_weight_lo);
    reg_read_chirho(REG_TRAIN_RESP_HI_CHIRHO, &final_weight_hi);

    printf("  Final weight: 0x%08X (Q16.16: %.4f)\n",
           final_weight_lo, (double)final_weight_lo / 65536.0);

    // Disable training mode
    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, 0);

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

// ============================================================================
// BENCHMARK 4: Neurosymbolic Inference (probabilistic queries)
// Tests the Q8.8 inference pipeline with soft AND/OR
// ============================================================================

static bench_result_chirho_t bench_infer_chirho(int iterations) {
    bench_result_chirho_t result = {0};
    result.iterations = iterations;

    // Enable probabilistic inference mode
    reg_write_chirho(REG_INFER_MODE_CHIRHO, 0x01);

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Probabilistic query: soft_and(prob_a, prob_b)
        // prob_a, prob_b in Q8.8 format (0x0100 = 1.0)
        uint32_t prob_a = (uint32_t)(0.7 * 256) & 0xFF;  // 0.7
        uint32_t prob_b = (uint32_t)(0.8 * 256) & 0xFF;  // 0.8

        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_INTERSECT_CHIRHO);  // Soft AND
        reg_write_chirho(REG_CMD_MID_CHIRHO, prob_a | (prob_b << 16));
        wait_done_chirho();
    }
    double end = get_time_ns_chirho();

    // Disable probabilistic mode
    reg_write_chirho(REG_INFER_MODE_CHIRHO, 0x00);

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

// ============================================================================
// BENCHMARK 5: Realistic Query Pattern (miniKanren-style search)
// Simulates appendo-like relational queries
// ============================================================================

static bench_result_chirho_t bench_relational_query_chirho(hier_mode_chirho_t mode, int iterations) {
    bench_result_chirho_t result = {0};
    result.domain_size = hier_mode_size_chirho(mode);
    result.iterations = iterations;

    reg_write_chirho(REG_HIER_MODE_CHIRHO, mode);

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Simulate: (appendo A B X), (appendo X C [known])
        // Step 1: Initialize domains for A, B, X, C
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_INIT_CHIRHO);
        wait_done_chirho();

        // Step 2: Apply constraint from known output
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_INTERSECT_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, 0x0000FFFF);  // Known output constrains X
        wait_done_chirho();

        // Step 3: Propagate constraint backward to A, B
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_INTERSECT_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, 0xFFFF0000);  // Backward propagation
        wait_done_chirho();

        // Step 4: Check for solutions (count non-empty intersections)
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_COUNT_CHIRHO);
        wait_done_chirho();
    }
    double end = get_time_ns_chirho();

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

// ============================================================================
// BENCHMARK 6: 256-bit Wide Word Operations
// Tests the 256-bit datapath (4x 64-bit words)
// ============================================================================

// Register addresses for wide operations
#define REG_WIDE_DATA_0_CHIRHO  0x80  // bits [63:0]
#define REG_WIDE_DATA_1_CHIRHO  0x84  // bits [127:64]
#define REG_WIDE_DATA_2_CHIRHO  0x88  // bits [191:128]
#define REG_WIDE_DATA_3_CHIRHO  0x8C  // bits [255:192]
#define REG_WIDE_DATA_4_CHIRHO  0x90  // bits [319:256] (for 512-bit)
#define REG_WIDE_DATA_5_CHIRHO  0x94  // bits [383:320]
#define REG_WIDE_DATA_6_CHIRHO  0x98  // bits [447:384]
#define REG_WIDE_DATA_7_CHIRHO  0x9C  // bits [511:448]
#define REG_WIDE_CMD_CHIRHO     0xA0  // Wide operation command
#define REG_WIDE_RESULT_CHIRHO  0xA4  // Result (popcount, etc.)

#define WIDE_CMD_AND_256_CHIRHO     0x01
#define WIDE_CMD_OR_256_CHIRHO      0x02
#define WIDE_CMD_POPCOUNT_256_CHIRHO 0x03
#define WIDE_CMD_AND_512_CHIRHO     0x11
#define WIDE_CMD_OR_512_CHIRHO      0x12
#define WIDE_CMD_POPCOUNT_512_CHIRHO 0x13

static bench_result_chirho_t bench_256bit_and_chirho(int iterations) {
    bench_result_chirho_t result = {0};
    result.domain_size = 256;
    result.iterations = iterations;

    // Set mode to 256² which uses 256-bit words
    reg_write_chirho(REG_HIER_MODE_CHIRHO, HIER_65K_CHIRHO);

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Load operand A (256 bits = 4 x 32-bit writes)
        uint32_t pattern_a = 0xAAAAAAAA ^ (i * 0x12345678);
        reg_write_chirho(REG_WIDE_DATA_0_CHIRHO, pattern_a);
        reg_write_chirho(REG_WIDE_DATA_1_CHIRHO, ~pattern_a);
        reg_write_chirho(REG_WIDE_DATA_2_CHIRHO, pattern_a >> 1);
        reg_write_chirho(REG_WIDE_DATA_3_CHIRHO, ~(pattern_a >> 1));

        // Execute 256-bit AND with operand B (implicit from hierarchy)
        reg_write_chirho(REG_WIDE_CMD_CHIRHO, WIDE_CMD_AND_256_CHIRHO);
        wait_done_chirho();
    }
    double end = get_time_ns_chirho();

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

static bench_result_chirho_t bench_256bit_popcount_chirho(int iterations) {
    bench_result_chirho_t result = {0};
    result.domain_size = 256;
    result.iterations = iterations;

    reg_write_chirho(REG_HIER_MODE_CHIRHO, HIER_65K_CHIRHO);

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Load sparse pattern (varying density)
        uint32_t density = (i % 64) + 1;  // 1-64 bits set per word
        uint32_t pattern = (1UL << density) - 1;

        reg_write_chirho(REG_WIDE_DATA_0_CHIRHO, pattern);
        reg_write_chirho(REG_WIDE_DATA_1_CHIRHO, pattern << 1);
        reg_write_chirho(REG_WIDE_DATA_2_CHIRHO, pattern << 2);
        reg_write_chirho(REG_WIDE_DATA_3_CHIRHO, pattern << 3);

        // Execute 256-bit popcount
        reg_write_chirho(REG_WIDE_CMD_CHIRHO, WIDE_CMD_POPCOUNT_256_CHIRHO);
        wait_done_chirho();

        // Read result (optional, for verification)
        uint32_t count;
        reg_read_chirho(REG_WIDE_RESULT_CHIRHO, &count);
    }
    double end = get_time_ns_chirho();

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

// ============================================================================
// BENCHMARK 7: 512-bit Wide Word Operations
// Tests the 512-bit datapath (8x 64-bit words)
// ============================================================================

static bench_result_chirho_t bench_512bit_and_chirho(int iterations) {
    bench_result_chirho_t result = {0};
    result.domain_size = 512;
    result.iterations = iterations;

    // Set mode to 512² which uses 512-bit words
    reg_write_chirho(REG_HIER_MODE_CHIRHO, HIER_262K_CHIRHO);

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Load operand A (512 bits = 8 x 32-bit writes)
        uint32_t pattern_a = 0x55555555 ^ (i * 0x87654321);
        reg_write_chirho(REG_WIDE_DATA_0_CHIRHO, pattern_a);
        reg_write_chirho(REG_WIDE_DATA_1_CHIRHO, ~pattern_a);
        reg_write_chirho(REG_WIDE_DATA_2_CHIRHO, pattern_a >> 1);
        reg_write_chirho(REG_WIDE_DATA_3_CHIRHO, ~(pattern_a >> 1));
        reg_write_chirho(REG_WIDE_DATA_4_CHIRHO, pattern_a << 1);
        reg_write_chirho(REG_WIDE_DATA_5_CHIRHO, ~(pattern_a << 1));
        reg_write_chirho(REG_WIDE_DATA_6_CHIRHO, pattern_a ^ 0xF0F0F0F0);
        reg_write_chirho(REG_WIDE_DATA_7_CHIRHO, pattern_a ^ 0x0F0F0F0F);

        // Execute 512-bit AND
        reg_write_chirho(REG_WIDE_CMD_CHIRHO, WIDE_CMD_AND_512_CHIRHO);
        wait_done_chirho();
    }
    double end = get_time_ns_chirho();

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

static bench_result_chirho_t bench_512bit_popcount_chirho(int iterations) {
    bench_result_chirho_t result = {0};
    result.domain_size = 512;
    result.iterations = iterations;

    reg_write_chirho(REG_HIER_MODE_CHIRHO, HIER_262K_CHIRHO);

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Load varying density pattern
        uint32_t density = (i % 32) + 1;
        uint32_t pattern = (1UL << density) - 1;

        reg_write_chirho(REG_WIDE_DATA_0_CHIRHO, pattern);
        reg_write_chirho(REG_WIDE_DATA_1_CHIRHO, pattern << 1);
        reg_write_chirho(REG_WIDE_DATA_2_CHIRHO, pattern << 2);
        reg_write_chirho(REG_WIDE_DATA_3_CHIRHO, pattern << 3);
        reg_write_chirho(REG_WIDE_DATA_4_CHIRHO, pattern << 4);
        reg_write_chirho(REG_WIDE_DATA_5_CHIRHO, pattern << 5);
        reg_write_chirho(REG_WIDE_DATA_6_CHIRHO, pattern << 6);
        reg_write_chirho(REG_WIDE_DATA_7_CHIRHO, pattern << 7);

        // Execute 512-bit popcount
        reg_write_chirho(REG_WIDE_CMD_CHIRHO, WIDE_CMD_POPCOUNT_512_CHIRHO);
        wait_done_chirho();

        uint32_t count;
        reg_read_chirho(REG_WIDE_RESULT_CHIRHO, &count);
    }
    double end = get_time_ns_chirho();

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

// ============================================================================
// BENCHMARK 8: Wide Word Sparse Iteration
// Tests iterating through set bits in wide words
// ============================================================================

static bench_result_chirho_t bench_256bit_iterate_chirho(int iterations) {
    bench_result_chirho_t result = {0};
    result.domain_size = 256;
    result.iterations = iterations;

    reg_write_chirho(REG_HIER_MODE_CHIRHO, HIER_65K_CHIRHO);

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Load sparse pattern (~10% density)
        reg_write_chirho(REG_WIDE_DATA_0_CHIRHO, 0x11111111);
        reg_write_chirho(REG_WIDE_DATA_1_CHIRHO, 0x22222222);
        reg_write_chirho(REG_WIDE_DATA_2_CHIRHO, 0x44444444);
        reg_write_chirho(REG_WIDE_DATA_3_CHIRHO, 0x88888888);

        // Iterate: get each set bit position
        reg_write_chirho(REG_CMD_LO_CHIRHO, 0x30);  // CMD_ITERATE
        do {
            wait_done_chirho();
            uint32_t status;
            reg_read_chirho(REG_STATUS_CHIRHO, &status);
            if (!(status & STATUS_VALID_CHIRHO)) break;  // No more bits
            // Read bit position (would be used in real query)
            uint32_t pos;
            reg_read_chirho(REG_RESP_BASE_CHIRHO, &pos);
        } while (1);
    }
    double end = get_time_ns_chirho();

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

static bench_result_chirho_t bench_512bit_iterate_chirho(int iterations) {
    bench_result_chirho_t result = {0};
    result.domain_size = 512;
    result.iterations = iterations;

    reg_write_chirho(REG_HIER_MODE_CHIRHO, HIER_262K_CHIRHO);

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Load sparse pattern
        for (int j = 0; j < 8; j++) {
            reg_write_chirho(REG_WIDE_DATA_0_CHIRHO + j * 4, 0x01010101 << j);
        }

        reg_write_chirho(REG_CMD_LO_CHIRHO, 0x30);  // CMD_ITERATE
        do {
            wait_done_chirho();
            uint32_t status;
            reg_read_chirho(REG_STATUS_CHIRHO, &status);
            if (!(status & STATUS_VALID_CHIRHO)) break;
            uint32_t pos;
            reg_read_chirho(REG_RESP_BASE_CHIRHO, &pos);
        } while (1);
    }
    double end = get_time_ns_chirho();

    result.total_ns = end - start;
    result.per_op_ns = result.total_ns / iterations;
    result.ops_per_sec = 1e9 / result.per_op_ns;

    return result;
}

// ============================================================================
// Print results
// ============================================================================

static void print_result_chirho(const char* name, bench_result_chirho_t* r) {
    printf("%-35s | %12lu | %8d | %10.1f | %12.0f\n",
           name, r->domain_size, r->iterations, r->per_op_ns, r->ops_per_sec);
}

// ============================================================================
// Main
// ============================================================================

int main(int argc, char **argv) {
    (void)argc; (void)argv;
    int rc;
    bench_result_chirho_t result;

    printf("============================================================\n");
    printf("miniKanren FPGA Benchmark Suite ☧\n");
    printf("v4: Hierarchical Domains + Neurosymbolic\n");
    printf("For God so loved the world - John 3:16\n");
    printf("============================================================\n\n");

    // Initialize
    rc = fpga_mgmt_init();
    if (rc) { printf("ERROR: fpga_mgmt_init: %d\n", rc); return 1; }

    rc = fpga_pci_attach(0, FPGA_APP_PF, APP_PF_BAR0, 0, &bar0_chirho);
    if (rc) { printf("ERROR: fpga_pci_attach BAR0: %d\n", rc); return 1; }

    rc = fpga_pci_attach(0, FPGA_APP_PF, APP_PF_BAR4, 0, &bar4_chirho);
    if (rc) { printf("WARN: fpga_pci_attach BAR4: %d (HBM may not work)\n", rc); }

    // Reset
    reg_write_chirho(REG_CONTROL_CHIRHO, CTRL_RESET_CHIRHO);
    usleep(10000);
    reg_write_chirho(REG_CONTROL_CHIRHO, CTRL_ENABLE_CHIRHO);

    printf("%-35s | %12s | %8s | %10s | %12s\n",
           "Benchmark", "Domain Size", "Iters", "ns/op", "ops/sec");
    printf("------------------------------------+-------------+----------+------------+-------------\n");

    // BENCHMARK 1: Hierarchical Intersection at various scales
    printf("\n=== Hierarchical Domain Intersection ===\n");

    result = bench_intersect_chirho(HIER_64_CHIRHO, 100000);
    print_result_chirho("Intersect 64 (BitVec64)", &result);

    result = bench_intersect_chirho(HIER_65K_CHIRHO, 10000);
    print_result_chirho("Intersect 65K (256²)", &result);

    result = bench_intersect_chirho(HIER_262K_CHIRHO, 10000);
    print_result_chirho("Intersect 262K (512²)", &result);

    result = bench_intersect_chirho(HIER_16M_CHIRHO, 1000);
    print_result_chirho("Intersect 16.7M (256³) [HBM]", &result);

    result = bench_intersect_chirho(HIER_134M_CHIRHO, 100);
    print_result_chirho("Intersect 134M (512³) [HBM]", &result);

    // BENCHMARK 2: Hierarchical Traversal
    printf("\n=== Hierarchical Traversal (streaming FSM) ===\n");

    result = bench_hier_traversal_chirho(HIER_65K_CHIRHO, 1000);
    print_result_chirho("Traversal 65K (2-level)", &result);

    result = bench_hier_traversal_chirho(HIER_16M_CHIRHO, 100);
    print_result_chirho("Traversal 16.7M (3-level) [HBM]", &result);

    // BENCHMARK 3: Neurosymbolic Training
    printf("\n=== Neurosymbolic Training (Q16.16) ===\n");

    result = bench_train_chirho(10000, 32);
    print_result_chirho("Train batch=32", &result);

    result = bench_train_chirho(10000, 128);
    print_result_chirho("Train batch=128", &result);

    // BENCHMARK 4: Neurosymbolic Inference
    printf("\n=== Neurosymbolic Inference (Q8.8 soft logic) ===\n");

    result = bench_infer_chirho(100000);
    print_result_chirho("Soft AND inference", &result);

    // BENCHMARK 5: Relational Queries
    printf("\n=== Relational Query Patterns ===\n");

    result = bench_relational_query_chirho(HIER_64_CHIRHO, 10000);
    print_result_chirho("Relational 64", &result);

    result = bench_relational_query_chirho(HIER_65K_CHIRHO, 1000);
    print_result_chirho("Relational 65K", &result);

    result = bench_relational_query_chirho(HIER_16M_CHIRHO, 100);
    print_result_chirho("Relational 16.7M [HBM]", &result);

    // BENCHMARK 6: 256-bit Wide Word Operations
    printf("\n=== 256-bit Wide Word Operations ===\n");

    result = bench_256bit_and_chirho(100000);
    print_result_chirho("256-bit AND", &result);

    result = bench_256bit_popcount_chirho(100000);
    print_result_chirho("256-bit Popcount", &result);

    result = bench_256bit_iterate_chirho(10000);
    print_result_chirho("256-bit Sparse Iterate", &result);

    // BENCHMARK 7: 512-bit Wide Word Operations
    printf("\n=== 512-bit Wide Word Operations ===\n");

    result = bench_512bit_and_chirho(100000);
    print_result_chirho("512-bit AND", &result);

    result = bench_512bit_popcount_chirho(100000);
    print_result_chirho("512-bit Popcount", &result);

    result = bench_512bit_iterate_chirho(10000);
    print_result_chirho("512-bit Sparse Iterate", &result);

    printf("\n============================================================\n");
    printf("Benchmarks complete. Soli Deo Gloria ☧\n");
    printf("============================================================\n");

    fpga_pci_detach(bar0_chirho);
    fpga_pci_detach(bar4_chirho);
    fpga_mgmt_close();

    return 0;
}
