#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16 ☧
# V5.3 Comprehensive Benchmark Suite
# Run on F2 instance after loading AFI
# ============================================================================

set -e

AFI_ID="afi-010cbb77b5413e1d6"
AGFI_ID="agfi-041630da370421d34"
SLOT=0

echo "=== miniKanren V5.3 FPGA Benchmark Suite ☧ ==="
echo "AFI: $AFI_ID"
echo "AGFI: $AGFI_ID"
date

# Check if we're on an F2 instance
if ! lspci | grep -q "Xilinx"; then
    echo "ERROR: Not on an FPGA instance or FPGA not detected"
    exit 1
fi

# Clear any existing AFI
echo "Clearing FPGA slot $SLOT..."
fpga-clear-local-image -S $SLOT

# Load our AFI
echo "Loading AFI $AFI_ID..."
fpga-load-local-image -S $SLOT -I $AGFI_ID

# Wait for AFI to load
echo "Waiting for AFI to become ready..."
while true; do
    STATUS=$(fpga-describe-local-image -S $SLOT -R -H | grep -o '"status": "[^"]*"' | head -1 | cut -d'"' -f4)
    if [ "$STATUS" == "loaded" ]; then
        echo "AFI loaded successfully!"
        break
    elif [ "$STATUS" == "load-failed" ]; then
        echo "ERROR: AFI load failed"
        fpga-describe-local-image -S $SLOT -R
        exit 1
    fi
    echo "Status: $STATUS - waiting..."
    sleep 2
done

# Show AFI details
fpga-describe-local-image -S $SLOT -R -H

# Create results directory
RESULTS_DIR="/tmp/benchmark_results_chirho"
mkdir -p $RESULTS_DIR
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo ""
echo "=== Compiling benchmarks ==="
cd /home/ec2-user/benchmarks_chirho

# Compile with AWS FPGA SDK
export SDK_DIR=/opt/aws/fpga/sdk
export INCLUDES="-I$SDK_DIR/userspace/include"
export LIBS="-L$SDK_DIR/userspace/lib/so -lfpga_mgmt -lrt -lpthread -lm"

echo "Compiling benchmark_suite_chirho.c..."
gcc -O3 $INCLUDES benchmark_suite_chirho.c $LIBS -o benchmark_suite_chirho

echo "Compiling neurosymbolic_bench_chirho.c..."
gcc -O3 $INCLUDES neurosymbolic_bench_chirho.c $LIBS -o neurosymbolic_bench_chirho

echo "Compiling mcmc_sampling_bench_chirho.c..."
gcc -O3 $INCLUDES mcmc_sampling_bench_chirho.c $LIBS -o mcmc_sampling_bench_chirho

echo "Compiling software_baseline_chirho.c..."
gcc -O3 software_baseline_chirho.c -lm -o software_baseline_chirho

echo ""
echo "=== Running Benchmarks ==="

# 1. Software baseline (CPU reference)
echo ""
echo "--- Software Baseline (CPU) ---"
./software_baseline_chirho | tee $RESULTS_DIR/cpu_baseline_${TIMESTAMP}.txt

# 2. Basic hierarchical operations
echo ""
echo "--- Hierarchical Domain Benchmarks ---"
./benchmark_suite_chirho | tee $RESULTS_DIR/hierarchical_${TIMESTAMP}.txt

# 3. Neurosymbolic training/inference
echo ""
echo "--- Neurosymbolic Training & Inference ---"
./neurosymbolic_bench_chirho | tee $RESULTS_DIR/neurosymbolic_${TIMESTAMP}.txt

# 4. MCMC/Probabilistic operations
echo ""
echo "--- MCMC & Probabilistic Inference ---"
./mcmc_sampling_bench_chirho | tee $RESULTS_DIR/mcmc_${TIMESTAMP}.txt

# Summarize results
echo ""
echo "=== Benchmark Summary ==="
echo "Results saved to $RESULTS_DIR/"
ls -la $RESULTS_DIR/

# Upload results to S3
echo ""
echo "Uploading results to S3..."
aws s3 cp $RESULTS_DIR/ s3://minikanren-fpga-chirho/benchmarks/v53_${TIMESTAMP}/ --recursive

echo ""
echo "=== Benchmark Complete ☧ ==="
date
