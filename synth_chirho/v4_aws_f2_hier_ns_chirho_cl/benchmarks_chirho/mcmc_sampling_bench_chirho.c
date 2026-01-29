// ============================================================================
// For God so loved the world - John 3:16 ☧
// MCMC Sampling & Probabilistic Exploration Benchmarks
// Tests soft_and and intersect_prob_domain for MCMC-style search
// ============================================================================
//
// This is NOT ML training - it's probabilistic logic programming:
// - Soft constraints with probabilistic weights
// - MCMC-style exploration of search space
// - Importance sampling for query answering
// - Probabilistic unification / constraint propagation
//
// ============================================================================

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include <math.h>

#include <fpga_pci.h>
#include <fpga_mgmt.h>

// Register addresses
#define REG_CONTROL_CHIRHO      0x04
#define REG_STATUS_CHIRHO       0x08
#define REG_CMD_LO_CHIRHO       0x10
#define REG_CMD_MID_CHIRHO      0x14
#define REG_CMD_HI_CHIRHO       0x18
#define REG_RESP_BASE_CHIRHO    0x20
#define REG_HIER_MODE_CHIRHO    0x40
#define REG_INFER_MODE_CHIRHO   0x70
#define REG_PROB_SEED_CHIRHO    0x74  // Random seed for sampling
#define REG_PROB_TEMP_CHIRHO    0x78  // Temperature for annealing

// Commands
#define CMD_SOFT_AND_CHIRHO        0x40  // Probabilistic AND
#define CMD_SOFT_OR_CHIRHO         0x41  // Probabilistic OR (noisy-or)
#define CMD_SAMPLE_CHIRHO          0x42  // Sample from distribution
#define CMD_IMPORTANCE_CHIRHO      0x43  // Importance weight
#define CMD_PROPAGATE_CHIRHO       0x44  // Propagate soft constraints
#define CMD_ANNEAL_STEP_CHIRHO     0x45  // Simulated annealing step

// Status bits
#define STATUS_DONE_CHIRHO      (1 << 0)
#define STATUS_VALID_CHIRHO     (1 << 1)

// Inference modes
#define INFER_BOOLEAN_CHIRHO    0x00
#define INFER_PROB_CHIRHO       0x01
#define INFER_LOG_PROB_CHIRHO   0x02  // Log-space for numerical stability

static pci_bar_handle_t bar0_chirho = PCI_BAR_HANDLE_INIT;

static inline int reg_read_chirho(uint32_t addr, uint32_t *val) {
    return fpga_pci_peek(bar0_chirho, addr, val);
}

static inline int reg_write_chirho(uint32_t addr, uint32_t val) {
    return fpga_pci_poke(bar0_chirho, addr, val);
}

static void wait_done_chirho(void) {
    uint32_t status;
    do { reg_read_chirho(REG_STATUS_CHIRHO, &status); }
    while (!(status & STATUS_DONE_CHIRHO));
}

static double get_time_ns_chirho(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e9 + ts.tv_nsec;
}

// Q8.8 fixed-point for probabilities (0.0 to 1.0 maps to 0x00 to 0x100)
static inline uint16_t prob_to_q8_chirho(float p) {
    if (p <= 0.0f) return 0;
    if (p >= 1.0f) return 0x100;
    return (uint16_t)(p * 256.0f);
}

static inline float q8_to_prob_chirho(uint16_t q) {
    return (float)q / 256.0f;
}

// ============================================================================
// BENCHMARK 1: Soft AND (Probabilistic Conjunction)
// Tests the soft_and_32_chirho and soft_and_16_chirho modules
// P(A AND B) ≈ P(A) * P(B) for independent, relaxed for soft constraints
// ============================================================================

