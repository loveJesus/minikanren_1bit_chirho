#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16 ☧
# V5.7 Build: 125MHz Clock to Fix Timing Violation (2026-02-01)
#
# PROBLEM: V5.6 had -2.506ns WNS at 250MHz, causing PCIS instability
# SOLUTION: Reduce to 125MHz (8ns period), giving +1.5ns positive slack
#
# Changes from V5.6:
#   - Clock recipe A0 (125MHz) instead of A2 (250MHz)
#   - Device ID 0xF057, Version 0xF2570001
#   - Same PCIS→HBM connectivity as V5.6
#
# TIMING ANALYSIS:
#   - Critical path: 6.5ns (measured from V5.6 WNS = 4.0ns - (-2.5ns))
#   - At 250MHz (4.0ns period): -2.5ns violation
#   - At 200MHz (5.0ns period): -1.5ns violation
#   - At 125MHz (8.0ns period): +1.5ns POSITIVE SLACK ✓
#
# MEMORY REQUIREMENT: Try c5.4xlarge (32GB RAM) first, c5.9xlarge if OOM
# ============================================================================
set -x
exec > >(tee /var/log/hdk-build-v57-125mhz-chirho.log) 2>&1
echo "=== V5.7 125MHz Build Starting ☧ ==="
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

# Download V5.7 design files (same as V5.6 but with new version ID)
# NOTE: Tarball contains design/ directory, so extract to $CL_DIR/ not $CL_DIR/design/
aws s3 cp s3://$BUCKET_CHIRHO/f2_hbm_hdk/design_v5.7_125mhz_chirho.tar.gz /tmp/design.tar.gz
tar -xzf /tmp/design.tar.gz -C $CL_DIR/

# Copy HBM wrappers from example
HBM_EXAMPLE_DIR=$HDK_DIR/cl/examples/cl_dram_hbm_dma
cp $HBM_EXAMPLE_DIR/design/cl_hbm_axi4.sv $CL_DIR/design/
cp $HBM_EXAMPLE_DIR/design/cl_hbm_wrapper.sv $CL_DIR/design/
cp $HBM_EXAMPLE_DIR/design/cl_dram_dma_pkg.sv $CL_DIR/design/
cp $HDK_DIR/common/shell_stable/design/interfaces/cl_ports.vh $CL_DIR/design/

if [ -f "$HBM_EXAMPLE_DIR/design/cl_dram_dma_defines.vh" ]; then
    cp $HBM_EXAMPLE_DIR/design/cl_dram_dma_defines.vh $CL_DIR/design/
fi

# VALID PCI ID: 0xF057 (AWS valid range 0xF000-0xF0FF, v5.7 = 57)
cat > "$CL_DIR/design/cl_id_defines.vh" << 'IDEOF'
// ============================================================================
// For God so loved the world - John 3:16 ☧
// V5.7: 125MHz clock to fix timing violation
// PCI DeviceID 0xF057 = valid AWS range + version 5.7
// ============================================================================
`define CL_SH_ID0 32'hF057_1D0F
`define CL_SH_ID1 32'h1D51_F057
IDEOF

# Build script symlinks
cd $CL_DIR/build/scripts
ln -sf $HDK_DIR/common/shell_stable/build/scripts/aws_build_dcp_from_cl.py .
ln -sf $HDK_DIR/common/shell_stable/build/scripts/build_all.tcl .
ln -sf $HDK_DIR/common/shell_stable/build/scripts/build_level_1_cl.tcl .

# Create synthesis TCL (same as V5.6)
cat > "$CL_DIR/build/scripts/synth_cl_minikanren_chirho.tcl" << 'SYNTHTCL'
source ${HDK_SHELL_DIR}/build/scripts/synth_cl_header.tcl
print "Reading user source codes - V5.7 125MHz Timing Fix ☧"

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

print "Starting synthesizing customer design ${CL} - V5.7 125MHz ☧"
update_compile_order -fileset sources_1
synth_design -mode out_of_context \
             -top ${CL} \
             -verilog_define XSDB_SLV_DIS \
             -part ${DEVICE_TYPE} \
             -keep_equivalent_registers
source ${HDK_SHELL_DIR}/build/scripts/synth_cl_footer.tcl
SYNTHTCL

# Constraints (same as V5.6)
cat > "$CL_DIR/build/constraints/cl_synth_user.xdc" << 'XDCEOF'
# ============================================================================
# V5.7 Constraints (2026-02-01) ☧
# NO pblock - HBM IP requires fixed hardware sites that conflict with pblocks
# ============================================================================
XDCEOF

