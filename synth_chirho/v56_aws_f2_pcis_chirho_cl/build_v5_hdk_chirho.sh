#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16 ☧
# v5.4 Build: Debug registers + width fix (2026-01-30)
#
# Changes from v5.3:
#   - Fixed register map conflicts (HIER/TRAIN moved to 0x80+)
#   - Fixed width bug in find_next_set_bit_chirho (use <= with 255/511)
#   - Added debug registers at 0xC0-0xD4 (FSM state, AXI status, etc.)
#   - Device ID 0xF054, Version 0xF2540001
#
# V5.3 changes carried forward:
#   - Sparse streaming: reduces FFs from ~983K to ~3K
#   - 200MHz clock (A1 recipe)
#   - HBM exclusion from pblock_CL
#
# MEMORY REQUIREMENT: 72GB RAM sufficient (c5.9xlarge)
# ============================================================================
set -x
exec > >(tee /var/log/hdk-build-v5.4-chirho.log) 2>&1
echo "=== v5.4 DEBUG REGISTERS Build Starting ☧ ==="
date

export HOME=/root
export AWS_DEFAULT_REGION=us-east-1

BUCKET_CHIRHO=minikanren-fpga-chirho
WORK_DIR_CHIRHO=/root/hdk_build_chirho

mkdir -p $WORK_DIR_CHIRHO
cd $WORK_DIR_CHIRHO

git clone --depth 1 https://github.com/aws/aws-fpga.git
cd aws-fpga

source /opt/Xilinx/2025.1/Vivado/settings64.sh
source hdk_setup.sh

export CL_DIR=$HDK_DIR/cl/examples/cl_minikanren_chirho
mkdir -p $CL_DIR/design
mkdir -p $CL_DIR/build/scripts
mkdir -p $CL_DIR/build/checkpoints
mkdir -p $CL_DIR/build/constraints

# Download v5.4 design files (debug registers + width fix)
aws s3 cp s3://$BUCKET_CHIRHO/f2_hbm_hdk/design_v5.4_chirho.tar.gz /tmp/design.tar.gz
tar -xzf /tmp/design.tar.gz -C $CL_DIR/design/

# Copy HBM wrappers from example
HBM_EXAMPLE_DIR=$HDK_DIR/cl/examples/cl_dram_hbm_dma
cp $HBM_EXAMPLE_DIR/design/cl_hbm_axi4.sv $CL_DIR/design/
cp $HBM_EXAMPLE_DIR/design/cl_hbm_wrapper.sv $CL_DIR/design/
cp $HBM_EXAMPLE_DIR/design/cl_dram_dma_pkg.sv $CL_DIR/design/
cp $HDK_DIR/common/shell_stable/design/interfaces/cl_ports.vh $CL_DIR/design/

if [ -f "$HBM_EXAMPLE_DIR/design/cl_dram_dma_defines.vh" ]; then
    cp $HBM_EXAMPLE_DIR/design/cl_dram_dma_defines.vh $CL_DIR/design/
fi

# VALID PCI ID: 0xF054 (AWS valid range 0xF000-0xF0FF, v5.4 = 54)
# Note: cl_id_defines.vh is now included in the tarball, but we create it here
# to ensure the correct version is used even if tarball is updated
cat > "$CL_DIR/design/cl_id_defines.vh" << 'IDEOF'
// ============================================================================
// For God so loved the world - John 3:16 ☧
// v5.4: Debug registers + width fix
// PCI DeviceID 0xF054 = valid AWS range + version 5.4
// ============================================================================
`define CL_SH_ID0 32'hF054_1D0F
`define CL_SH_ID1 32'h1D51_F054
IDEOF

# Build script symlinks
cd $CL_DIR/build/scripts
ln -sf $HDK_DIR/common/shell_stable/build/scripts/aws_build_dcp_from_cl.py .
ln -sf $HDK_DIR/common/shell_stable/build/scripts/build_all.tcl .
ln -sf $HDK_DIR/common/shell_stable/build/scripts/build_level_1_cl.tcl .

