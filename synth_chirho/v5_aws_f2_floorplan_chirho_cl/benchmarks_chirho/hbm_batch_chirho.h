/* ============================================================================
 * For God so loved the world, that He gave His only begotten Son,
 * that whosoever believeth in Him should not perish, but have everlasting life.
 * John 3:16
 *
 * HBM Batch Operations for V5.5 FPGA
 * Provides 460 GB/s HBM bandwidth for bulk domain operations
 *
 * Soli Deo Gloria ☧
 * ============================================================================ */

#ifndef HBM_BATCH_CHIRHO_H
#define HBM_BATCH_CHIRHO_H

#include <stdint.h>
#include <stddef.h>

/* ============================================================================
 * REGISTER MAP (V5.5)
 * ============================================================================ */

// PCIe BAR sizes
#define BAR0_SIZE_CHIRHO        0x4000000ULL   // 64MB registers
#define BAR4_SIZE_CHIRHO        0x200000000ULL // 128GB HBM (on f2.6xlarge)

// Control/Status registers
#define REG_VERSION_CHIRHO      0x00
#define REG_CONTROL_CHIRHO      0x04
#define REG_STATUS_CHIRHO       0x08
#define REG_CMD_LO_CHIRHO       0x10
#define REG_CMD_MID_CHIRHO      0x14
#define REG_CMD_HI_CHIRHO       0x18
#define REG_RESP_BASE_CHIRHO    0x20
#define REG_HBM_COUNT_CHIRHO    0x30

// Control bits
#define CTRL_ENABLE_CHIRHO      0x01
#define CTRL_RESET_CHIRHO       0x02
#define CTRL_HBM_MODE_CHIRHO    0x04

// Status bits
#define STATUS_DONE_CHIRHO      0x01
#define STATUS_VALID_CHIRHO     0x02
#define STATUS_HBM_READY_CHIRHO 0x04

// NeuroSymbolic registers
#define REG_TRAIN_MODE_CHIRHO   0x50
#define REG_TRAIN_CMD_LO_CHIRHO 0x54
#define REG_TRAIN_CMD_MID_CHIRHO 0x58
#define REG_TRAIN_CMD_HI_CHIRHO 0x5C
#define REG_TRAIN_CMD_TOP_CHIRHO 0x60
#define REG_TRAIN_RESP_LO_CHIRHO 0x64
#define REG_TRAIN_RESP_HI_CHIRHO 0x68

// Hierarchical mode register
#define REG_HIER_MODE_CHIRHO    0x80
#define HIER_MODE_FLAT256_CHIRHO   0x00
#define HIER_MODE_HIER_65K_CHIRHO  0x01
#define HIER_MODE_HIER_262K_CHIRHO 0x02

/* ============================================================================
 * HBM BATCH CONFIGURATION
 * ============================================================================ */

typedef struct {
    size_t min_batch_size_chirho;  // Below this, use register path
    size_t max_batch_size_chirho;  // Maximum pairs per batch
    uint64_t input_offset_chirho;  // HBM offset for inputs
    uint64_t output_offset_chirho; // HBM offset for outputs
} HbmConfigChirho;

// Default config (matches Rust)
#define HBM_CONFIG_DEFAULT_CHIRHO { \
    .min_batch_size_chirho = 100,         \
    .max_batch_size_chirho = 1000000,     \
    .input_offset_chirho = 0x00000000ULL, \
    .output_offset_chirho = 0x01000000ULL \
}

/* ============================================================================
 * FPGA CONTEXT
 * ============================================================================ */

typedef struct {
    volatile uint32_t* bar0_chirho;  // Register space (64MB)
    volatile uint64_t* bar4_chirho;  // HBM space (128GB)
    int fd_bar0_chirho;
    int fd_bar4_chirho;
    uint32_t version_chirho;
    int hbm_available_chirho;
    HbmConfigChirho hbm_config_chirho;
} FpgaContextChirho;

/* ============================================================================
 * INITIALIZATION
 * ============================================================================ */

