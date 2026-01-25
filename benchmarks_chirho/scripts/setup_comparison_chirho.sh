#!/bin/bash
# Setup script for miniKanren comparison benchmarks ☧
#
# Installs:
# - OCanren (OCaml)
# - faster-miniKanren (Chez Scheme)
# - core.logic (Clojure) [optional]

set -e

echo "=== miniKanren Comparison Setup ☧ ==="

# Check for prerequisites
check_command() {
    if command -v "$1" &> /dev/null; then
        echo "✓ $1 found"
        return 0
    else
        echo "✗ $1 not found"
        return 1
    fi
}

echo ""
echo "Checking prerequisites..."

# OCaml / opam
if check_command opam; then
    echo "  Installing OCanren..."
    opam install OCanren -y 2>/dev/null || echo "  Note: OCanren may need manual installation from source"
else
    echo "  To install OCaml: brew install opam && opam init"
fi

# Chez Scheme
if check_command chez; then
    echo "  Chez Scheme ready for faster-miniKanren"
else
    if check_command scheme; then
        echo "  scheme found (may be Chez)"
    else
        echo "  To install Chez Scheme: brew install chezscheme"
    fi
fi

# Clone faster-miniKanren if not present
FASTER_DIR="../faster-miniKanren"
if [ -d "$FASTER_DIR" ]; then
    echo "✓ faster-miniKanren already cloned"
else
    echo "  Cloning faster-miniKanren..."
    git clone https://github.com/michaelballantyne/faster-miniKanren.git "$FASTER_DIR" 2>/dev/null || \
        echo "  Note: Clone failed, you may need to clone manually"
fi

# Clojure (optional)
if check_command clj; then
    echo "✓ Clojure CLI ready for core.logic"
else
    echo "  Clojure not found (optional). To install: brew install clojure/tools/clojure"
fi

# Rust (required)
if check_command cargo; then
    echo "✓ Cargo (Rust) ready"
    echo "  Building our benchmarks..."
    (cd ../../rust_chirho && cargo build --release --bench comparison_bench_chirho 2>/dev/null) || \
        echo "  Note: Benchmark not yet created"
else
    echo "✗ Cargo required for our implementation"
fi

echo ""
echo "=== Setup Complete ==="
echo ""
echo "To run comparisons:"
echo "  ./run_comparison_chirho.sh"
echo ""
echo "Manual steps if needed:"
echo "  1. OCanren: opam install OCanren"
echo "  2. faster-miniKanren: git clone https://github.com/michaelballantyne/faster-miniKanren"
echo "  3. core.logic: Add to deps.edn: org.clojure/core.logic {:mvn/version \"1.0.1\"}"
