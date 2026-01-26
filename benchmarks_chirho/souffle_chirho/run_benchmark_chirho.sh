#!/bin/bash
# Soufflé Transitive Closure Benchmark ☧
#
# Runs TC on different graph sizes and reports timing
# "To every thing there is a season" — Ecclesiastes 3:1

cd "$(dirname "$0")"

echo "☧ Soufflé Transitive Closure Benchmark ☧"
echo ""

for size in 1k 10k 100k; do
    INPUT="edge_chirho_${size}.facts"
    OUTPUT="path_chirho.csv"

    if [ ! -f "$INPUT" ]; then
        echo "Generating $INPUT..."
        python3 generate_graphs_chirho.py
    fi

    # Create temp directory with correct input name
    TMP_DIR=$(mktemp -d)
    cp "$INPUT" "$TMP_DIR/edge_chirho.facts"

    echo "=== Graph: $size edges ==="

    # Compile and run with timing
    START=$(python3 -c "import time; print(time.time())")

    souffle tc_chirho.dl -F "$TMP_DIR" -D "$TMP_DIR" 2>/dev/null

    END=$(python3 -c "import time; print(time.time())")
    ELAPSED=$(python3 -c "print(f'{($END - $START) * 1000:.2f}')")

    # Count output paths
    if [ -f "$TMP_DIR/path_chirho.csv" ]; then
        PATHS=$(wc -l < "$TMP_DIR/path_chirho.csv")
        echo "  Paths found: $PATHS"
    fi

    echo "  Time: ${ELAPSED}ms"
    echo ""

    rm -rf "$TMP_DIR"
done

echo "☧ Soli Deo Gloria ☧"
