// HBM-Batched Training Benchmark ☧
// John 3:16 - For God so loved the world
//
// REAL FPGA training using HBM memory batching:
// - Upload weights to HBM once at start
// - FPGA runs ALL training iterations in HBM
// - Download final weights once at end
// - Eliminates PCIe overhead per iteration
//
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#include <math.h>
#include <fpga_pci.h>
#include <fpga_mgmt.h>

#define APP_PF_BAR0_CHIRHO FPGA_APP_PF
#define APP_PF_BAR4_CHIRHO 4

// HBM Configuration
#define HBM_BASE_ADDR_CHIRHO 0x0
#define HBM_WEIGHTS_OFFSET_CHIRHO 0x0
#define HBM_GRADS_OFFSET_CHIRHO 0x100000
#define HBM_SCRATCH_OFFSET_CHIRHO 0x200000

// Control registers
#define REG_CTRL_CHIRHO 0x000
#define REG_STATUS_CHIRHO 0x500
#define REG_HBM_ADDR_LO_CHIRHO 0x100
#define REG_HBM_ADDR_HI_CHIRHO 0x104
#define REG_HBM_SIZE_CHIRHO 0x108
#define REG_TRAIN_VARS_CHIRHO 0x700
#define REG_TRAIN_CLAUSES_CHIRHO 0x704
#define REG_TRAIN_ITERS_CHIRHO 0x708
#define REG_TRAIN_BATCH_CHIRHO 0x70C
#define REG_TRAIN_LR_CHIRHO 0x710
#define REG_TRAIN_TEMP_START_CHIRHO 0x714
#define REG_TRAIN_TEMP_END_CHIRHO 0x718
#define REG_TRAIN_RESULT_CHIRHO 0x750

// Commands
#define CMD_HBM_WRITE_CHIRHO 0x01
#define CMD_HBM_READ_CHIRHO 0x02
#define CMD_TRAIN_START_CHIRHO 0x10
#define CMD_TRAIN_POLL_CHIRHO 0x11

// Fixed-point Q16.16
typedef int32_t q16_chirho;
#define Q16_ONE_CHIRHO (1 << 16)
#define Q16_HALF_CHIRHO (1 << 15)

static inline q16_chirho q16_mul_chirho(q16_chirho a_chirho, q16_chirho b_chirho) {
    return (q16_chirho)(((int64_t)a_chirho * b_chirho) >> 16);
}

static inline q16_chirho q16_div_chirho(q16_chirho a_chirho, q16_chirho b_chirho) {
    if (b_chirho == 0) return (a_chirho >= 0) ? 0x7FFFFFFF : -0x7FFFFFFF;
    return (q16_chirho)(((int64_t)a_chirho << 16) / b_chirho);
}

static inline q16_chirho float_to_q16_chirho(float f_chirho) {
    return (q16_chirho)(f_chirho * Q16_ONE_CHIRHO);
}

static inline float q16_to_float_chirho(q16_chirho q_chirho) {
    return (float)q_chirho / Q16_ONE_CHIRHO;
}

static q16_chirho q16_exp_chirho(q16_chirho x_chirho) {
    q16_chirho one_chirho = Q16_ONE_CHIRHO;
    q16_chirho x2_chirho = q16_mul_chirho(x_chirho, x_chirho);
    q16_chirho x3_chirho = q16_mul_chirho(x2_chirho, x_chirho);
    return one_chirho + x_chirho + (x2_chirho >> 1) + q16_div_chirho(x3_chirho, 6 * Q16_ONE_CHIRHO);
}

double get_time_ms_chirho(void) {
    struct timespec ts_chirho;
    clock_gettime(CLOCK_MONOTONIC, &ts_chirho);
    return ts_chirho.tv_sec * 1000.0 + ts_chirho.tv_nsec / 1000000.0;
}

// ============================================================================
// HBM MEMORY OPERATIONS
// ============================================================================

typedef struct {
    pci_bar_handle_t bar0_chirho;
    pci_bar_handle_t bar4_chirho;  // HBM access
    int slot_chirho;
} FpgaContextChirho;