static void bench_soft_and_chirho(int iterations) {
    printf("\n=== Soft AND (Probabilistic Conjunction) ===\n");
    printf("Tests soft_and modules for MCMC constraint propagation\n\n");

    reg_write_chirho(REG_INFER_MODE_CHIRHO, INFER_PROB_CHIRHO);

    double start = get_time_ns_chirho();
    float total_error = 0;
    int num_tests = 0;

    // Test various probability combinations
    float test_probs[] = {0.1f, 0.3f, 0.5f, 0.7f, 0.9f, 0.95f, 0.99f};
    int num_probs = sizeof(test_probs) / sizeof(float);

    for (int iter = 0; iter < iterations; iter++) {
        for (int i = 0; i < num_probs; i++) {
            for (int j = 0; j < num_probs; j++) {
                float p_a = test_probs[i];
                float p_b = test_probs[j];
                float expected = p_a * p_b;  // Independent AND

                uint16_t q_a = prob_to_q8_chirho(p_a);
                uint16_t q_b = prob_to_q8_chirho(p_b);

                // Pack into command: [prob_a:16][prob_b:16]
                uint32_t cmd = (q_a << 16) | q_b;

                reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_SOFT_AND_CHIRHO);
                reg_write_chirho(REG_CMD_MID_CHIRHO, cmd);
                wait_done_chirho();

                uint32_t result;
                reg_read_chirho(REG_RESP_BASE_CHIRHO, &result);
                float actual = q8_to_prob_chirho(result & 0xFFFF);

                total_error += fabsf(actual - expected);
                num_tests++;
            }
        }
    }

    double end = get_time_ns_chirho();
    double per_op_ns = (end - start) / num_tests;

    printf("Operations: %d\n", num_tests);
    printf("Time: %.1f ms (%.1f ns/op)\n", (end - start) / 1e6, per_op_ns);
    printf("Mean absolute error: %.6f\n", total_error / num_tests);
    printf("Throughput: %.0f soft_and/sec\n", 1e9 / per_op_ns);
}

// ============================================================================
// BENCHMARK 2: Probabilistic Domain Intersection
// Tests intersect_prob_domain_64_chirho for weighted constraint solving
// ============================================================================

static void bench_prob_domain_intersect_chirho(int iterations) {
    printf("\n=== Probabilistic Domain Intersection ===\n");
    printf("Tests intersect_prob_domain for weighted constraint propagation\n\n");

    reg_write_chirho(REG_INFER_MODE_CHIRHO, INFER_PROB_CHIRHO);

    // Create test domains with probabilistic weights
    // Domain A: values {0,1,2,3} with weights {0.4, 0.3, 0.2, 0.1}
    // Domain B: values {1,2,3,4} with weights {0.1, 0.2, 0.3, 0.4}
    // Intersection should give weighted overlap

    double start = get_time_ns_chirho();

    for (int iter = 0; iter < iterations; iter++) {
        // Load domain A weights
        uint32_t weights_a = (prob_to_q8_chirho(0.4f) << 24) |
                             (prob_to_q8_chirho(0.3f) << 16) |
                             (prob_to_q8_chirho(0.2f) << 8) |
                             prob_to_q8_chirho(0.1f);

        // Load domain B weights (shifted by 1)
        uint32_t weights_b = (prob_to_q8_chirho(0.1f) << 24) |
                             (prob_to_q8_chirho(0.2f) << 16) |
                             (prob_to_q8_chirho(0.3f) << 8) |
                             prob_to_q8_chirho(0.4f);

        // Domain masks (which values are present)
        uint32_t mask_a = 0x0F;  // {0,1,2,3}
        uint32_t mask_b = 0x1E;  // {1,2,3,4}

        reg_write_chirho(REG_CMD_LO_CHIRHO, 0x50);  // Prob intersect cmd
        reg_write_chirho(REG_CMD_MID_CHIRHO, weights_a);
        reg_write_chirho(REG_CMD_HI_CHIRHO, (mask_a << 16) | mask_b);
        wait_done_chirho();

        // Second operand
        reg_write_chirho(REG_CMD_LO_CHIRHO, 0x51);  // Continue
        reg_write_chirho(REG_CMD_MID_CHIRHO, weights_b);
        wait_done_chirho();
    }

    double end = get_time_ns_chirho();
    printf("Iterations: %d\n", iterations);
    printf("Time: %.1f ms (%.1f us/intersect)\n",
           (end - start) / 1e6, (end - start) / 1e3 / iterations);

    // Read result
    uint32_t result_mask, result_weights;
    reg_read_chirho(REG_RESP_BASE_CHIRHO, &result_mask);
    reg_read_chirho(REG_RESP_BASE_CHIRHO + 4, &result_weights);

    printf("Result mask: 0x%02X (expect 0x0E = {1,2,3})\n", result_mask & 0xFF);
}

