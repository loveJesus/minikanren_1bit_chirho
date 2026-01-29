// ============================================================================
// For God so loved the world - John 3:16 ☧
// Software Baseline Implementations for FPGA Comparison
// Run on CPU to measure speedup vs FPGA
// ============================================================================

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <math.h>
#include <immintrin.h>  // For AVX2 intrinsics

// ============================================================================
// Timing utilities
// ============================================================================

static double get_time_ns_chirho(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e9 + ts.tv_nsec;
}

typedef struct {
    const char *name;
    double cpu_ns;
    double fpga_ns;  // Fill in from FPGA run
    double speedup;
} comparison_chirho_t;

static comparison_chirho_t results_chirho[32];
static int num_results_chirho = 0;

static void record_result_chirho(const char *name, double cpu_ns) {
    results_chirho[num_results_chirho].name = name;
    results_chirho[num_results_chirho].cpu_ns = cpu_ns;
    results_chirho[num_results_chirho].fpga_ns = 0;  // Set later
    results_chirho[num_results_chirho].speedup = 0;
    num_results_chirho++;
}

// ============================================================================
// SOFTWARE: 64-bit Domain Intersection
// ============================================================================

static uint64_t sw_intersect_64_chirho(uint64_t a, uint64_t b) {
    return a & b;
}

static void bench_sw_intersect_64_chirho(int iterations) {
    printf("\n=== [CPU] 64-bit Domain Intersection ===\n");

    uint64_t a = 0xAAAAAAAAAAAAAAAAULL;
    uint64_t b = 0x5555555555555555ULL;
    uint64_t result = 0;

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        a ^= i;
        b ^= (i << 1);
        result ^= sw_intersect_64_chirho(a, b);
    }
    double end = get_time_ns_chirho();

    double per_op = (end - start) / iterations;
    printf("Iterations: %d\n", iterations);
    printf("Time: %.3f ms (%.2f ns/op)\n", (end - start) / 1e6, per_op);
    printf("Throughput: %.0f ops/sec\n", 1e9 / per_op);
    printf("(result: 0x%lx - prevents optimization)\n", result);

    record_result_chirho("Intersect 64-bit", per_op);
}

// ============================================================================
// SOFTWARE: 256-bit Domain Intersection (4x uint64_t)
// ============================================================================

typedef struct { uint64_t w[4]; } bitvec256_chirho_t;

static bitvec256_chirho_t sw_intersect_256_chirho(bitvec256_chirho_t a, bitvec256_chirho_t b) {
    bitvec256_chirho_t r;
    r.w[0] = a.w[0] & b.w[0];
    r.w[1] = a.w[1] & b.w[1];
    r.w[2] = a.w[2] & b.w[2];
    r.w[3] = a.w[3] & b.w[3];
    return r;
}

#ifdef __AVX2__
static __m256i sw_intersect_256_avx_chirho(__m256i a, __m256i b) {
    return _mm256_and_si256(a, b);
}
#endif

static void bench_sw_intersect_256_chirho(int iterations) {
    printf("\n=== [CPU] 256-bit Domain Intersection ===\n");

    bitvec256_chirho_t a = {{0xAAAAAAAAAAAAAAAAULL, 0x5555555555555555ULL,
                            0xAAAAAAAAAAAAAAAAULL, 0x5555555555555555ULL}};
    bitvec256_chirho_t b = {{0x5555555555555555ULL, 0xAAAAAAAAAAAAAAAAULL,
                            0x5555555555555555ULL, 0xAAAAAAAAAAAAAAAAULL}};
    bitvec256_chirho_t result = {{0}};

    // Scalar version
    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        a.w[0] ^= i;
        bitvec256_chirho_t r = sw_intersect_256_chirho(a, b);
        result.w[0] ^= r.w[0];
    }
    double end = get_time_ns_chirho();
    double scalar_ns = (end - start) / iterations;

    printf("Scalar: %.2f ns/op (%.0f ops/sec)\n", scalar_ns, 1e9 / scalar_ns);

#ifdef __AVX2__
    // AVX2 version
    __m256i va = _mm256_loadu_si256((__m256i*)&a);
    __m256i vb = _mm256_loadu_si256((__m256i*)&b);
    __m256i vr = _mm256_setzero_si256();

    start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        va = _mm256_xor_si256(va, _mm256_set1_epi64x(i));
        vr = _mm256_xor_si256(vr, sw_intersect_256_avx_chirho(va, vb));
    }
    end = get_time_ns_chirho();
    double avx_ns = (end - start) / iterations;

    printf("AVX2:   %.2f ns/op (%.0f ops/sec)\n", avx_ns, 1e9 / avx_ns);
    record_result_chirho("Intersect 256-bit (AVX2)", avx_ns);