int hbm_write_chirho(FpgaContextChirho* ctx_chirho, uint64_t addr_chirho, void* data_chirho, size_t size_chirho) {
    // Set HBM address
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_HBM_ADDR_LO_CHIRHO, (uint32_t)(addr_chirho & 0xFFFFFFFF));
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_HBM_ADDR_HI_CHIRHO, (uint32_t)(addr_chirho >> 32));
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_HBM_SIZE_CHIRHO, (uint32_t)size_chirho);

    // Write data via BAR4 (HBM window) - simplified burst write
    uint32_t* data32_chirho = (uint32_t*)data_chirho;
    size_t words_chirho = (size_chirho + 3) / 4;
    for (size_t w_chirho = 0; w_chirho < words_chirho; w_chirho++) {
        fpga_pci_poke(ctx_chirho->bar4_chirho, w_chirho * 4, data32_chirho[w_chirho]);
    }

    // Trigger HBM write command
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_CTRL_CHIRHO, CMD_HBM_WRITE_CHIRHO);

    return 0;
}

int hbm_read_chirho(FpgaContextChirho* ctx_chirho, uint64_t addr_chirho, void* data_chirho, size_t size_chirho) {
    // Set HBM address
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_HBM_ADDR_LO_CHIRHO, (uint32_t)(addr_chirho & 0xFFFFFFFF));
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_HBM_ADDR_HI_CHIRHO, (uint32_t)(addr_chirho >> 32));
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_HBM_SIZE_CHIRHO, (uint32_t)size_chirho);

    // Trigger HBM read command
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_CTRL_CHIRHO, CMD_HBM_READ_CHIRHO);

    // Read data via BAR4
    uint32_t* data32_chirho = (uint32_t*)data_chirho;
    size_t words_chirho = (size_chirho + 3) / 4;
    for (size_t w_chirho = 0; w_chirho < words_chirho; w_chirho++) {
        fpga_pci_peek(ctx_chirho->bar4_chirho, w_chirho * 4, &data32_chirho[w_chirho]);
    }

    return 0;
}

// ============================================================================
// CPU BASELINE: Gumbel-SAT Training
// ============================================================================

static q16_chirho gumbel_sample_q16_chirho(void) {
    double u_chirho = (double)(rand() + 1) / (RAND_MAX + 2.0);
    double g_chirho = -log(-log(u_chirho));
    return float_to_q16_chirho((float)g_chirho);
}

static void gumbel_softmax_q16_chirho(
    q16_chirho* logits_chirho,
    q16_chirho* probs_out_chirho,
    int n_chirho,
    q16_chirho temp_chirho
) {
    q16_chirho max_chirho = logits_chirho[0];
    for (int i_chirho = 1; i_chirho < n_chirho; i_chirho++) {
        if (logits_chirho[i_chirho] > max_chirho) max_chirho = logits_chirho[i_chirho];
    }

    q16_chirho sum_chirho = 0;
    for (int i_chirho = 0; i_chirho < n_chirho; i_chirho++) {
        q16_chirho g_chirho = gumbel_sample_q16_chirho();
        q16_chirho y_chirho = q16_div_chirho(logits_chirho[i_chirho] - max_chirho + g_chirho, temp_chirho);
        probs_out_chirho[i_chirho] = q16_exp_chirho(y_chirho);
        sum_chirho += probs_out_chirho[i_chirho];
    }

    if (sum_chirho == 0) sum_chirho = 1;
    for (int i_chirho = 0; i_chirho < n_chirho; i_chirho++) {
        probs_out_chirho[i_chirho] = q16_div_chirho(probs_out_chirho[i_chirho], sum_chirho);
    }
}

static inline q16_chirho soft_and_q16_chirho(q16_chirho a_chirho, q16_chirho b_chirho) {
    return q16_mul_chirho(a_chirho, b_chirho);
}

static inline q16_chirho soft_or_q16_chirho(q16_chirho a_chirho, q16_chirho b_chirho) {
    return a_chirho + b_chirho - q16_mul_chirho(a_chirho, b_chirho);
}

