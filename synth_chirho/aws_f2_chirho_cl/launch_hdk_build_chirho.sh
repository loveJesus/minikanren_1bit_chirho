#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16
# Launch AWS F2 HDK Build with Real HBM IP ☧
# ============================================================================
#
# This script runs the FULL AWS HDK build flow on an AWS FPGA Developer AMI.
# It uses the real cl_hbm_wrapper and HBM IP, not stubs.
#
# Prerequisites:
#   - AWS FPGA Developer AMI (Rocky Linux 9 or Ubuntu with Vivado 2025.1)
#   - Source hdk_setup.sh before running
#   - c5.9xlarge or larger instance (72 GB RAM recommended for HBM builds)
#
# Usage:
#   source $HDK_DIR/hdk_setup.sh
#   ./launch_hdk_build_chirho.sh
# ============================================================================

set -e

# Divine header
echo "============================================"
echo "miniKanren F2 HBM Build Starting ☧"
echo "For God so loved the world - John 3:16"
echo "============================================"
date

# Check environment
if [ -z "$HDK_DIR" ]; then
    echo "ERROR: HDK_DIR not set. Please source hdk_setup.sh first:"
    echo "  source \$AWS_FPGA_REPO_DIR/hdk_setup.sh"
    exit 1
fi

if [ -z "$CL_DIR" ]; then
    export CL_DIR=$HDK_DIR/cl/examples/cl_minikanren_chirho
fi

echo "HDK_DIR: $HDK_DIR"
echo "CL_DIR: $CL_DIR"

# Verify design files exist
if [ ! -f "$CL_DIR/design/cl_minikanren_chirho.sv" ]; then
    echo "ERROR: Design files not found at $CL_DIR/design/"
    echo "Please copy design files first:"
    echo "  cp -r aws_f2_chirho_cl/design/* \$CL_DIR/design/"
    exit 1
fi

# Check for cl_id_defines.vh (required by AWS build)
if [ ! -f "$CL_DIR/design/cl_id_defines.vh" ]; then
    echo "Creating cl_id_defines.vh..."
    cat > "$CL_DIR/design/cl_id_defines.vh" << 'EOF'
// ============================================================================
// For God so loved the world - John 3:16
// PCIe Device IDs for miniKanren F2 Custom Logic
// ============================================================================
// Vendor ID: 0x1D0F (Amazon)
// Device ID: 0xF216 (Custom F2 device)
// Subsystem Vendor: 0xFEED (custom)
// Subsystem Device: 0x1BIT (1-bit logic)
// ============================================================================

`define CL_SH_ID0 32'hF216_1D0F   // {device_id, vendor_id}
`define CL_SH_ID1 32'h1BIT_FEED   // {subsystem_device_id, subsystem_vendor_id}
EOF
fi

# Create timestamp for this build
TIMESTAMP_CHIRHO=$(date +%Y_%m_%d-%H%M%S)
BUILD_DIR_CHIRHO=$CL_DIR/build/checkpoints/$TIMESTAMP_CHIRHO
mkdir -p $BUILD_DIR_CHIRHO

echo "Build output directory: $BUILD_DIR_CHIRHO"

# Run AWS HDK build
cd $HDK_DIR/common/shell_stable/build/scripts

echo ""
echo "Starting AWS HDK DCP build..."
echo "This will take 3-5 hours for HBM designs."
echo ""

python3 aws_build_dcp_from_cl.py \
    -clock_recipe_a A2 \
    -cl_dir $CL_DIR \
    -foreground \
    2>&1 | tee $BUILD_DIR_CHIRHO/build_log_chirho.txt

# Check for success
if [ -f "$CL_DIR/build/checkpoints/to_aws/*.Developer_CL.tar" ]; then
    echo ""
    echo "============================================"
    echo "BUILD SUCCESSFUL ☧"
    echo "============================================"
    echo ""
    echo "DCP tarball ready for AFI creation:"
    ls -la $CL_DIR/build/checkpoints/to_aws/*.Developer_CL.tar
    echo ""
    echo "Next steps:"
    echo "1. Upload to S3:"
    echo "   aws s3 cp \$(ls $CL_DIR/build/checkpoints/to_aws/*.Developer_CL.tar) s3://your-bucket/dcp/"
    echo ""
    echo "2. Create AFI:"
    echo "   aws ec2 create-fpga-image \\"
    echo "     --name miniKanren-HBM-chirho \\"
    echo "     --input-storage-location Bucket=your-bucket,Key=dcp/$(basename $(ls $CL_DIR/build/checkpoints/to_aws/*.Developer_CL.tar)) \\"
    echo "     --logs-storage-location Bucket=your-bucket,Key=logs/"
else
    echo ""
    echo "============================================"
    echo "BUILD FAILED - Check logs"
    echo "============================================"
    echo "Log file: $BUILD_DIR_CHIRHO/build_log_chirho.txt"
    exit 1
fi

echo ""
echo "Soli Deo Gloria ☧"