#else
    record_result_chirho("Intersect 256-bit (scalar)", scalar_ns);
#endif
}

// ============================================================================
// SOFTWARE: 512-bit Domain Intersection (8x uint64_t)
// ============================================================================

typedef struct { uint64_t w[8]; } bitvec512_chirho_t;

static bitvec512_chirho_t sw_intersect_512_chirho(bitvec512_chirho_t a, bitvec512_chirho_t b) {
    bitvec512_chirho_t r;
    for (int i = 0; i < 8; i++) {
        r.w[i] = a.w[i] & b.w[i];
    }
    return r;
}

static void bench_sw_intersect_512_chirho(int iterations) {
    printf("\n=== [CPU] 512-bit Domain Intersection ===\n");

    bitvec512_chirho_t a, b, result;
    for (int i = 0; i < 8; i++) {
        a.w[i] = 0xAAAAAAAAAAAAAAAAULL ^ (i * 0x123456789ABCDEFULL);
        b.w[i] = 0x5555555555555555ULL ^ (i * 0xFEDCBA987654321ULL);
        result.w[i] = 0;
    }

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        a.w[0] ^= i;
        bitvec512_chirho_t r = sw_intersect_512_chirho(a, b);
        result.w[0] ^= r.w[0];
    }
    double end = get_time_ns_chirho();

    double per_op = (end - start) / iterations;
    printf("Scalar: %.2f ns/op (%.0f ops/sec)\n", per_op, 1e9 / per_op);

    record_result_chirho("Intersect 512-bit", per_op);
}

// ============================================================================
// SOFTWARE: Popcount (256-bit and 512-bit)
// ============================================================================

static int sw_popcount_256_chirho(bitvec256_chirho_t v) {
    return __builtin_popcountll(v.w[0]) + __builtin_popcountll(v.w[1]) +
           __builtin_popcountll(v.w[2]) + __builtin_popcountll(v.w[3]);
}

static int sw_popcount_512_chirho(bitvec512_chirho_t v) {
    int count = 0;
    for (int i = 0; i < 8; i++) {
        count += __builtin_popcountll(v.w[i]);
    }
    return count;
}

static void bench_sw_popcount_chirho(int iterations) {
    printf("\n=== [CPU] Popcount (256-bit and 512-bit) ===\n");

    bitvec256_chirho_t v256 = {{0x123456789ABCDEFULL, 0xFEDCBA9876543210ULL,
                                0xAAAAAAAAAAAAAAAAULL, 0x5555555555555555ULL}};
    bitvec512_chirho_t v512;
    for (int i = 0; i < 8; i++) v512.w[i] = v256.w[i % 4] ^ i;

    int result = 0;

    // 256-bit popcount
    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        v256.w[0] ^= i;
        result += sw_popcount_256_chirho(v256);
    }
    double end = get_time_ns_chirho();
    double pop256_ns = (end - start) / iterations;
    printf("256-bit popcount: %.2f ns/op\n", pop256_ns);

    // 512-bit popcount
    start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        v512.w[0] ^= i;
        result += sw_popcount_512_chirho(v512);
    }
    end = get_time_ns_chirho();
    double pop512_ns = (end - start) / iterations;
    printf("512-bit popcount: %.2f ns/op\n", pop512_ns);

    printf("(result: %d)\n", result);

    record_result_chirho("Popcount 256-bit", pop256_ns);
    record_result_chirho("Popcount 512-bit", pop512_ns);
}

// ============================================================================
// SOFTWARE: Hierarchical 65K (256²) Intersection
// ============================================================================

typedef struct {
    uint64_t level0;           // 64-bit mask for which L1 blocks are active
    uint64_t level1[64];       // 64 x 64-bit = 4096 bits per active L0 bit
} hier_65k_chirho_t;

static int sw_hier_65k_intersect_chirho(hier_65k_chirho_t *a, hier_65k_chirho_t *b,
                                         hier_65k_chirho_t *result) {
    result->level0 = a->level0 & b->level0;
    int count = 0;

    uint64_t active = result->level0;
    while (active) {
        int idx = __builtin_ctzll(active);
        result->level1[idx] = a->level1[idx] & b->level1[idx];
        count += __builtin_popcountll(result->level1[idx]);
        active &= active - 1;
    }
    return count;
}

