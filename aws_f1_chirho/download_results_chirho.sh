#!/bin/bash
# Download Synthesis Results ☧
#
# Downloads all artifacts from S3 after synthesis completes.
# Part of P5-01: Physical FPGA Incarnation.

set -euo pipefail

SCRIPT_DIR_CHIRHO="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT_CHIRHO="$(dirname "${SCRIPT_DIR_CHIRHO}")"

# Load configuration
if [ -f "${SCRIPT_DIR_CHIRHO}/config_chirho.sh" ]; then
    source "${SCRIPT_DIR_CHIRHO}/config_chirho.sh"
else
    echo "ERROR: Run setup_chirho.sh first"
    exit 1
fi

# Create results directory
RESULTS_DIR_CHIRHO="${SCRIPT_DIR_CHIRHO}/results_chirho"
mkdir -p "${RESULTS_DIR_CHIRHO}"

echo "=== Download Synthesis Results ☧ ==="
echo "Bucket: ${S3_BUCKET_CHIRHO}"
echo "Destination: ${RESULTS_DIR_CHIRHO}"
echo ""

# Check what's available
echo "Available files:"
aws s3 ls "s3://${S3_BUCKET_CHIRHO}/results/" || echo "  (no results yet)"
echo ""

# Download all results
echo "Downloading..."
aws s3 sync "s3://${S3_BUCKET_CHIRHO}/results/" "${RESULTS_DIR_CHIRHO}/"

echo ""
echo "=== Downloaded Files ☧ ==="
ls -la "${RESULTS_DIR_CHIRHO}/"

# Copy key files to synth_chirho for record
if [ -f "${RESULTS_DIR_CHIRHO}/timing_chirho.rpt" ]; then
    echo ""
    echo "Copying to synth_chirho/ for project record..."
    cp "${RESULTS_DIR_CHIRHO}/timing_chirho.rpt" "${PROJECT_ROOT_CHIRHO}/synth_chirho/vivado_timing_chirho.rpt"
    cp "${RESULTS_DIR_CHIRHO}/utilization_chirho.rpt" "${PROJECT_ROOT_CHIRHO}/synth_chirho/vivado_utilization_chirho.rpt"
    echo "  Copied timing and utilization reports"
fi

# Extract timing summary
if [ -f "${RESULTS_DIR_CHIRHO}/timing_chirho.rpt" ]; then
    echo ""
    echo "=== Timing Summary ==="
    grep -A5 "Design Timing Summary" "${RESULTS_DIR_CHIRHO}/timing_chirho.rpt" || true
    grep "WNS" "${RESULTS_DIR_CHIRHO}/timing_chirho.rpt" || true
fi

echo ""
echo "=== Download Complete ☧ ==="
