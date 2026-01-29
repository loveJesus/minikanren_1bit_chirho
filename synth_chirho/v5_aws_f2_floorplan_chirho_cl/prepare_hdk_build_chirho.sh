#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16
# Prepare AWS F2 HDK for miniKanren Build ☧
# ============================================================================
#
# This script copies our design files into the AWS F2 HDK structure
# and prepares for the full build with real HBM IP.
#
# Run this AFTER sourcing hdk_setup.sh:
#   source $AWS_FPGA_REPO_DIR/hdk_setup.sh
#   ./prepare_hdk_build_chirho.sh
# ============================================================================

set -e

SCRIPT_DIR_CHIRHO=$(cd "$(dirname "$0")" && pwd)

echo "============================================"
echo "Preparing AWS F2 HDK Build ☧"
echo "============================================"

# Check environment
if [ -z "$HDK_DIR" ]; then
    echo "ERROR: HDK_DIR not set. Please source hdk_setup.sh first."
    exit 1
fi

# Create CL directory if it doesn't exist
CL_TARGET_CHIRHO=$HDK_DIR/cl/examples/cl_minikanren_chirho
mkdir -p $CL_TARGET_CHIRHO/design
mkdir -p $CL_TARGET_CHIRHO/build/scripts
mkdir -p $CL_TARGET_CHIRHO/software
mkdir -p $CL_TARGET_CHIRHO/verif

echo "Copying design files to $CL_TARGET_CHIRHO/design/..."

# Copy our design files (NOT the stubs - those are for standalone synthesis)
cp $SCRIPT_DIR_CHIRHO/design/cl_minikanren_chirho.sv $CL_TARGET_CHIRHO/design/
cp $SCRIPT_DIR_CHIRHO/design/cl_minikanren_chirho_defines.vh $CL_TARGET_CHIRHO/design/
cp $SCRIPT_DIR_CHIRHO/design/searchEngineChirho.v $CL_TARGET_CHIRHO/design/

# Copy HDK-required files from common lib (these ARE the real implementations)
echo "Copying HBM wrapper from HDK common lib..."
cp $HDK_DIR/common/lib/cl_hbm_axi4.sv $CL_TARGET_CHIRHO/design/
cp $HDK_DIR/common/lib/cl_hbm_wrapper.sv $CL_TARGET_CHIRHO/design/
cp $HDK_DIR/common/lib/cl_dram_dma_defines.vh $CL_TARGET_CHIRHO/design/

# Create cl_id_defines.vh if not exists
if [ ! -f "$CL_TARGET_CHIRHO/design/cl_id_defines.vh" ]; then
    echo "Creating cl_id_defines.vh..."
    cat > "$CL_TARGET_CHIRHO/design/cl_id_defines.vh" << 'EOF'
// ============================================================================
// For God so loved the world - John 3:16
// PCIe Device IDs for miniKanren F2 Custom Logic ☧
// ============================================================================

`define CL_SH_ID0 32'hF216_1D0F   // {device_id, vendor_id}
`define CL_SH_ID1 32'h1BIT_FEED   // {subsystem_device_id, subsystem_vendor_id}
EOF
fi

# Copy cl_ports.vh from shell interface
echo "Copying shell interface definitions..."
cp $HDK_DIR/common/shell_stable/design/interfaces/cl_ports.vh $CL_TARGET_CHIRHO/design/

# Set CL_DIR for subsequent commands
export CL_DIR=$CL_TARGET_CHIRHO

echo ""
echo "============================================"
echo "HDK Build Prepared ☧"
echo "============================================"
echo ""
echo "CL_DIR: $CL_DIR"
echo ""
echo "Design files:"
ls -la $CL_DIR/design/
echo ""
echo "Next step: Run the HDK build:"
echo "  ./launch_hdk_build_chirho.sh"
echo ""
echo "Or manually:"
echo "  cd \$HDK_DIR/common/shell_stable/build/scripts"
echo "  python3 aws_build_dcp_from_cl.py -clock_recipe_a A2 -cl_dir \$CL_DIR -foreground"
