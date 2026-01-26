#!/bin/bash
# FPGA Synthesis via Docker ☧
#
# Runs Quartus Prime Lite in Docker to synthesize the miniKanren search engine.
# No local Quartus installation required.
#
# Usage: ./synth_docker_chirho.sh
#
# Output: Resource utilization report in synth_report_chirho.txt
#
# Soli Deo Gloria

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== miniKanren FPGA Synthesis ☧ ==="
echo "Project: $PROJECT_ROOT"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "Error: Docker not installed"
    echo "Install: https://www.docker.com/products/docker-desktop/"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo "Error: Docker daemon not running"
    echo "Start Docker Desktop and try again"
    exit 1
fi

echo "Pulling Quartus Prime Lite Docker image..."
echo "(This may take a while on first run - ~15GB)"
echo ""

# Note: Intel's official image may require registration
# Alternative: use a community image or build from Dockerfile
QUARTUS_IMAGE="intel/quartus-prime-lite:23.1"

# Check if image exists, if not provide instructions
if ! docker image inspect "$QUARTUS_IMAGE" &> /dev/null 2>&1; then
    echo "Quartus Docker image not found."
    echo ""
    echo "Option 1: Pull from Intel (requires registration)"
    echo "  docker pull $QUARTUS_IMAGE"
    echo ""
    echo "Option 2: Use EDA Playground (free, web-based)"
    echo "  https://www.edaplayground.com/"
    echo "  - Select 'Intel Quartus' as simulator"
    echo "  - Upload searchEngineChirho.v"
    echo "  - Run synthesis"
    echo ""
    echo "Option 3: Install Quartus locally in a Linux VM"
    echo "  https://www.intel.com/content/www/us/en/products/details/fpga/development-tools/quartus-prime/resource.html"
    echo ""
    exit 1
fi

echo "Running synthesis..."
docker run --rm \
    -v "$PROJECT_ROOT:/work" \
    -w /work/synth_chirho \
    "$QUARTUS_IMAGE" \
    quartus_sh --flow compile minikanren_chirho

echo ""
echo "=== Extracting resource report ==="

# Extract key metrics from the fit report
docker run --rm \
    -v "$PROJECT_ROOT:/work" \
    -w /work/synth_chirho \
    "$QUARTUS_IMAGE" \
    bash -c 'cat output_files/minikanren_chirho.fit.summary 2>/dev/null || echo "Fit summary not found"' \
    | tee synth_report_chirho.txt

echo ""
echo "=== Synthesis complete ☧ ==="
echo "Report saved to: synth_chirho/synth_report_chirho.txt"
