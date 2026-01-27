#!/bin/bash
# ============================================================================
# ☧ For God so loved the world, that He gave His only begotten Son,
# that whosoever believeth in Him should not perish, but have everlasting life.
# - John 3:16
# ============================================================================
#
# AWS F1 Build Script for miniKanren Custom Logic
# This script builds the CL design and creates an AFI-ready tarball.
#
# Usage: ./build_cl_chirho.sh [options]
#   -fg    Run in foreground (default is background)
#   -clock A0/A1/A2  Override clock recipe
# ============================================================================

set -e

# Colors for output
RED_CHIRHO='\033[0;31m'
GREEN_CHIRHO='\033[0;32m'
YELLOW_CHIRHO='\033[1;33m'
NC_CHIRHO='\033[0m' # No Color

echo -e "${GREEN_CHIRHO}=== miniKanren CL Build Started ☧ ===${NC_CHIRHO}"
date

# ============================================================================
# Environment Setup
# ============================================================================

if [ -z "$HDK_DIR" ]; then
    echo -e "${YELLOW_CHIRHO}HDK_DIR not set, sourcing hdk_setup.sh...${NC_CHIRHO}"
    if [ -f ~/aws-fpga/hdk_setup.sh ]; then
        source ~/aws-fpga/hdk_setup.sh
    elif [ -f /home/ubuntu/aws-fpga/hdk_setup.sh ]; then
        source /home/ubuntu/aws-fpga/hdk_setup.sh
    elif [ -f /home/rocky/aws-fpga/hdk_setup.sh ]; then
        source /home/rocky/aws-fpga/hdk_setup.sh
    else
        echo -e "${RED_CHIRHO}ERROR: Cannot find hdk_setup.sh${NC_CHIRHO}"
        exit 1
    fi
fi

# Set CL_DIR to our design
export CL_DIR=$(cd "$(dirname "$0")/../.." && pwd)
echo "CL_DIR: $CL_DIR"

# ============================================================================
# Copy Design Files
# ============================================================================

echo -e "${GREEN_CHIRHO}Setting up design files...${NC_CHIRHO}"

# Create the developer_designs directory if it doesn't exist
mkdir -p $HDK_DIR/cl/developer_designs

# Create symlink to our design (or copy)
if [ ! -L "$HDK_DIR/cl/developer_designs/cl_minikanren_chirho" ] && \
   [ ! -d "$HDK_DIR/cl/developer_designs/cl_minikanren_chirho" ]; then
    ln -sf $CL_DIR $HDK_DIR/cl/developer_designs/cl_minikanren_chirho
fi

# ============================================================================
# Create Filelist
# ============================================================================

mkdir -p $CL_DIR/build/scripts

cat > $CL_DIR/build/scripts/cl_filelist_chirho.tcl << 'EOF'
# miniKanren CL Filelist for Vivado
set CL_FILES_CHIRHO [list \
    "$CL_DIR/design/cl_minikanren_chirho_defines.vh" \
    "$CL_DIR/design/searchEngineChirho.v" \
    "$CL_DIR/design/cl_minikanren_chirho.sv" \
]
EOF

# ============================================================================
# Create Constraints
# ============================================================================

mkdir -p $CL_DIR/build/constraints

cat > $CL_DIR/build/constraints/cl_timing_chirho.xdc << 'EOF'
# ============================================================================
# miniKanren CL Timing Constraints
# ============================================================================

# The engine runs on a divided clock (62.5MHz from 250MHz)
# Main shell clock is 250MHz (4ns period)

# False path for clock divider
set_false_path -from [get_pins -hier -filter {NAME =~ */clk_div_chirho_reg*/C}] \
               -to [get_pins -hier -filter {NAME =~ */u_engine_chirho/*/D}]

# Multicycle path for slow engine clock domain
set_multicycle_path 4 -setup -from [get_pins -hier -filter {NAME =~ */u_engine_chirho/*/C}] \
                             -to [get_pins -hier -filter {NAME =~ */u_engine_chirho/*/D}]
set_multicycle_path 3 -hold  -from [get_pins -hier -filter {NAME =~ */u_engine_chirho/*/C}] \
                             -to [get_pins -hier -filter {NAME =~ */u_engine_chirho/*/D}]
EOF

# ============================================================================
# Run Build
# ============================================================================

echo -e "${GREEN_CHIRHO}Starting Vivado build...${NC_CHIRHO}"

cd $CL_DIR/build/scripts

# Check if AWS build script exists
if [ -f "$HDK_DIR/cl/examples/cl_hello_world/build/scripts/aws_build_dcp_from_cl.sh" ]; then
    BUILD_SCRIPT_CHIRHO="$HDK_DIR/cl/examples/cl_hello_world/build/scripts/aws_build_dcp_from_cl.sh"
else
    BUILD_SCRIPT_CHIRHO="$HDK_SHELL_DIR/build/scripts/aws_build_dcp_from_cl.sh"
fi

# Parse arguments
FOREGROUND_CHIRHO=""
CLOCK_A_CHIRHO="A0"

while [[ $# -gt 0 ]]; do
    case $1 in
        -fg)
            FOREGROUND_CHIRHO="-foreground"
            shift
            ;;
        -clock)
            CLOCK_A_CHIRHO="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

# Run the build
$BUILD_SCRIPT_CHIRHO -cl_dir $CL_DIR \
              -clock_recipe_a $CLOCK_A_CHIRHO \
              -clock_recipe_b B0 \
              -clock_recipe_c C0 \
              $FOREGROUND_CHIRHO

echo -e "${GREEN_CHIRHO}=== Build Command Issued ===${NC_CHIRHO}"
echo "Check logs in: $CL_DIR/build/logs/"
echo "Results will be in: $CL_DIR/build/checkpoints/"