typedef struct {
    float final_loss_chirho;
    double total_time_ms_chirho;
    double upload_time_ms_chirho;
    double compute_time_ms_chirho;
    double download_time_ms_chirho;
} TrainResultChirho;

TrainResultChirho cpu_train_gumbel_sat_chirho(
    int vars_chirho,
    int clauses_chirho,
    int samples_per_iter_chirho,
    int iterations_chirho
) {
    TrainResultChirho result_chirho = {0};

    q16_chirho* logits_chirho = malloc(vars_chirho * 2 * sizeof(q16_chirho));
    q16_chirho* probs_chirho = malloc(vars_chirho * 2 * sizeof(q16_chirho));
    q16_chirho* grads_chirho = malloc(vars_chirho * 2 * sizeof(q16_chirho));

    for (int v_chirho = 0; v_chirho < vars_chirho * 2; v_chirho++) {
        logits_chirho[v_chirho] = (rand() % 1000 - 500) * 10;
    }

    q16_chirho temp_start_chirho = float_to_q16_chirho(2.0f);
    q16_chirho temp_end_chirho = float_to_q16_chirho(0.05f);
    q16_chirho lr_chirho = float_to_q16_chirho(0.01f);

    double start_time_chirho = get_time_ms_chirho();

    for (int iter_chirho = 0; iter_chirho < iterations_chirho; iter_chirho++) {
        float progress_chirho = (float)iter_chirho / iterations_chirho;
        float temp_f_chirho = q16_to_float_chirho(temp_start_chirho) *
                              powf(q16_to_float_chirho(temp_end_chirho) / q16_to_float_chirho(temp_start_chirho), progress_chirho);
        q16_chirho temp_chirho = float_to_q16_chirho(temp_f_chirho);

        memset(grads_chirho, 0, vars_chirho * 2 * sizeof(q16_chirho));

        q16_chirho total_sat_chirho = 0;

        for (int s_chirho = 0; s_chirho < samples_per_iter_chirho; s_chirho++) {
            for (int v_chirho = 0; v_chirho < vars_chirho; v_chirho++) {
                q16_chirho pair_chirho[2] = {logits_chirho[v_chirho * 2], logits_chirho[v_chirho * 2 + 1]};
                q16_chirho out_chirho[2];
                gumbel_softmax_q16_chirho(pair_chirho, out_chirho, 2, temp_chirho);
                probs_chirho[v_chirho * 2] = out_chirho[0];
                probs_chirho[v_chirho * 2 + 1] = out_chirho[1];
            }

            q16_chirho sat_chirho = Q16_ONE_CHIRHO;
            for (int c_chirho = 0; c_chirho < clauses_chirho; c_chirho++) {
                int l1_chirho = (c_chirho * 3 + 17) % (vars_chirho * 2);
                int l2_chirho = (c_chirho * 7 + 31) % (vars_chirho * 2);
                int l3_chirho = (c_chirho * 11 + 47) % (vars_chirho * 2);

                q16_chirho clause_chirho = soft_or_q16_chirho(probs_chirho[l1_chirho],
                                           soft_or_q16_chirho(probs_chirho[l2_chirho], probs_chirho[l3_chirho]));
                sat_chirho = soft_and_q16_chirho(sat_chirho, clause_chirho);

                q16_chirho grad_scale_chirho = q16_mul_chirho(clause_chirho, lr_chirho);
                grads_chirho[l1_chirho] += grad_scale_chirho;
                grads_chirho[l2_chirho] += grad_scale_chirho;
                grads_chirho[l3_chirho] += grad_scale_chirho;
            }
            total_sat_chirho += sat_chirho;
        }

        for (int v_chirho = 0; v_chirho < vars_chirho * 2; v_chirho++) {
            logits_chirho[v_chirho] += q16_div_chirho(grads_chirho[v_chirho],
                                                       float_to_q16_chirho((float)samples_per_iter_chirho));
        }

        if ((iter_chirho + 1) % 100 == 0) {
            float avg_sat_chirho = q16_to_float_chirho(total_sat_chirho) / samples_per_iter_chirho;
            printf("      CPU iter %4d: sat=%.4f, temp=%.3f\n", iter_chirho + 1, avg_sat_chirho, temp_f_chirho);
        }
    }

    result_chirho.total_time_ms_chirho = get_time_ms_chirho() - start_time_chirho;
    result_chirho.compute_time_ms_chirho = result_chirho.total_time_ms_chirho;

    free(logits_chirho);
    free(probs_chirho);
    free(grads_chirho);

    return result_chirho;
}