static void bench_sw_hier_65k_chirho(int iterations) {
    printf("\n=== [CPU] Hierarchical 65K (256²) Intersection ===\n");

    hier_65k_chirho_t a, b, result;

    // Initialize with sparse patterns (~25% density)
    a.level0 = 0xAAAAAAAAAAAAAAAAULL;
    b.level0 = 0xCCCCCCCCCCCCCCCCULL;
    for (int i = 0; i < 64; i++) {
        a.level1[i] = 0x123456789ABCDEFULL ^ (i * 0x1111111111111111ULL);
        b.level1[i] = 0xFEDCBA9876543210ULL ^ (i * 0x2222222222222222ULL);
    }

    int total_count = 0;
    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        a.level0 ^= (i & 0xF);
        total_count += sw_hier_65k_intersect_chirho(&a, &b, &result);
    }
    double end = get_time_ns_chirho();

    double per_op = (end - start) / iterations;
    printf("Time: %.2f ns/op (%.0f ops/sec)\n", per_op, 1e9 / per_op);
    printf("Avg intersection size: %.1f bits\n", (double)total_count / iterations);

    record_result_chirho("Hier 65K intersect", per_op);
}

// ============================================================================
// SOFTWARE: Soft AND (Probabilistic Conjunction)
// P(A AND B) = P(A) * P(B)
// ============================================================================

static float sw_soft_and_chirho(float p_a, float p_b) {
    return p_a * p_b;
}

static void bench_sw_soft_and_chirho(int iterations) {
    printf("\n=== [CPU] Soft AND (Probabilistic) ===\n");

    float result = 0;
    float probs[] = {0.1f, 0.3f, 0.5f, 0.7f, 0.9f};
    int num_probs = 5;

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        float p_a = probs[i % num_probs];
        float p_b = probs[(i + 1) % num_probs];
        result += sw_soft_and_chirho(p_a, p_b);
    }
    double end = get_time_ns_chirho();

    double per_op = (end - start) / iterations;
    printf("Time: %.2f ns/op (%.0f ops/sec)\n", per_op, 1e9 / per_op);
    printf("(result: %.4f)\n", result);

    record_result_chirho("Soft AND", per_op);
}

// ============================================================================
// SOFTWARE: MCMC Sampling Step (Metropolis-Hastings)
// ============================================================================

static uint32_t xorshift32_chirho(uint32_t *state) {
    uint32_t x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    return x;
}

static int sw_mcmc_step_chirho(float *current_prob, float proposed_prob, uint32_t *rng) {
    // Accept if proposed is better, or probabilistically if worse
    if (proposed_prob >= *current_prob) {
        *current_prob = proposed_prob;
        return 1;
    }
    float accept_ratio = proposed_prob / *current_prob;
    float random = (float)xorshift32_chirho(rng) / (float)UINT32_MAX;
    if (random < accept_ratio) {
        *current_prob = proposed_prob;
        return 1;
    }
    return 0;
}

static void bench_sw_mcmc_chirho(int iterations) {
    printf("\n=== [CPU] MCMC Sampling (Metropolis-Hastings) ===\n");

    uint32_t rng = 12345;
    float current = 0.5f;
    int accepted = 0;

    double start = get_time_ns_chirho();
    for (int i = 0; i < iterations; i++) {
        // Propose new state
        float proposed = (float)xorshift32_chirho(&rng) / (float)UINT32_MAX;
        accepted += sw_mcmc_step_chirho(&current, proposed, &rng);
    }
    double end = get_time_ns_chirho();

    double per_op = (end - start) / iterations;
    printf("Time: %.2f ns/op (%.0f samples/sec)\n", per_op, 1e9 / per_op);
    printf("Acceptance rate: %.1f%%\n", 100.0 * accepted / iterations);

    record_result_chirho("MCMC step", per_op);
}

// ============================================================================
// SOFTWARE: Importance Sampling
// ============================================================================

static void bench_sw_importance_sampling_chirho(int num_samples) {
    printf("\n=== [CPU] Importance Sampling ===\n");

    uint32_t rng = 54321;
    float sum_weights = 0;
    float sum_weighted = 0;

    double start = get_time_ns_chirho();
    for (int i = 0; i < num_samples; i++) {
        // Sample from proposal
        float sample = (float)xorshift32_chirho(&rng) / (float)UINT32_MAX;

        // Compute importance weight (target / proposal)
        float target = sample * sample;  // Example: prefer higher values
        float proposal = 1.0f;           // Uniform proposal
        float weight = target / proposal;

        sum_weights += weight;
        sum_weighted += weight * sample;
    }
    double end = get_time_ns_chirho();

    float estimate = sum_weighted / sum_weights;
    double per_op = (end - start) / num_samples;

    printf("Time: %.2f ns/sample (%.0f samples/sec)\n", per_op, 1e9 / per_op);
    printf("Estimate: %.4f\n", estimate);

    record_result_chirho("Importance sample", per_op);
}

