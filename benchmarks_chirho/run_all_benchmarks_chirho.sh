#!/bin/bash
# Run all paper benchmarks and save results ☧
#
# This script reproduces all performance claims in the paper.
# Results are saved to results_chirho/ for comparison with paper tables.
#
# Usage: ./run_all_benchmarks_chirho.sh

set -e

SCRIPT_DIR_CHIRHO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR_CHIRHO="$(dirname "$SCRIPT_DIR_CHIRHO")"
RESULTS_DIR_CHIRHO="$SCRIPT_DIR_CHIRHO/results_chirho"
RUST_DIR_CHIRHO="$PROJECT_DIR_CHIRHO/rust_chirho"

# Create results directory
mkdir -p "$RESULTS_DIR_CHIRHO"

# Timestamp for this run
TIMESTAMP_CHIRHO=$(date +"%Y%m%d_%H%M%S")
RESULT_FILE_CHIRHO="$RESULTS_DIR_CHIRHO/benchmark_results_${TIMESTAMP_CHIRHO}_chirho.json"

echo "=========================================="
echo "miniKanren 1-Bit Benchmark Suite ☧"
echo "=========================================="
echo "Date: $(date)"
echo "Results: $RESULT_FILE_CHIRHO"
echo ""

cd "$RUST_DIR_CHIRHO"

# Initialize JSON output
echo "{" > "$RESULT_FILE_CHIRHO"
echo "  \"timestamp_chirho\": \"$(date -Iseconds)\"," >> "$RESULT_FILE_CHIRHO"
echo "  \"platform_chirho\": \"$(uname -s) $(uname -r)\"," >> "$RESULT_FILE_CHIRHO"
echo "  \"benchmarks_chirho\": {" >> "$RESULT_FILE_CHIRHO"

# ==========================================
# Section 1: Microbenchmarks (Table 2)
# ==========================================
echo "--- Section 1: Microbenchmarks ---"

echo "    Running domain_and benchmark..."
if cargo bench --bench criterion_bench_chirho -- domain_and 2>/dev/null | grep -E "time:" | head -1 > /tmp/domain_and_chirho.txt 2>&1; then
    DOMAIN_AND_RESULT_CHIRHO=$(cat /tmp/domain_and_chirho.txt | head -1 || echo "N/A")
    echo "    Domain AND: $DOMAIN_AND_RESULT_CHIRHO"
fi

# ==========================================
# Section 2: Run Rust tests to verify correctness
# ==========================================
echo ""
echo "--- Section 2: Test Suite ---"
echo "    Running cargo test..."
TEST_RESULT_CHIRHO=$(cargo test 2>&1 | tail -5)
TEST_COUNT_CHIRHO=$(echo "$TEST_RESULT_CHIRHO" | grep -oE "[0-9]+ passed" | head -1 || echo "0 passed")
echo "    Tests: $TEST_COUNT_CHIRHO"

# ==========================================
# Section 3: Example executables
# ==========================================
echo ""
echo "--- Section 3: Examples ---"

echo "    Running profile_breakdown_chirho..."
if cargo run --release --example profile_breakdown_chirho 2>/dev/null > "$RESULTS_DIR_CHIRHO/profile_breakdown_${TIMESTAMP_CHIRHO}_chirho.txt"; then
    echo "    Profile breakdown saved"
fi

echo "    Running type_infer_chirho..."
if cargo run --release --example type_infer_chirho 2>/dev/null > "$RESULTS_DIR_CHIRHO/type_infer_${TIMESTAMP_CHIRHO}_chirho.txt"; then
    echo "    Type inference saved"
fi

echo "    Running learn_deep_chirho..."
if cargo run --release --example learn_deep_chirho 2>/dev/null > "$RESULTS_DIR_CHIRHO/learn_deep_${TIMESTAMP_CHIRHO}_chirho.txt"; then
    echo "    Gradient attenuation saved"
fi

echo "    Running synthesis_chirho..."
if cargo run --release --example synthesis_chirho 2>/dev/null > "$RESULTS_DIR_CHIRHO/synthesis_${TIMESTAMP_CHIRHO}_chirho.txt"; then
    echo "    SyGuS synthesis saved"
fi

echo "    Running symbolic_addition_chirho..."
if cargo run --release --example symbolic_addition_chirho 2>/dev/null > "$RESULTS_DIR_CHIRHO/symbolic_addition_${TIMESTAMP_CHIRHO}_chirho.txt"; then
    echo "    Symbolic addition saved"
fi

# ==========================================
# Section 4: Benchmark summary
# ==========================================
echo ""
echo "--- Section 4: Benchmark Suite ---"

# Run full criterion benchmarks
echo "    Running full criterion benchmark suite..."
if cargo bench --bench criterion_bench_chirho 2>/dev/null > "$RESULTS_DIR_CHIRHO/criterion_${TIMESTAMP_CHIRHO}_chirho.txt"; then
    echo "    Criterion benchmarks saved"
fi

# Close JSON
echo "    \"test_count_chirho\": \"$TEST_COUNT_CHIRHO\"" >> "$RESULT_FILE_CHIRHO"
echo "  }" >> "$RESULT_FILE_CHIRHO"
echo "}" >> "$RESULT_FILE_CHIRHO"

# ==========================================
# Summary
# ==========================================
echo ""
echo "=========================================="
echo "Benchmark Complete ☧"
echo "=========================================="
echo ""
echo "Results saved to:"
echo "  - $RESULT_FILE_CHIRHO (JSON summary)"
echo "  - $RESULTS_DIR_CHIRHO/profile_breakdown_${TIMESTAMP_CHIRHO}_chirho.txt"
echo "  - $RESULTS_DIR_CHIRHO/type_infer_${TIMESTAMP_CHIRHO}_chirho.txt"
echo "  - $RESULTS_DIR_CHIRHO/learn_deep_${TIMESTAMP_CHIRHO}_chirho.txt"
echo "  - $RESULTS_DIR_CHIRHO/synthesis_${TIMESTAMP_CHIRHO}_chirho.txt"
echo "  - $RESULTS_DIR_CHIRHO/criterion_${TIMESTAMP_CHIRHO}_chirho.txt"
echo ""
echo "Compare with paper tables in spec_chirho/traceability_chirho.md"
echo ""
echo "Soli Deo Gloria ☧"
