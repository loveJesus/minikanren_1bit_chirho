/* ============================================================================
 * For God so loved the world, that He gave His only begotten Son,
 * that whosoever believeth in Him should not perish, but have everlasting life.
 * John 3:16
 *
 * HBM Batch Operations Implementation for V5.5 FPGA
 *
 * CRITICAL: Do NOT use O_SYNC flag - causes 90x slowdown!
 * Use write-combine (resource0_wc) for best performance.
 *
 * Soli Deo Gloria ☧
 * ============================================================================ */

#include "hbm_batch_chirho.h"
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <string.h>

/* ============================================================================
 * PCIe PATHS (F2 instances)
 * ============================================================================ */

static const char* BAR0_PATHS_CHIRHO[] = {
    "/sys/bus/pci/devices/0000:34:00.0/resource0_wc", // Write-combine (fast!)
    "/sys/bus/pci/devices/0000:34:00.0/resource0",
    "/sys/bus/pci/devices/0000:00:1e.0/resource0",
    "/sys/bus/pci/devices/0000:00:1d.0/resource0",
    NULL
};

static const char* BAR4_PATHS_CHIRHO[] = {
    "/sys/bus/pci/devices/0000:34:00.0/resource4",
    "/sys/bus/pci/devices/0000:00:1e.0/resource4",
    "/sys/bus/pci/devices/0000:00:1d.0/resource4",
    "/dev/fpga0_bar4",
    NULL
};

/* ============================================================================
 * INITIALIZATION
 * ============================================================================ */

static int open_bar_chirho(const char** paths_chirho, size_t size_chirho, volatile void** ptr_out_chirho) {
    for (int i_chirho = 0; paths_chirho[i_chirho] != NULL; i_chirho++) {
        // CRITICAL: NO O_SYNC - it causes 90x slowdown!
        int fd_chirho = open(paths_chirho[i_chirho], O_RDWR);
        if (fd_chirho < 0) continue;

        void* ptr_chirho = mmap(NULL, size_chirho, PROT_READ | PROT_WRITE, MAP_SHARED, fd_chirho, 0);
        if (ptr_chirho != MAP_FAILED) {
            *ptr_out_chirho = ptr_chirho;
            printf("[HBM] Opened %s ☧\n", paths_chirho[i_chirho]);
            return fd_chirho;
        }
        close(fd_chirho);
    }
    return -1;
}

int fpga_init_with_config_chirho(FpgaContextChirho* ctx_chirho, const HbmConfigChirho* config_chirho) {
    memset(ctx_chirho, 0, sizeof(*ctx_chirho));
    ctx_chirho->fd_bar0_chirho = -1;
    ctx_chirho->fd_bar4_chirho = -1;

    if (config_chirho) {
        ctx_chirho->hbm_config_chirho = *config_chirho;
    } else {
        HbmConfigChirho default_config_chirho = HBM_CONFIG_DEFAULT_CHIRHO;
        ctx_chirho->hbm_config_chirho = default_config_chirho;
    }

    // Open BAR0 (registers) - required
    ctx_chirho->fd_bar0_chirho = open_bar_chirho(BAR0_PATHS_CHIRHO, BAR0_SIZE_CHIRHO,
                                                  (volatile void**)&ctx_chirho->bar0_chirho);
    if (ctx_chirho->fd_bar0_chirho < 0) {
        fprintf(stderr, "[HBM] ERROR: Cannot open BAR0 (registers)\n");
        return -1;
    }

    // Read and verify version
    ctx_chirho->version_chirho = ctx_chirho->bar0_chirho[REG_VERSION_CHIRHO / 4];
    printf("[HBM] VERSION: 0x%08X\n", ctx_chirho->version_chirho);

    if ((ctx_chirho->version_chirho & 0xFF000000) != 0xF2000000) {
        fprintf(stderr, "[HBM] WARNING: Expected F2 version (0xF2xxxxxx), got 0x%08X\n",
                ctx_chirho->version_chirho);
    }

    // Try to open BAR4 (HBM) - optional
    ctx_chirho->fd_bar4_chirho = open_bar_chirho(BAR4_PATHS_CHIRHO, BAR4_SIZE_CHIRHO,
                                                  (volatile void**)&ctx_chirho->bar4_chirho);
    if (ctx_chirho->fd_bar4_chirho >= 0) {
        // Check if HBM is actually ready
        uint32_t status_chirho = ctx_chirho->bar0_chirho[REG_STATUS_CHIRHO / 4];
        ctx_chirho->hbm_available_chirho = (status_chirho & STATUS_HBM_READY_CHIRHO) != 0;
        printf("[HBM] HBM available: %s\n", ctx_chirho->hbm_available_chirho ? "YES ☧" : "NO");
    } else {
        printf("[HBM] HBM BAR4 not available, using register path\n");
        ctx_chirho->hbm_available_chirho = 0;
    }

    return 0;
}