// ============================================================================
// SOFTWARE: Constraint Propagation
// ============================================================================

typedef struct {
    uint64_t domain;  // Bit vector of possible values
} variable_chirho_t;

static int sw_propagate_chirho(variable_chirho_t *vars, int num_vars,
                                int (*constraint)(int, int)) {
    int changes = 0;
    for (int i = 0; i < num_vars; i++) {
        for (int j = i + 1; j < num_vars; j++) {
            uint64_t old_i = vars[i].domain;
            uint64_t old_j = vars[j].domain;

            // Arc consistency: remove inconsistent values
            uint64_t new_i = 0, new_j = 0;
            for (int vi = 0; vi < 64; vi++) {
                if (!(old_i & (1ULL << vi))) continue;
                for (int vj = 0; vj < 64; vj++) {
                    if (!(old_j & (1ULL << vj))) continue;
                    if (constraint(vi, vj)) {
                        new_i |= (1ULL << vi);
                        new_j |= (1ULL << vj);
                    }
                }
            }

            if (new_i != old_i) { vars[i].domain = new_i; changes++; }
            if (new_j != old_j) { vars[j].domain = new_j; changes++; }
        }
    }
    return changes;
}

static int constraint_neq_chirho(int a, int b) { return a != b; }

static void bench_sw_propagation_chirho(int num_vars, int iterations) {
    printf("\n=== [CPU] Constraint Propagation ===\n");

    variable_chirho_t vars[32];
    for (int i = 0; i < num_vars && i < 32; i++) {
        vars[i].domain = 0xFFFFFFFFFFFFFFFFULL;  // All values possible
    }

    int total_changes = 0;
    double start = get_time_ns_chirho();
    for (int iter = 0; iter < iterations; iter++) {
        // Reset domains
        for (int i = 0; i < num_vars; i++) {
            vars[i].domain = 0xFFFFFFFFFFFFFFFFULL >> (64 - 8);  // 8 values
        }
        total_changes += sw_propagate_chirho(vars, num_vars, constraint_neq_chirho);
    }
    double end = get_time_ns_chirho();

    double per_iter = (end - start) / iterations;
    printf("Variables: %d\n", num_vars);
    printf("Time: %.2f us/iteration\n", per_iter / 1e3);
    printf("Avg changes: %.1f\n", (double)total_changes / iterations);

    record_result_chirho("Propagation (8 vars)", per_iter);
}

// ============================================================================
// Print comparison summary
// ============================================================================

static void print_comparison_chirho(void) {
    printf("\n");
    printf("============================================================\n");
    printf("CPU BASELINE SUMMARY\n");
    printf("============================================================\n");
    printf("%-30s | %12s | %12s | %8s\n", "Benchmark", "CPU (ns)", "FPGA (ns)", "Speedup");
    printf("-------------------------------+--------------+--------------+---------\n");

    for (int i = 0; i < num_results_chirho; i++) {
        comparison_chirho_t *r = &results_chirho[i];
        if (r->fpga_ns > 0) {
            r->speedup = r->cpu_ns / r->fpga_ns;
            printf("%-30s | %12.1f | %12.1f | %7.1fx\n",
                   r->name, r->cpu_ns, r->fpga_ns, r->speedup);
        } else {
            printf("%-30s | %12.1f | %12s | %8s\n",
                   r->name, r->cpu_ns, "(TBD)", "(TBD)");
        }
    }

    printf("============================================================\n");
    printf("To complete comparison, run FPGA benchmarks and fill in times\n");
}

// ============================================================================
// Main
// ============================================================================

int main(void) {
    printf("============================================================\n");
    printf("Software Baseline Benchmarks ☧\n");
    printf("CPU reference for FPGA comparison\n");
    printf("For God so loved the world - John 3:16\n");
    printf("============================================================\n");

    // Domain intersection benchmarks
    bench_sw_intersect_64_chirho(10000000);
    bench_sw_intersect_256_chirho(10000000);
    bench_sw_intersect_512_chirho(10000000);
    bench_sw_popcount_chirho(10000000);
    bench_sw_hier_65k_chirho(100000);

    // Probabilistic / MCMC benchmarks
    bench_sw_soft_and_chirho(10000000);
    bench_sw_mcmc_chirho(1000000);
    bench_sw_importance_sampling_chirho(1000000);
    bench_sw_propagation_chirho(8, 10000);

    // Print summary
    print_comparison_chirho();

    printf("\nSoli Deo Gloria ☧\n");
    return 0;
}