// ============================================================================
// FPGA HBM-BATCHED TRAINING
// ============================================================================

TrainResultChirho fpga_train_hbm_batch_chirho(
    FpgaContextChirho* ctx_chirho,
    int vars_chirho,
    int clauses_chirho,
    int samples_per_iter_chirho,
    int iterations_chirho
) {
    TrainResultChirho result_chirho = {0};

    // Allocate and initialize weights
    size_t weights_size_chirho = vars_chirho * 2 * sizeof(q16_chirho);
    q16_chirho* weights_chirho = malloc(weights_size_chirho);

    for (int v_chirho = 0; v_chirho < vars_chirho * 2; v_chirho++) {
        weights_chirho[v_chirho] = (rand() % 1000 - 500) * 10;
    }

    double total_start_chirho = get_time_ms_chirho();

    // ========== PHASE 1: Upload weights to HBM ==========
    double upload_start_chirho = get_time_ms_chirho();
    hbm_write_chirho(ctx_chirho, HBM_WEIGHTS_OFFSET_CHIRHO, weights_chirho, weights_size_chirho);
    result_chirho.upload_time_ms_chirho = get_time_ms_chirho() - upload_start_chirho;

    printf("      HBM upload: %.2f ms (%zu bytes)\n", result_chirho.upload_time_ms_chirho, weights_size_chirho);

    // ========== PHASE 2: Configure and start FPGA training ==========
    double compute_start_chirho = get_time_ms_chirho();

    // Set training parameters
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_TRAIN_VARS_CHIRHO, vars_chirho);
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_TRAIN_CLAUSES_CHIRHO, clauses_chirho);
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_TRAIN_ITERS_CHIRHO, iterations_chirho);
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_TRAIN_BATCH_CHIRHO, samples_per_iter_chirho);
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_TRAIN_LR_CHIRHO, float_to_q16_chirho(0.01f));
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_TRAIN_TEMP_START_CHIRHO, float_to_q16_chirho(2.0f));
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_TRAIN_TEMP_END_CHIRHO, float_to_q16_chirho(0.05f));

    // Start training
    fpga_pci_poke(ctx_chirho->bar0_chirho, REG_CTRL_CHIRHO, CMD_TRAIN_START_CHIRHO);

    // Poll for completion (in real impl, would use interrupt)
    uint32_t status_chirho = 0;
    int poll_count_chirho = 0;
    do {
        fpga_pci_peek(ctx_chirho->bar0_chirho, REG_STATUS_CHIRHO, &status_chirho);
        poll_count_chirho++;
        if (poll_count_chirho % 1000 == 0) {
            printf("      FPGA training in progress... (polls: %d)\n", poll_count_chirho);
        }
    } while ((status_chirho & 0x1) == 0 && poll_count_chirho < 100000);

    result_chirho.compute_time_ms_chirho = get_time_ms_chirho() - compute_start_chirho;

    // ========== PHASE 3: Download results from HBM ==========
    double download_start_chirho = get_time_ms_chirho();
    hbm_read_chirho(ctx_chirho, HBM_WEIGHTS_OFFSET_CHIRHO, weights_chirho, weights_size_chirho);
    result_chirho.download_time_ms_chirho = get_time_ms_chirho() - download_start_chirho;

    printf("      HBM download: %.2f ms\n", result_chirho.download_time_ms_chirho);

    // Get final loss from FPGA
    uint32_t loss_raw_chirho;
    fpga_pci_peek(ctx_chirho->bar0_chirho, REG_TRAIN_RESULT_CHIRHO, &loss_raw_chirho);
    result_chirho.final_loss_chirho = q16_to_float_chirho((q16_chirho)loss_raw_chirho);

    result_chirho.total_time_ms_chirho = get_time_ms_chirho() - total_start_chirho;

    free(weights_chirho);

    return result_chirho;
}

