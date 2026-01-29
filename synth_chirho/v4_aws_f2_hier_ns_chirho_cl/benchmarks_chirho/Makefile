# ============================================================================
# For God so loved the world - John 3:16 ☧
# Makefile for miniKanren FPGA Benchmarks
# ============================================================================

SDK_DIR ?= $(HOME)/src/project_data/aws-fpga/sdk
INCLUDES = -I$(SDK_DIR)/userspace/include
LDFLAGS = -L$(SDK_DIR)/userspace/lib/so -Wl,-rpath,$(SDK_DIR)/userspace/lib/so
LDLIBS = -lfpga_mgmt -lrt -lpthread -lm

CC = gcc
CFLAGS = -Wall -Wextra -O3 -g $(INCLUDES)

# FPGA benchmarks (require SDK)
FPGA_TARGETS = benchmark_suite_chirho neurosymbolic_bench_chirho mcmc_sampling_bench_chirho

# CPU-only baselines (no SDK needed)
CPU_TARGETS = software_baseline_chirho

# All targets
TARGETS = $(FPGA_TARGETS) $(CPU_TARGETS)

.PHONY: all clean run run-all run-mcmc run-neurosym run-baseline fpga-only cpu-only compare

all: $(TARGETS)

benchmark_suite_chirho: benchmark_suite_chirho.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS) $(LDLIBS)

neurosymbolic_bench_chirho: neurosymbolic_bench_chirho.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS) $(LDLIBS)

mcmc_sampling_bench_chirho: mcmc_sampling_bench_chirho.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS) $(LDLIBS)

# CPU-only software baseline (no SDK dependency, uses AVX2)
software_baseline_chirho: software_baseline_chirho.c
	$(CC) -Wall -Wextra -O3 -g -mavx2 -o $@ $< -lrt -lpthread -lm

# Build only FPGA benchmarks (requires SDK)
fpga-only: $(FPGA_TARGETS)

# Build only CPU baselines (can run anywhere)
cpu-only: $(CPU_TARGETS)

clean:
	rm -f $(TARGETS) *.o results_*.txt *_results_chirho.txt

# Run all benchmarks
run-all: $(TARGETS)
	@echo "=== Hardware Domain Benchmarks ===" | tee benchmark_results_chirho.txt
	sudo ./benchmark_suite_chirho | tee -a benchmark_results_chirho.txt
	@echo "\n=== MCMC Sampling Benchmarks ===" | tee -a benchmark_results_chirho.txt
	sudo ./mcmc_sampling_bench_chirho | tee -a benchmark_results_chirho.txt
	@echo "\n=== Neurosymbolic Benchmarks ===" | tee -a benchmark_results_chirho.txt
	sudo ./neurosymbolic_bench_chirho | tee -a benchmark_results_chirho.txt

# Run MCMC benchmarks only (primary use case)
run-mcmc: mcmc_sampling_bench_chirho
	sudo ./mcmc_sampling_bench_chirho | tee mcmc_results_chirho.txt

# Run neurosymbolic benchmarks
run-neurosym: neurosymbolic_bench_chirho
	sudo ./neurosymbolic_bench_chirho | tee neurosym_results_chirho.txt

# Run hardware domain benchmarks
run: benchmark_suite_chirho
	sudo ./benchmark_suite_chirho | tee benchmark_results_chirho.txt

# Run CPU software baselines (no FPGA needed)
run-baseline: software_baseline_chirho
	./software_baseline_chirho | tee baseline_results_chirho.txt

# Run FPGA benchmarks then CPU baseline for comparison
compare: $(TARGETS)
	@echo "=============================================" | tee compare_results_chirho.txt
	@echo "FPGA vs CPU Comparison Benchmarks" | tee -a compare_results_chirho.txt
	@echo "=============================================" | tee -a compare_results_chirho.txt
	@echo "\n=== CPU Baseline ===" | tee -a compare_results_chirho.txt
	./software_baseline_chirho | tee -a compare_results_chirho.txt
	@echo "\n=== FPGA Hardware ===" | tee -a compare_results_chirho.txt
	sudo ./benchmark_suite_chirho | tee -a compare_results_chirho.txt
	@echo "\n=== FPGA MCMC ===" | tee -a compare_results_chirho.txt
	sudo ./mcmc_sampling_bench_chirho | tee -a compare_results_chirho.txt
	@echo "\nResults saved to compare_results_chirho.txt"

# Soli Deo Gloria ☧
