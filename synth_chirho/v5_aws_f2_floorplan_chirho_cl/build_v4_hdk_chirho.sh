#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16 ☧
# v4 Build: Hierarchical Domains (256², 512², 256³, 512³) + Neurosymbolic
# Based on working v3 pattern
#
# MEMORY REQUIREMENT: 256GB RAM recommended (r5.8xlarge)
# - c5.9xlarge (72GB) fails with OOM during Timing Optimization
# - r5.4xlarge (128GB) marginal - uses 123GB+ during build
# - Vivado uses ~120GB+ peak for this design with HBM IP
# ============================================================================
set -x
exec > >(tee /var/log/hdk-build-v4-chirho.log) 2>&1
echo "=== v4 Hierarchical + Neurosymbolic Build Starting ☧ ==="
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

# Download v4 design files (hierarchical + neurosymbolic)
aws s3 cp s3://$BUCKET_CHIRHO/f2_hbm_hdk/design_v4_hier_ns_chirho.tar.gz /tmp/design.tar.gz
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

# VALID PCI ID: 0xF004 (AWS valid range 0xF000-0xF0FF, v4 = 04)
cat > "$CL_DIR/design/cl_id_defines.vh" << 'IDEOF'
// ============================================================================
// For God so loved the world - John 3:16 ☧
// v4: Hierarchical Domains + Neurosymbolic
// PCI DeviceID 0xF004 = valid AWS range + version 4
// ============================================================================
`define CL_SH_ID0 32'hF004_1D0F
`define CL_SH_ID1 32'h1D51_F004
IDEOF

# Build script symlinks
cd $CL_DIR/build/scripts
ln -sf $HDK_DIR/common/shell_stable/build/scripts/aws_build_dcp_from_cl.py .
ln -sf $HDK_DIR/common/shell_stable/build/scripts/build_all.tcl .
ln -sf $HDK_DIR/common/shell_stable/build/scripts/build_level_1_cl.tcl .

# Create synthesis TCL
cat > "$CL_DIR/build/scripts/synth_cl_minikanren_chirho.tcl" << 'SYNTHTCL'
source ${HDK_SHELL_DIR}/build/scripts/synth_cl_header.tcl
print "Reading user source codes - v4 Hierarchical + Neurosymbolic ☧"

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

print "Starting synthesizing customer design ${CL} - v4 ☧"
update_compile_order -fileset sources_1
synth_design -mode out_of_context \
             -top ${CL} \
             -verilog_define XSDB_SLV_DIS \
             -part ${DEVICE_TYPE} \
             -keep_equivalent_registers
source ${HDK_SHELL_DIR}/build/scripts/synth_cl_footer.tcl
SYNTHTCL

cat > "$CL_DIR/build/constraints/cl_synth_user.xdc" << 'XDCEOF'
# No special synthesis constraints for v4
XDCEOF

cat > "$CL_DIR/build/constraints/cl_timing_user.xdc" << 'XDCEOF'
# HBM async paths - CDC handled by HBM IP synchronizers
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*WREADY*}]
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*RREADY*}]
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*BREADY*}]
XDCEOF

cat > "$CL_DIR/build/constraints/small_shell_cl_pnr_user.xdc" << 'PNREOF'
# No special P&R constraints for v4
PNREOF

# encrypt.tcl - CRITICAL: list ALL v4 design files
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

# Hierarchical intersection modules
file copy -force $CL_DIR/design/intersect_512_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_hier_65k_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_hier_262k_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_hier_16m_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_hier_134m_chirho.v $src_post_enc_dir

# Neurosymbolic modules
file copy -force $CL_DIR/design/diffTrainChirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/soft_and_32_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/soft_and_16_chirho.v $src_post_enc_dir
file copy -force $CL_DIR/design/intersect_prob_domain_64_chirho.v $src_post_enc_dir
ENCEOF

echo "Starting v4 HDK build (250 MHz, A2 recipe)..."
python3 $HDK_DIR/common/shell_stable/build/scripts/aws_build_dcp_from_cl.py \
    --cl cl_minikanren_chirho \
    --aws_clk_gen \
    --clock_recipe_a A2 \
    2>&1 | tee $WORK_DIR_CHIRHO/build_v4_chirho.log

cd $CL_DIR/build
DCP_TAR=$(find . -name "*.Developer_CL.tar" 2>/dev/null | head -1)
if [ -n "$DCP_TAR" ]; then
    aws s3 cp "$DCP_TAR" s3://$BUCKET_CHIRHO/f2_hbm_hdk/dcp_v4_hier_ns/
    echo "v4_hier_ns_success" > /tmp/build_status.txt
else
    echo "v4_hier_ns_failed" > /tmp/build_status.txt
fi

aws s3 cp $WORK_DIR_CHIRHO/build_v4_chirho.log s3://$BUCKET_CHIRHO/f2_hbm_hdk/build_v4_chirho.log
aws s3 cp /tmp/build_status.txt s3://$BUCKET_CHIRHO/f2_hbm_hdk/build_v4_status_chirho.txt
aws s3 cp /var/log/hdk-build-v4-chirho.log s3://$BUCKET_CHIRHO/f2_hbm_hdk/userdata_v4_chirho.log

echo "=== v4 Hierarchical + Neurosymbolic Build Complete ☧ ==="
date

# Keep instance alive for 30 min to check results, then shutdown
sleep 1800
shutdown -h now
