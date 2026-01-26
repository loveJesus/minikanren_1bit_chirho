#!/bin/bash
# FPGA Synthesis via Yosys (Open Source) ☧
#
# Uses Yosys open-source synthesis for resource estimation.
# Works natively on macOS via Homebrew.
#
# Install: brew install yosys
#
# Usage: ./synth_yosys_chirho.sh [clash|calyx]
#
# Soli Deo Gloria

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

TARGET="${1:-clash}"

echo "=== miniKanren FPGA Synthesis (Yosys) ☧ ==="
echo "Target: $TARGET"
echo ""

# Check Yosys
if ! command -v yosys &> /dev/null; then
    echo "Yosys not installed. Installing via Homebrew..."
    brew install yosys
fi

echo "Yosys version: $(yosys -V)"
echo ""

# Select source file
EXTRA_FLAGS=""

case "$TARGET" in
    clash)
        VERILOG_FILE="$PROJECT_ROOT/clash_chirho/verilog/MiniKanrenChirho.searchEngineChirho/searchEngineChirho.v"
        TOP_MODULE="searchEngineChirho"
        ;;
    calyx)
        # Calyx uses SystemVerilog and $fatal - need preprocessing
        VERILOG_FILE="/tmp/domain_clean_chirho.v"
        TOP_MODULE="main"
        # Strip $fatal (simulation-only construct) for synthesis
        echo "Preprocessing: stripping \$fatal from Calyx Verilog..."
        sed 's/\$fatal.*;//g' "$PROJECT_ROOT/calyx_chirho/domain_chirho.v" > "$VERILOG_FILE"
        EXTRA_FLAGS="-sv"
        ;;
    *)
        echo "Usage: $0 [clash|calyx]"
        exit 1
        ;;
esac

if [ ! -f "$VERILOG_FILE" ]; then
    echo "Error: Verilog file not found: $VERILOG_FILE"
    exit 1
fi

echo "Source: $VERILOG_FILE"
echo "Top module: $TOP_MODULE"
echo ""

# Create Yosys synthesis script
SYNTH_SCRIPT="$SCRIPT_DIR/synth_${TARGET}_chirho.ys"
cat > "$SYNTH_SCRIPT" << EOF
# Yosys Synthesis Script ☧
# Target: Generic (for resource estimation)

# Read Verilog
read_verilog $EXTRA_FLAGS $VERILOG_FILE

# Elaborate
hierarchy -top $TOP_MODULE

# Synthesize
synth -top $TOP_MODULE

# Map to generic gates for resource counting
techmap

# Optimize
opt -full

# Print statistics
stat

# Write output
write_verilog $SCRIPT_DIR/output_${TARGET}_chirho.v
EOF

echo "Running Yosys synthesis..."
echo ""

# Run synthesis and capture output
yosys -s "$SYNTH_SCRIPT" 2>&1 | tee "$SCRIPT_DIR/yosys_${TARGET}_report_chirho.txt"

echo ""
echo "=== Resource Summary ==="
grep -A 20 "Printing statistics" "$SCRIPT_DIR/yosys_${TARGET}_report_chirho.txt" || true

echo ""
echo "=== Synthesis complete ☧ ==="
echo "Full report: synth_chirho/yosys_${TARGET}_report_chirho.txt"
echo "Output Verilog: synth_chirho/output_${TARGET}_chirho.v"