cat > "$CL_DIR/build/constraints/cl_timing_user.xdc" << 'XDCEOF'
# ============================================================================
# V5.7 Timing Constraints ☧
# Target: 125MHz (8ns period) - relaxed from V5.6's 250MHz to fix timing
# ============================================================================
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*WREADY*}]
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*RREADY*}]
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*BREADY*}]
set_multicycle_path 2 -setup -through [get_pins -hierarchical -filter {NAME =~ *SLR*}] -quiet
set_multicycle_path 1 -hold -through [get_pins -hierarchical -filter {NAME =~ *SLR*}] -quiet
XDCEOF

cat > "$CL_DIR/build/constraints/small_shell_cl_pnr_user.xdc" << 'PNREOF'
# ============================================================================
# V5.7 P&R constraints - HBM exclusion from pblock_CL ☧
# ============================================================================
set hbm_cells [get_cells -hierarchical -filter {NAME =~ *HBM*} -quiet]
if {[llength $hbm_cells] > 0} {
    foreach cell $hbm_cells {
        set pblock [get_pblocks -of_objects $cell -quiet]
        if {[llength $pblock] > 0} {
            remove_cells_from_pblock $pblock $cell
        }
    }
}
set_property STEPS.PHYS_OPT_DESIGN.ARGS.DIRECTIVE AggressiveExplore [get_runs impl_1]
set_property STEPS.ROUTE_DESIGN.ARGS.DIRECTIVE AggressiveExplore [get_runs impl_1]
PNREOF

# encrypt.tcl (same as V5.6)
cat > "$CL_DIR/build/scripts/encrypt.tcl" << 'ENCEOF'
if {[llength [glob -nocomplain -dir $src_post_enc_dir *]] != 0} {
  eval file delete -force [glob $src_post_enc_dir/*]
}
file copy -force $CL_DIR/design/cl_minikanren_chirho_defines.vh $src_post_enc_dir
file copy -force $CL_DIR/design/cl_id_defines.vh $src_post_enc_dir
file copy -force $CL_DIR/design/cl_ports.vh $src_post_enc_dir
if {[file exists $CL_DIR/design/cl_dram_dma_pkg.sv]} {
    file copy -force $CL_DIR/design/cl_dram_dma_pkg.sv $src_post_enc_dir
}
if {[file exists $CL_DIR/design/cl_dram_dma_defines.vh]} {
    file copy -force $CL_DIR/design/cl_dram_dma_defines.vh $src_post_enc_dir
}
file copy -force $CL_DIR/design/cl_hbm_axi4.sv $src_post_enc_dir
file copy -force $CL_DIR/design/cl_hbm_wrapper.sv $src_post_enc_dir
file copy -force $CL_DIR/design/cl_minikanren_chirho.sv $src_post_enc_dir
file copy -force $CL_DIR/design/cl_pcis_handler_chirho.sv $src_post_enc_dir
file copy -force $CL_DIR/design/cl_axi_arbiter_chirho.sv $src_post_enc_dir
file copy -force $CL_DIR/design/searchEngineChirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/searchEngine64BitChirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_512_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_hier_65k_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_hier_262k_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/diffTrainChirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/soft_and_32_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/soft_and_16_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_prob_domain_64_chirho.v $src_post_enc_dir
ENCEOF

# ============================================================================
# V5.7 BUILD: 125MHz (A0 recipe) to fix timing violation
# ============================================================================
echo "Starting V5.7 HDK build (125 MHz, A0 recipe)..."
python3 $HDK_DIR/common/shell_stable/build/scripts/aws_build_dcp_from_cl.py \
    --cl cl_minikanren_chirho \
    --aws_clk_gen \
    --clock_recipe_a A0 \
    2>&1 | tee $WORK_DIR_CHIRHO/build_v57_chirho.log

cd $CL_DIR/build
DCP_TAR=$(find . -name "*.Developer_CL.tar" 2>/dev/null | head -1)
if [ -n "$DCP_TAR" ]; then
    aws s3 cp "$DCP_TAR" s3://$BUCKET_CHIRHO/f2_hbm_hdk/dcp_v57_125mhz/
    echo "v57_125mhz_success" > /tmp/build_status.txt
else
    echo "v57_125mhz_failed" > /tmp/build_status.txt
fi

aws s3 cp $WORK_DIR_CHIRHO/build_v57_chirho.log s3://$BUCKET_CHIRHO/f2_hbm_hdk/build_v57_chirho.log
aws s3 cp /tmp/build_status.txt s3://$BUCKET_CHIRHO/f2_hbm_hdk/build_v57_status_chirho.txt
aws s3 cp /var/log/hdk-build-v57-125mhz-chirho.log s3://$BUCKET_CHIRHO/f2_hbm_hdk/userdata_v57_chirho.log

echo "=== V5.7 125MHz Build Complete ☧ ==="
date

# Keep instance alive for 30 min to check results
sleep 1800
shutdown -h now