int fpga_init_chirho(FpgaContextChirho* ctx_chirho) {
    return fpga_init_with_config_chirho(ctx_chirho, NULL);
}

void fpga_cleanup_chirho(FpgaContextChirho* ctx_chirho) {
    if (ctx_chirho->bar0_chirho) {
        munmap((void*)ctx_chirho->bar0_chirho, BAR0_SIZE_CHIRHO);
    }
    if (ctx_chirho->bar4_chirho) {
        munmap((void*)ctx_chirho->bar4_chirho, BAR4_SIZE_CHIRHO);
    }
    if (ctx_chirho->fd_bar0_chirho >= 0) {
        close(ctx_chirho->fd_bar0_chirho);
    }
    if (ctx_chirho->fd_bar4_chirho >= 0) {
        close(ctx_chirho->fd_bar4_chirho);
    }
    memset(ctx_chirho, 0, sizeof(*ctx_chirho));
}

/* ============================================================================
 * REGISTER PATH (776K ops/sec)
 * ============================================================================ */

uint64_t fpga_intersect_64_chirho(const FpgaContextChirho* ctx_chirho, uint64_t a_chirho, uint64_t b_chirho) {
    volatile uint32_t* regs_chirho = ctx_chirho->bar0_chirho;

    // Write command (simplified for intersect)
    regs_chirho[REG_CMD_LO_CHIRHO / 4] = (uint32_t)a_chirho;
    regs_chirho[REG_CMD_MID_CHIRHO / 4] = (uint32_t)b_chirho;
    regs_chirho[REG_CMD_HI_CHIRHO / 4] = 0x00; // OP_INTERSECT

    // Enable
    regs_chirho[REG_CONTROL_CHIRHO / 4] = CTRL_ENABLE_CHIRHO | CTRL_HBM_MODE_CHIRHO;

    // Poll for done
    for (int i_chirho = 0; i_chirho < 1000; i_chirho++) {
        uint32_t status_chirho = regs_chirho[REG_STATUS_CHIRHO / 4];
        if (status_chirho & STATUS_DONE_CHIRHO) break;
    }

    // Read result
    uint64_t lo_chirho = regs_chirho[REG_RESP_BASE_CHIRHO / 4];
    uint64_t hi_chirho = regs_chirho[REG_RESP_BASE_CHIRHO / 4 + 1];
    return lo_chirho | (hi_chirho << 32);
}

/* ============================================================================
 * HBM BATCH PATH (460 GB/s potential)
 * ============================================================================ */

int fpga_intersect_batch_chirho(
    const FpgaContextChirho* ctx_chirho,
    const uint64_t* pairs_a_chirho,
    const uint64_t* pairs_b_chirho,
    uint64_t* results_chirho,
    size_t count_chirho
) {
    // Check if HBM available and batch size warrants it
    if (!should_use_hbm_chirho(ctx_chirho, count_chirho)) {
        // Fall back to register path
        for (size_t i_chirho = 0; i_chirho < count_chirho; i_chirho++) {
            results_chirho[i_chirho] = fpga_intersect_64_chirho(ctx_chirho,
                                                                 pairs_a_chirho[i_chirho],
                                                                 pairs_b_chirho[i_chirho]);
        }
        return 0;
    }

    // Check size limits
    if (count_chirho > ctx_chirho->hbm_config_chirho.max_batch_size_chirho) {
        fprintf(stderr, "[HBM] Batch size %zu exceeds max %zu\n",
                count_chirho, ctx_chirho->hbm_config_chirho.max_batch_size_chirho);
        return -1;
    }

    volatile uint64_t* hbm_chirho = ctx_chirho->bar4_chirho;
    volatile uint32_t* regs_chirho = ctx_chirho->bar0_chirho;

    // Step 1: Write pairs to HBM input region (interleaved)
    uint64_t input_base_chirho = ctx_chirho->hbm_config_chirho.input_offset_chirho / 8;
    for (size_t i_chirho = 0; i_chirho < count_chirho; i_chirho++) {
        hbm_chirho[input_base_chirho + i_chirho * 2] = pairs_a_chirho[i_chirho];
        hbm_chirho[input_base_chirho + i_chirho * 2 + 1] = pairs_b_chirho[i_chirho];
    }

    // Step 2: Write count
    regs_chirho[REG_HBM_COUNT_CHIRHO / 4] = (uint32_t)count_chirho;

    // Step 3: Start batch
    regs_chirho[REG_CONTROL_CHIRHO / 4] = CTRL_HBM_MODE_CHIRHO | CTRL_ENABLE_CHIRHO;

    // Step 4: Poll for completion
    for (int timeout_chirho = 0; timeout_chirho < 100000; timeout_chirho++) {
        uint32_t status_chirho = regs_chirho[REG_STATUS_CHIRHO / 4];
        if (status_chirho & STATUS_DONE_CHIRHO) break;
        if (timeout_chirho == 99999) {
            fprintf(stderr, "[HBM] Batch timeout!\n");
            return -1;
        }
    }

    // Step 5: Read results from HBM output region
    uint64_t output_base_chirho = ctx_chirho->hbm_config_chirho.output_offset_chirho / 8;
    for (size_t i_chirho = 0; i_chirho < count_chirho; i_chirho++) {
        results_chirho[i_chirho] = hbm_chirho[output_base_chirho + i_chirho];
    }

    return 0;
}