// ============================================================================
// BENCHMARK 3: MCMC Sampling from Soft Constraints
// Metropolis-Hastings style sampling using hardware RNG
// ============================================================================

static void bench_mcmc_sampling_chirho(int num_samples, int num_vars) {
    printf("\n=== MCMC Sampling from Soft Constraints ===\n");
    printf("Metropolis-Hastings exploration of constraint space\n\n");

    reg_write_chirho(REG_INFER_MODE_CHIRHO, INFER_PROB_CHIRHO);

    // Set random seed
    reg_write_chirho(REG_PROB_SEED_CHIRHO, (uint32_t)time(NULL));

    // Temperature for acceptance probability
    float temperature = 1.0f;
    reg_write_chirho(REG_PROB_TEMP_CHIRHO, prob_to_q8_chirho(temperature));

    double start = get_time_ns_chirho();
    int accepted = 0;

    // Initial state: all variables at 0.5 probability
    uint32_t current_state[16] = {0};
    for (int i = 0; i < num_vars && i < 16; i++) {
        current_state[i] = prob_to_q8_chirho(0.5f);
    }

    for (int sample = 0; sample < num_samples; sample++) {
        // Propose new state (flip one variable)
        int flip_var = sample % num_vars;

        // Current probability of this assignment
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_SAMPLE_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, current_state[flip_var]);
        wait_done_chirho();

        uint32_t current_prob;
        reg_read_chirho(REG_RESP_BASE_CHIRHO, &current_prob);

        // Propose flipped value
        uint32_t proposed = 0x100 - current_state[flip_var];  // Flip probability

        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_SAMPLE_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, proposed);
        wait_done_chirho();

        uint32_t proposed_prob;
        reg_read_chirho(REG_RESP_BASE_CHIRHO, &proposed_prob);

        // Metropolis acceptance: accept if proposed_prob > current_prob * random
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_IMPORTANCE_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, (proposed_prob << 16) | current_prob);
        wait_done_chirho();

        uint32_t accept;
        reg_read_chirho(REG_RESP_BASE_CHIRHO, &accept);

        if (accept & 1) {
            current_state[flip_var] = proposed;
            accepted++;
        }
    }

    double end = get_time_ns_chirho();

    printf("Samples: %d, Variables: %d\n", num_samples, num_vars);
    printf("Time: %.1f ms (%.2f us/sample)\n",
           (end - start) / 1e6, (end - start) / 1e3 / num_samples);
    printf("Acceptance rate: %.1f%%\n", 100.0 * accepted / num_samples);
    printf("Throughput: %.0f samples/sec\n", num_samples * 1e9 / (end - start));
}

// ============================================================================
// BENCHMARK 4: Importance Sampling for Query Answering
// Estimate probability of query by weighted sampling
// ============================================================================

typedef struct {
    uint32_t evidence;    // Observed variable assignments
    uint32_t query;       // Query variable
    float true_prob;      // Known true probability (for validation)
} importance_query_chirho_t;

static importance_query_chirho_t importance_queries_chirho[] = {
    // Simple Bayesian network queries
    {0x01, 0x02, 0.7f},   // P(B|A) where A->B
    {0x03, 0x04, 0.6f},   // P(C|A,B)
    {0x00, 0x01, 0.5f},   // P(A) - prior
    {0x05, 0x02, 0.8f},   // P(B|A,C)
};
#define NUM_IMPORTANCE_QUERIES_CHIRHO \
    (sizeof(importance_queries_chirho) / sizeof(importance_query_chirho_t))