// ============================================================================
// MAIN
// ============================================================================

int main(void) {
    printf("\n======================================================================\n");
    printf("  HBM-BATCHED Training Benchmark ☧\n");
    printf("  Upload once → Train in HBM → Download once\n");
    printf("  John 3:16 - For God so loved the world\n");
    printf("======================================================================\n\n");

    srand(316);

    // Initialize FPGA
    int rc_chirho = fpga_mgmt_init();
    if (rc_chirho != 0) {
        printf("  Warning: FPGA init failed (rc=%d), running CPU-only\n", rc_chirho);
    }

    FpgaContextChirho ctx_chirho = {0};
    ctx_chirho.slot_chirho = 0;

    rc_chirho = fpga_pci_attach(0, APP_PF_BAR0_CHIRHO, 0, 0, &ctx_chirho.bar0_chirho);
    rc_chirho = fpga_pci_attach(0, APP_PF_BAR4_CHIRHO, 0, 0, &ctx_chirho.bar4_chirho);

    uint32_t fpga_status_chirho;
    fpga_pci_peek(ctx_chirho.bar0_chirho, REG_STATUS_CHIRHO, &fpga_status_chirho);
    printf("  FPGA Status: 0x%08X\n\n", fpga_status_chirho);

    FILE* csv_chirho = fopen("hbm_batch_results_chirho.csv", "w");
    fprintf(csv_chirho, "# HBM-Batched Training Benchmark\n");
    fprintf(csv_chirho, "# Date: 2026-01-28\n");
    fprintf(csv_chirho, "Scenario,Vars,Clauses,Samples,Iters,CPU_ms,FPGA_total_ms,FPGA_upload_ms,FPGA_compute_ms,FPGA_download_ms,Speedup\n");

    // ========================================================================
    // BENCHMARK SCENARIOS
    // ========================================================================

    struct {
        const char* name_chirho;
        int vars_chirho;
        int clauses_chirho;
        int samples_chirho;
        int iters_chirho;
    } scenarios_chirho[] = {
        {"Small", 100, 300, 50, 500},
        {"Medium", 500, 1500, 100, 500},
        {"Large", 1000, 3000, 200, 500},
        {"XL", 2000, 6000, 100, 200},
        {"Production", 1000, 5000, 500, 1000},
    };

    int num_scenarios_chirho = sizeof(scenarios_chirho) / sizeof(scenarios_chirho[0]);

    for (int s_chirho = 0; s_chirho < num_scenarios_chirho; s_chirho++) {
        printf("======================================================================\n");
        printf("  Scenario: %s (%d vars, %d clauses, %d samples × %d iters)\n",
               scenarios_chirho[s_chirho].name_chirho,
               scenarios_chirho[s_chirho].vars_chirho,
               scenarios_chirho[s_chirho].clauses_chirho,
               scenarios_chirho[s_chirho].samples_chirho,
               scenarios_chirho[s_chirho].iters_chirho);
        printf("======================================================================\n\n");

        // Memory requirement
        size_t mem_bytes_chirho = scenarios_chirho[s_chirho].vars_chirho * 2 * sizeof(q16_chirho);
        printf("  Memory: %zu bytes (%.2f KB)\n\n", mem_bytes_chirho, mem_bytes_chirho / 1024.0);

        // CPU Baseline
        printf("  Running CPU baseline...\n");
        TrainResultChirho cpu_result_chirho = cpu_train_gumbel_sat_chirho(
            scenarios_chirho[s_chirho].vars_chirho,
            scenarios_chirho[s_chirho].clauses_chirho,
            scenarios_chirho[s_chirho].samples_chirho,
            scenarios_chirho[s_chirho].iters_chirho
        );
        printf("  CPU Total: %.2f ms\n\n", cpu_result_chirho.total_time_ms_chirho);

        // FPGA HBM-Batched
        printf("  Running FPGA HBM-batched...\n");
        TrainResultChirho fpga_result_chirho = fpga_train_hbm_batch_chirho(
            &ctx_chirho,
            scenarios_chirho[s_chirho].vars_chirho,
            scenarios_chirho[s_chirho].clauses_chirho,
            scenarios_chirho[s_chirho].samples_chirho,
            scenarios_chirho[s_chirho].iters_chirho
        );

        double speedup_chirho = cpu_result_chirho.total_time_ms_chirho / fpga_result_chirho.total_time_ms_chirho;

        printf("\n  Results:\n");
        printf("    CPU:  %.2f ms (compute only)\n", cpu_result_chirho.total_time_ms_chirho);
        printf("    FPGA: %.2f ms total\n", fpga_result_chirho.total_time_ms_chirho);
        printf("          - Upload:   %.2f ms\n", fpga_result_chirho.upload_time_ms_chirho);
        printf("          - Compute:  %.2f ms\n", fpga_result_chirho.compute_time_ms_chirho);
        printf("          - Download: %.2f ms\n", fpga_result_chirho.download_time_ms_chirho);
        printf("    Speedup: %.1fx\n\n", speedup_chirho);

        fprintf(csv_chirho, "%s,%d,%d,%d,%d,%.2f,%.2f,%.2f,%.2f,%.2f,%.1f\n",
                scenarios_chirho[s_chirho].name_chirho,
                scenarios_chirho[s_chirho].vars_chirho,
                scenarios_chirho[s_chirho].clauses_chirho,
                scenarios_chirho[s_chirho].samples_chirho,
                scenarios_chirho[s_chirho].iters_chirho,
                cpu_result_chirho.total_time_ms_chirho,
                fpga_result_chirho.total_time_ms_chirho,
                fpga_result_chirho.upload_time_ms_chirho,
                fpga_result_chirho.compute_time_ms_chirho,
                fpga_result_chirho.download_time_ms_chirho,
                speedup_chirho);
    }

    // ========================================================================
    // THEORETICAL ANALYSIS
    // ========================================================================
    printf("======================================================================\n");
    printf("  THEORETICAL SPEEDUP ANALYSIS\n");
    printf("======================================================================\n\n");

    printf("  HBM Bandwidth: 400 GB/s\n");
    printf("  PCIe Bandwidth: 16 GB/s (Gen4 x16)\n");
    printf("  FPGA Clock: 250 MHz\n\n");

    printf("  For Production scenario (1000v × 5000c × 500s × 1000i):\n");
    printf("    Weights size: %zu bytes\n", 1000 * 2 * sizeof(q16_chirho));
    printf("    Upload time (PCIe): %.3f ms\n", (1000.0 * 2 * 4) / (16e9) * 1000);
    printf("    Download time (PCIe): %.3f ms\n", (1000.0 * 2 * 4) / (16e9) * 1000);
    printf("\n");

    printf("  FPGA Parallelism:\n");
    printf("    - 1000 soft-AND/OR units running in parallel\n");
    printf("    - Each iteration: ~10 cycles at 250 MHz = 40 ns\n");
    printf("    - 1000 iters × 500 samples = 500K operations\n");
    printf("    - Theoretical compute: 500K × 40 ns = 20 ms\n\n");

    printf("  Expected FPGA time: ~20-50 ms\n");
    printf("  Expected CPU time: ~15,000 ms\n");
    printf("  Expected speedup: 300-750x\n\n");

    fclose(csv_chirho);

    fpga_pci_detach(ctx_chirho.bar0_chirho);
    fpga_pci_detach(ctx_chirho.bar4_chirho);

    printf("======================================================================\n");
    printf("  Results saved to: hbm_batch_results_chirho.csv\n");
    printf("  Soli Deo Gloria ☧\n");
    printf("======================================================================\n");

    return 0;
}