/**
 * Initialize FPGA context
 *
 * @param ctx_chirho Context to initialize
 * @return 0 on success, -1 on error
 *
 * Opens BAR0 (registers) and optionally BAR4 (HBM).
 * Uses write-combine for best performance (no O_SYNC!).
 */
int fpga_init_chirho(FpgaContextChirho* ctx_chirho);

/**
 * Initialize with custom HBM config
 */
int fpga_init_with_config_chirho(FpgaContextChirho* ctx_chirho, const HbmConfigChirho* config_chirho);

/**
 * Cleanup FPGA context
 */
void fpga_cleanup_chirho(FpgaContextChirho* ctx_chirho);

/* ============================================================================
 * REGISTER OPERATIONS (776K ops/sec)
 * ============================================================================ */

/**
 * Read 32-bit register
 */
static inline uint32_t fpga_reg_read_chirho(const FpgaContextChirho* ctx_chirho, uint32_t offset_chirho) {
    return ctx_chirho->bar0_chirho[offset_chirho / 4];
}

/**
 * Write 32-bit register
 */
static inline void fpga_reg_write_chirho(const FpgaContextChirho* ctx_chirho, uint32_t offset_chirho, uint32_t value_chirho) {
    ctx_chirho->bar0_chirho[offset_chirho / 4] = value_chirho;
}

/**
 * Single 64-bit domain intersection
 */
uint64_t fpga_intersect_64_chirho(const FpgaContextChirho* ctx_chirho, uint64_t a_chirho, uint64_t b_chirho);

/* ============================================================================
 * HBM BATCH OPERATIONS (460 GB/s potential)
 * ============================================================================ */

/**
 * Batch intersect via HBM
 *
 * @param ctx_chirho FPGA context
 * @param pairs_a_chirho Array of first domains
 * @param pairs_b_chirho Array of second domains
 * @param results_chirho Output array (must be pre-allocated)
 * @param count_chirho Number of pairs
 * @return 0 on success, -1 on error
 *
 * Protocol:
 * 1. Write pairs to HBM input region (interleaved: a0, b0, a1, b1, ...)
 * 2. Write count to REG_HBM_COUNT
 * 3. Set CTRL_HBM_MODE | CTRL_ENABLE
 * 4. Poll STATUS_DONE
 * 5. Read results from HBM output region
 */
int fpga_intersect_batch_chirho(
    const FpgaContextChirho* ctx_chirho,
    const uint64_t* pairs_a_chirho,
    const uint64_t* pairs_b_chirho,
    uint64_t* results_chirho,
    size_t count_chirho
);

/**
 * Check if HBM batch should be used for this size
 */
static inline int should_use_hbm_chirho(const FpgaContextChirho* ctx_chirho, size_t batch_size_chirho) {
    return ctx_chirho->hbm_available_chirho &&
           batch_size_chirho >= ctx_chirho->hbm_config_chirho.min_batch_size_chirho;
}

/* ============================================================================
 * NEUROSYMBOLIC TRAINING
 * ============================================================================ */

typedef struct {
    uint32_t learning_rate_q16_chirho;  // Q16.16 (0x00001999 = 0.1)
    uint32_t temperature_q16_chirho;    // Q16.16 (0x00020000 = 2.0)
    uint16_t num_epochs_chirho;
    uint16_t num_samples_chirho;
    uint32_t clause_count_chirho;
} TrainConfigChirho;

typedef struct {
    uint32_t final_loss_q16_chirho;
    uint16_t final_epoch_chirho;
    int converged_chirho;
} TrainResultChirho;

/**
 * Execute on-chip neurosymbolic training
 *
 * Uses Gumbel-softmax reparameterization for differentiable discrete optimization.
 */
int fpga_train_neurosym_chirho(
    const FpgaContextChirho* ctx_chirho,
    const TrainConfigChirho* config_chirho,
    TrainResultChirho* result_chirho
);

/**
 * Soft AND: P(A ∧ B) = P(A) × P(B) in Q16.16
 */
uint32_t fpga_soft_and_q16_chirho(const FpgaContextChirho* ctx_chirho, uint32_t a_q16_chirho, uint32_t b_q16_chirho);

#endif /* HBM_BATCH_CHIRHO_H */