static void bench_importance_sampling_chirho(int samples_per_query) {
    printf("\n=== Importance Sampling for Query Answering ===\n");
    printf("Estimate query probabilities via weighted sampling\n\n");

    reg_write_chirho(REG_INFER_MODE_CHIRHO, INFER_LOG_PROB_CHIRHO);  // Log space

    double start = get_time_ns_chirho();
    float total_error = 0;

    for (size_t q = 0; q < NUM_IMPORTANCE_QUERIES_CHIRHO; q++) {
        importance_query_chirho_t *query = &importance_queries_chirho[q];

        float sum_weights = 0;
        float sum_weighted_query = 0;

        for (int s = 0; s < samples_per_query; s++) {
            // Sample from proposal distribution
            reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_SAMPLE_CHIRHO);
            reg_write_chirho(REG_CMD_MID_CHIRHO, query->evidence);
            wait_done_chirho();

            uint32_t sample;
            reg_read_chirho(REG_RESP_BASE_CHIRHO, &sample);

            // Compute importance weight
            reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_IMPORTANCE_CHIRHO);
            reg_write_chirho(REG_CMD_MID_CHIRHO, sample);
            reg_write_chirho(REG_CMD_HI_CHIRHO, query->evidence);
            wait_done_chirho();

            uint32_t weight_q8;
            reg_read_chirho(REG_RESP_BASE_CHIRHO, &weight_q8);
            float weight = q8_to_prob_chirho(weight_q8);

            // Check if query is satisfied in this sample
            int query_satisfied = (sample & query->query) != 0;

            sum_weights += weight;
            if (query_satisfied) {
                sum_weighted_query += weight;
            }
        }

        float estimated = sum_weighted_query / sum_weights;
        float error = fabsf(estimated - query->true_prob);
        total_error += error;

        printf("  Query %zu: estimated=%.3f, true=%.3f, error=%.3f\n",
               q, estimated, query->true_prob, error);
    }

    double end = get_time_ns_chirho();

    printf("\nTotal time: %.1f ms\n", (end - start) / 1e6);
    printf("Mean absolute error: %.4f\n", total_error / NUM_IMPORTANCE_QUERIES_CHIRHO);
}

// ============================================================================
// BENCHMARK 5: Simulated Annealing for Constraint Optimization
// Find satisfying assignment using temperature-based exploration
// ============================================================================

static void bench_simulated_annealing_chirho(int num_vars, int max_steps) {
    printf("\n=== Simulated Annealing for Constraint Optimization ===\n");
    printf("Find satisfying assignments using temperature schedule\n\n");

    reg_write_chirho(REG_INFER_MODE_CHIRHO, INFER_PROB_CHIRHO);

    // Create a random constraint satisfaction problem
    // (simulated - actual constraints would be loaded)

    double start = get_time_ns_chirho();

    float temperature = 1.0f;
    float cooling_rate = 0.99f;
    float min_temp = 0.01f;

    uint32_t best_score = 0;
    int best_step = 0;

    for (int step = 0; step < max_steps && temperature > min_temp; step++) {
        // Update temperature in hardware
        reg_write_chirho(REG_PROB_TEMP_CHIRHO, prob_to_q8_chirho(temperature));

        // Take annealing step
        reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_ANNEAL_STEP_CHIRHO);
        reg_write_chirho(REG_CMD_MID_CHIRHO, num_vars);
        wait_done_chirho();

        // Get current score (number of satisfied constraints)
        uint32_t score;
        reg_read_chirho(REG_RESP_BASE_CHIRHO, &score);

        if (score > best_score) {
            best_score = score;
            best_step = step;
        }

        // Cool down
        temperature *= cooling_rate;
    }

    double end = get_time_ns_chirho();

    printf("Variables: %d, Max steps: %d\n", num_vars, max_steps);
    printf("Time: %.1f ms\n", (end - start) / 1e6);
    printf("Best score: %u at step %d\n", best_score, best_step);
    printf("Final temperature: %.4f\n", temperature);
}