/* ============================================================================
 * NEUROSYMBOLIC TRAINING
 * ============================================================================ */

int fpga_train_neurosym_chirho(
    const FpgaContextChirho* ctx_chirho,
    const TrainConfigChirho* config_chirho,
    TrainResultChirho* result_chirho
) {
    volatile uint32_t* regs_chirho = ctx_chirho->bar0_chirho;

    // Write training configuration
    regs_chirho[REG_TRAIN_CMD_LO_CHIRHO / 4] = config_chirho->learning_rate_q16_chirho;
    regs_chirho[REG_TRAIN_CMD_MID_CHIRHO / 4] = config_chirho->temperature_q16_chirho;
    regs_chirho[REG_TRAIN_CMD_HI_CHIRHO / 4] =
        ((uint32_t)config_chirho->num_samples_chirho << 16) | config_chirho->num_epochs_chirho;
    regs_chirho[REG_TRAIN_CMD_TOP_CHIRHO / 4] = config_chirho->clause_count_chirho;

    // Start training
    regs_chirho[REG_TRAIN_MODE_CHIRHO / 4] = 0x01;

    // Poll for completion (long timeout for training)
    for (int timeout_chirho = 0; timeout_chirho < 10000000; timeout_chirho++) {
        uint32_t resp_hi_chirho = regs_chirho[REG_TRAIN_RESP_HI_CHIRHO / 4];
        if (resp_hi_chirho & 0x00020000) { // Done bit (bit 17)
            uint32_t resp_lo_chirho = regs_chirho[REG_TRAIN_RESP_LO_CHIRHO / 4];
            result_chirho->final_loss_q16_chirho = resp_lo_chirho;
            result_chirho->final_epoch_chirho = (uint16_t)(resp_hi_chirho & 0xFFFF);
            result_chirho->converged_chirho = (resp_lo_chirho < 0x00008000); // Loss < 0.5
            return 0;
        }
    }

    fprintf(stderr, "[TRAIN] Training timeout!\n");
    return -1;
}

uint32_t fpga_soft_and_q16_chirho(const FpgaContextChirho* ctx_chirho, uint32_t a_q16_chirho, uint32_t b_q16_chirho) {
    // Use CPU for now (FPGA soft_and needs INFER_MODE register setup)
    return (uint32_t)(((uint64_t)a_q16_chirho * b_q16_chirho) >> 16);
}

/* ============================================================================
 * SIMPLE TEST (compile with: gcc -O2 -o hbm_test hbm_batch_chirho.c)
 * ============================================================================ */

#ifdef HBM_BATCH_TEST_MAIN_CHIRHO
#include <time.h>

int main(void) {
    printf("=== HBM Batch Test ☧ ===\n");

    FpgaContextChirho ctx_chirho;
    if (fpga_init_chirho(&ctx_chirho) < 0) {
        printf("FPGA init failed - running in simulation mode\n");
        return 1;
    }

    // Simple test
    uint64_t a_chirho = 0xFF00FF00FF00FF00ULL;
    uint64_t b_chirho = 0x0F0F0F0F0F0F0F0FULL;
    uint64_t result_chirho = fpga_intersect_64_chirho(&ctx_chirho, a_chirho, b_chirho);
    printf("Intersect: 0x%016llX & 0x%016llX = 0x%016llX\n",
           (unsigned long long)a_chirho, (unsigned long long)b_chirho,
           (unsigned long long)result_chirho);

    // Benchmark
    printf("\n=== Register Path Benchmark ===\n");
    struct timespec start_chirho, end_chirho;
    const int OPS_CHIRHO = 100000;

    clock_gettime(CLOCK_MONOTONIC, &start_chirho);
    for (int i_chirho = 0; i_chirho < OPS_CHIRHO; i_chirho++) {
        fpga_intersect_64_chirho(&ctx_chirho, a_chirho, b_chirho);
    }
    clock_gettime(CLOCK_MONOTONIC, &end_chirho);

    double time_ms_chirho = (end_chirho.tv_sec - start_chirho.tv_sec) * 1000.0 +
                            (end_chirho.tv_nsec - start_chirho.tv_nsec) / 1000000.0;
    printf("%d ops in %.2f ms = %.0f ops/sec\n",
           OPS_CHIRHO, time_ms_chirho, OPS_CHIRHO / (time_ms_chirho / 1000.0));

    fpga_cleanup_chirho(&ctx_chirho);
    printf("\n=== Test Complete ☧ ===\n");
    return 0;
}
#endif