# Create synthesis TCL
cat > "$CL_DIR/build/scripts/synth_cl_minikanren_chirho.tcl" << 'SYNTHTCL'
source ${HDK_SHELL_DIR}/build/scripts/synth_cl_header.tcl
print "Reading user source codes - v5.4 Debug Registers + Width Fix ☧"

read_verilog -sv ${src_post_enc_dir}/cl_dram_dma_pkg.sv
read_verilog -sv [glob ${src_post_enc_dir}/*.sv]
read_verilog [glob ${src_post_enc_dir}/*.v]

read_verilog -sv ${src_post_enc_dir}/cl_minikanren_chirho_defines.vh
set_property file_type {Verilog Header} [get_files ${src_post_enc_dir}/cl_minikanren_chirho_defines.vh]
set_property is_global_include true [get_files ${src_post_enc_dir}/cl_minikanren_chirho_defines.vh]

if {[file exists ${src_post_enc_dir}/cl_dram_dma_defines.vh]} {
    read_verilog -sv ${src_post_enc_dir}/cl_dram_dma_defines.vh
    set_property file_type {Verilog Header} [get_files ${src_post_enc_dir}/cl_dram_dma_defines.vh]
    set_property is_global_include true [get_files ${src_post_enc_dir}/cl_dram_dma_defines.vh]
}

print "Reading CL IP blocks"
read_ip [ list \
  ${HDK_IP_SRC_DIR}/cl_hbm_mmcm/cl_hbm_mmcm.xci \
  ${HDK_IP_SRC_DIR}/cl_hbm/cl_hbm.xci \
]
read_ip [ list \
  $HDK_SHELL_DESIGN_DIR/../../ip/cl_ip/cl_ip.srcs/sources_1/ip/clk_mmcm_a/clk_mmcm_a.xci \
  $HDK_SHELL_DESIGN_DIR/../../ip/cl_ip/cl_ip.srcs/sources_1/ip/clk_mmcm_b/clk_mmcm_b.xci \
  $HDK_SHELL_DESIGN_DIR/../../ip/cl_ip/cl_ip.srcs/sources_1/ip/clk_mmcm_c/clk_mmcm_c.xci \
  $HDK_SHELL_DESIGN_DIR/../../ip/cl_ip/cl_ip.srcs/sources_1/ip/clk_mmcm_hbm/clk_mmcm_hbm.xci \
  $HDK_SHELL_DESIGN_DIR/../../ip/cl_ip/cl_ip.srcs/sources_1/ip/cl_clk_axil_xbar/cl_clk_axil_xbar.xci \
  $HDK_SHELL_DESIGN_DIR/../../ip/cl_ip/cl_ip.srcs/sources_1/ip/cl_sda_axil_xbar/cl_sda_axil_xbar.xci \
]
read_ip [ list \
  ${HDK_IP_SRC_DIR}/axi_register_slice/axi_register_slice.xci \
  ${HDK_IP_SRC_DIR}/axi_register_slice_light/axi_register_slice_light.xci \
  ${HDK_IP_SRC_DIR}/cl_axi3_256b_reg_slice/cl_axi3_256b_reg_slice.xci \
]
read_ip [ list \
  ${HDK_IP_SRC_DIR}/cl_axi_clock_converter/cl_axi_clock_converter.xci \
  ${HDK_IP_SRC_DIR}/cl_axi_clock_converter_light/cl_axi_clock_converter_light.xci \
]
read_ip [ list \
  ${HDK_IP_SRC_DIR}/cl_axi_interconnect_64G_ddr/cl_axi_interconnect_64G_ddr.xci \
]
add_files [ list \
  ${HDK_BD_SRC_DIR}/cl_axi_sc_1x1/cl_axi_sc_1x1.bd \
  ${HDK_BD_SRC_DIR}/cl_axi_sc_2x2/cl_axi_sc_2x2.bd \
]
read_verilog [ list \
  ${HDK_BD_GEN_DIR}/cl_axi_sc_1x1/hdl/cl_axi_sc_1x1_wrapper.v \
  ${HDK_BD_GEN_DIR}/cl_axi_sc_2x2/hdl/cl_axi_sc_2x2_wrapper.v \
]

print "Reading user constraints"
read_xdc [ list \
  ${constraints_dir}/cl_synth_user.xdc \
  ${constraints_dir}/cl_timing_user.xdc \
]
set_property PROCESSING_ORDER LATE [get_files cl_synth_user.xdc]
set_property PROCESSING_ORDER LATE [get_files cl_timing_user.xdc]

print "Starting synthesizing customer design ${CL} - v5.4 ☧"
update_compile_order -fileset sources_1
synth_design -mode out_of_context \
             -top ${CL} \
             -verilog_define XSDB_SLV_DIS \
             -part ${DEVICE_TYPE} \
             -keep_equivalent_registers
source ${HDK_SHELL_DIR}/build/scripts/synth_cl_footer.tcl
SYNTHTCL

# ============================================================================
# FLOORPLANNING CONSTRAINTS - v5.1 ☧
# NO pblock constraints - HBM IP has fixed placement sites
# Let SSI_SpreadLogic_high directive handle placement naturally
# ============================================================================
cat > "$CL_DIR/build/constraints/cl_synth_user.xdc" << 'XDCEOF'
# ============================================================================
# v5.1 Constraints (2026-01-29) ☧
# NO pblock - HBM IP requires fixed hardware sites that conflict with pblocks
# Vivado SSI_SpreadLogic_high directive handles placement
# ============================================================================

# No pblock constraints - HBM hard macros have fixed locations
# The SSI_SpreadLogic_high directive will spread logic across SLRs automatically
XDCEOF

cat > "$CL_DIR/build/constraints/cl_timing_user.xdc" << 'XDCEOF'
# ============================================================================
# v5 Timing Constraints ☧
# Target: 200MHz (5ns period) - relaxed from v4's 250MHz
# ============================================================================

# HBM async paths - CDC handled by HBM IP synchronizers
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*WREADY*}]
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*RREADY*}]
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*BREADY*}]

# SLR crossing paths get extra slack (1-2ns crossing delay expected)
set_multicycle_path 2 -setup -through [get_pins -hierarchical -filter {NAME =~ *SLR*}] -quiet
set_multicycle_path 1 -hold -through [get_pins -hierarchical -filter {NAME =~ *SLR*}] -quiet
XDCEOF

cat > "$CL_DIR/build/constraints/small_shell_cl_pnr_user.xdc" << 'PNREOF'
# ============================================================================
# v5.2 P&R constraints - HBM exclusion from pblock_CL ☧
# ============================================================================
# Problem: AWS shell creates pblock_CL for reconfigurable CL region.
# HBM IP has fixed physical sites (BLI_HBM_APB_INTF) at chip edges,
# which are outside pblock_CL bounds. We must exclude HBM from pblock_CL.
# ============================================================================

# Remove HBM cells from the CL pblock
# HBM hard macros must be placed at their fixed physical locations
# These sites are at the bottom of the VU47P die, outside pblock_CL
set hbm_cells [get_cells -hierarchical -filter {NAME =~ *HBM*} -quiet]
if {[llength $hbm_cells] > 0} {
    # Clear any pblock assignment for HBM cells
    foreach cell $hbm_cells {
        set pblock [get_pblocks -of_objects $cell -quiet]
        if {[llength $pblock] > 0} {
            remove_cells_from_pblock $pblock $cell
        }
    }
}

# Aggressive optimization directives for timing
set_property STEPS.PHYS_OPT_DESIGN.ARGS.DIRECTIVE AggressiveExplore [get_runs impl_1]
set_property STEPS.ROUTE_DESIGN.ARGS.DIRECTIVE AggressiveExplore [get_runs impl_1]
PNREOF

# encrypt.tcl - v5 REVISED: Only flat + 65K + 262K (no 16M/134M)
cat > "$CL_DIR/build/scripts/encrypt.tcl" << 'ENCEOF'
if {[llength [glob -nocomplain -dir $src_post_enc_dir *]] != 0} {
  eval file delete -force [glob $src_post_enc_dir/*]
}

# Header files
file copy -force $CL_DIR/design/cl_minikanren_chirho_defines.vh $src_post_enc_dir
file copy -force $CL_DIR/design/cl_id_defines.vh $src_post_enc_dir
file copy -force $CL_DIR/design/cl_ports.vh $src_post_enc_dir

# HDK interface files
if {[file exists $CL_DIR/design/cl_dram_dma_pkg.sv]} {
    file copy -force $CL_DIR/design/cl_dram_dma_pkg.sv $src_post_enc_dir
}
if {[file exists $CL_DIR/design/cl_dram_dma_defines.vh]} {
    file copy -force $CL_DIR/design/cl_dram_dma_defines.vh $src_post_enc_dir
}
file copy -force $CL_DIR/design/cl_hbm_axi4.sv $src_post_enc_dir
file copy -force $CL_DIR/design/cl_hbm_wrapper.sv $src_post_enc_dir

# Core design
file copy -force $CL_DIR/design/cl_minikanren_chirho.sv $src_post_enc_dir

# Search engines (Clash-generated)
file copy -force $CL_DIR/design/searchEngineChirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/searchEngine64BitChirho.v $src_post_enc_dir

# Hierarchical intersection modules - V5: ONLY 65K + 262K
file copy -force $CL_DIR/design/intersect_512_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_hier_65k_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_hier_262k_chirho.v $src_post_enc_dir
# EXCLUDED in V5: 16M and 134M streaming hierarchies (timing failures)

# Neurosymbolic modules
file copy -force $CL_DIR/design/diffTrainChirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/soft_and_32_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/soft_and_16_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_prob_domain_64_chirho.v $src_post_enc_dir
ENCEOF

# ============================================================================
# v5.3 BUILD: 200MHz (A1 recipe), Sparse Streaming (~3K FFs instead of ~983K)
# ============================================================================
echo "Starting v5.3 HDK build (200 MHz, A1 recipe, sparse streaming)..."
python3 $HDK_DIR/common/shell_stable/build/scripts/aws_build_dcp_from_cl.py \
    --cl cl_minikanren_chirho \
    --aws_clk_gen \
    --clock_recipe_a A1 \
    2>&1 | tee $WORK_DIR_CHIRHO/build_v5_chirho.log

cd $CL_DIR/build
DCP_TAR=$(find . -name "*.Developer_CL.tar" 2>/dev/null | head -1)
if [ -n "$DCP_TAR" ]; then
    aws s3 cp "$DCP_TAR" s3://$BUCKET_CHIRHO/f2_hbm_hdk/dcp_v5_floorplan/
    echo "v5_floorplan_success" > /tmp/build_status.txt
else
    echo "v5_floorplan_failed" > /tmp/build_status.txt
fi

aws s3 cp $WORK_DIR_CHIRHO/build_v5_chirho.log s3://$BUCKET_CHIRHO/f2_hbm_hdk/build_v5_chirho.log
aws s3 cp /tmp/build_status.txt s3://$BUCKET_CHIRHO/f2_hbm_hdk/build_v5_status_chirho.txt
aws s3 cp /var/log/hdk-build-v5-chirho.log s3://$BUCKET_CHIRHO/f2_hbm_hdk/userdata_v5_chirho.log

echo "=== v5.3 Sparse Streaming + 200MHz Build Complete ☧ ==="
date

# Keep instance alive for 30 min to check results, then shutdown
sleep 1800
shutdown -h now
