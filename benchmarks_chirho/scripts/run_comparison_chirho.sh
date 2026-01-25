#!/bin/bash
# Run miniKanren comparison benchmarks ☧
#
# Outputs CSV format for easy analysis

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$BENCHMARK_DIR/../rust_chirho"
OUTPUT_FILE="$BENCHMARK_DIR/results_chirho.csv"

echo "=== miniKanren Comparison Benchmarks ☧ ==="
echo "Output: $OUTPUT_FILE"
echo ""

# CSV header
echo "implementation_chirho,benchmark_chirho,time_ms_chirho,solutions_chirho" > "$OUTPUT_FILE"

# Our implementation (Rust)
echo "Running our implementation (Rust)..."
if [ -f "$RUST_DIR/Cargo.toml" ]; then
    cd "$RUST_DIR"

    # Run appendo benchmark
    echo "  appendo..."
    RUST_OUTPUT=$(cargo bench --bench comparison_bench_chirho -- appendo 2>/dev/null | grep -E "time:" | head -1 || echo "")
    if [ -n "$RUST_OUTPUT" ]; then
        TIME=$(echo "$RUST_OUTPUT" | grep -oE '[0-9.]+' | head -1)
        echo "ours_rust_chirho,appendo,$TIME,6" >> "$OUTPUT_FILE"
    else
        echo "ours_rust_chirho,appendo,pending,6" >> "$OUTPUT_FILE"
    fi

    # Run nqueens benchmark
    echo "  nqueens..."
    RUST_OUTPUT=$(cargo bench --bench comparison_bench_chirho -- nqueens 2>/dev/null | grep -E "time:" | head -1 || echo "")
    if [ -n "$RUST_OUTPUT" ]; then
        TIME=$(echo "$RUST_OUTPUT" | grep -oE '[0-9.]+' | head -1)
        echo "ours_rust_chirho,nqueens,$TIME,92" >> "$OUTPUT_FILE"
    else
        echo "ours_rust_chirho,nqueens,pending,92" >> "$OUTPUT_FILE"
    fi

    cd - > /dev/null
else
    echo "  Rust project not found, skipping"
fi

# OCanren (if available)
echo "Running OCanren..."
if command -v ocamlfind &> /dev/null; then
    cd "$BENCHMARK_DIR/ocanren"
    # Note: Would need to compile and run OCanren benchmarks
    # This is a placeholder for the actual compilation/execution
    echo "ocanren_chirho,appendo,pending,6" >> "$OUTPUT_FILE"
    echo "ocanren_chirho,nqueens,pending,92" >> "$OUTPUT_FILE"
    cd - > /dev/null
else
    echo "  OCanren not available, recording as pending"
    echo "ocanren_chirho,appendo,pending,6" >> "$OUTPUT_FILE"
    echo "ocanren_chirho,nqueens,pending,92" >> "$OUTPUT_FILE"
fi

# faster-miniKanren (if Chez available)
echo "Running faster-miniKanren..."
if command -v chez &> /dev/null || command -v scheme &> /dev/null; then
    cd "$BENCHMARK_DIR/faster"
    # Note: Would run the Scheme files
    echo "faster_mk_chirho,appendo,pending,6" >> "$OUTPUT_FILE"
    echo "faster_mk_chirho,nqueens,pending,92" >> "$OUTPUT_FILE"
    cd - > /dev/null
else
    echo "  Chez Scheme not available, recording as pending"
    echo "faster_mk_chirho,appendo,pending,6" >> "$OUTPUT_FILE"
    echo "faster_mk_chirho,nqueens,pending,92" >> "$OUTPUT_FILE"
fi

echo ""
echo "=== Results ==="
cat "$OUTPUT_FILE"
echo ""
echo "Results saved to: $OUTPUT_FILE"