// ============================================================================
// BENCHMARK 6: Soft Constraint Propagation (Arc Consistency)
// Propagate probabilistic constraints through variable network
// ============================================================================

static void bench_soft_propagation_chirho(int num_vars, int num_constraints, int iterations) {
    printf("\n=== Soft Constraint Propagation ===\n");
    printf("Probabilistic arc consistency for constraint networks\n\n");

    reg_write_chirho(REG_INFER_MODE_CHIRHO, INFER_PROB_CHIRHO);

    double start = get_time_ns_chirho();

    // Initialize all domains to uniform
    for (int v = 0; v < num_vars; v++) {
        reg_write_chirho(REG_CMD_LO_CHIRHO, 0x60);  // Init domain
        reg_write_chirho(REG_CMD_MID_CHIRHO, v);
        reg_write_chirho(REG_CMD_HI_CHIRHO, prob_to_q8_chirho(0.5f));
        wait_done_chirho();
    }

    // Propagation iterations
    int changes = 0;
    for (int iter = 0; iter < iterations; iter++) {
        int iter_changes = 0;

        // For each constraint, propagate
        for (int c = 0; c < num_constraints; c++) {
            // Simulate constraint: var[c%num_vars] <-> var[(c+1)%num_vars]
            int var1 = c % num_vars;
            int var2 = (c + 1) % num_vars;

            reg_write_chirho(REG_CMD_LO_CHIRHO, CMD_PROPAGATE_CHIRHO);
            reg_write_chirho(REG_CMD_MID_CHIRHO, (var1 << 16) | var2);
            wait_done_chirho();

            uint32_t changed;
            reg_read_chirho(REG_RESP_BASE_CHIRHO, &changed);
            iter_changes += (changed & 1);
        }

        changes += iter_changes;
        if (iter_changes == 0) {
            printf("Converged at iteration %d\n", iter);
            break;
        }
    }

    double end = get_time_ns_chirho();

    printf("Variables: %d, Constraints: %d\n", num_vars, num_constraints);
    printf("Time: %.1f ms\n", (end - start) / 1e6);
    printf("Total domain changes: %d\n", changes);
}

// ============================================================================
// Main
// ============================================================================

int main(int argc, char **argv) {
    (void)argc; (void)argv;
    int rc;

    printf("============================================================\n");
    printf("MCMC Sampling & Probabilistic Exploration ☧\n");
    printf("Benchmarks for soft_and, intersect_prob_domain modules\n");
    printf("For God so loved the world - John 3:16\n");
    printf("============================================================\n");

    rc = fpga_mgmt_init();
    if (rc) { printf("ERROR: fpga_mgmt_init: %d\n", rc); return 1; }

    rc = fpga_pci_attach(0, FPGA_APP_PF, APP_PF_BAR0, 0, &bar0_chirho);
    if (rc) { printf("ERROR: fpga_pci_attach: %d\n", rc); return 1; }

    // Reset
    reg_write_chirho(REG_CONTROL_CHIRHO, 0x02);
    usleep(10000);
    reg_write_chirho(REG_CONTROL_CHIRHO, 0x01);

    // Run benchmarks
    bench_soft_and_chirho(100);
    bench_prob_domain_intersect_chirho(10000);
    bench_mcmc_sampling_chirho(10000, 8);
    bench_importance_sampling_chirho(1000);
    bench_simulated_annealing_chirho(16, 1000);
    bench_soft_propagation_chirho(32, 64, 100);

    printf("\n============================================================\n");
    printf("MCMC benchmarks complete. Soli Deo Gloria ☧\n");
    printf("============================================================\n");

    fpga_pci_detach(bar0_chirho);
    fpga_mgmt_close();

    return 0;
}
