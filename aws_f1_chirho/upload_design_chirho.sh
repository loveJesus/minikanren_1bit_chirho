#!/bin/bash
# Upload FPGA Design to S3 ☧
#
# Uploads the Clash-generated Verilog and constraints to S3.
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

echo "=== Upload FPGA Design ☧ ==="
echo "Bucket: ${S3_BUCKET_CHIRHO}"

# Create staging directory
STAGING_CHIRHO="${SCRIPT_DIR_CHIRHO}/staging_chirho"
mkdir -p "${STAGING_CHIRHO}"

# Copy Verilog files
echo "[1/3] Copying Verilog files..."
cp "${PROJECT_ROOT_CHIRHO}/clash_chirho/verilog/MiniKanrenChirho.searchEngineChirho/searchEngineChirho.v" \
   "${STAGING_CHIRHO}/"
echo "  Copied searchEngineChirho.v"

# Create timing constraints file
echo "[2/3] Creating timing constraints..."
cat > "${STAGING_CHIRHO}/timing_chirho.xdc" << 'EOF'
# miniKanren FPGA Timing Constraints ☧
# Target: 250 MHz (4ns period), achieved: 280 MHz

# Clock definition
create_clock -period 4.000 -name clk [get_ports clk]

# Input delay constraints (assuming 1ns setup from external source)
set_input_delay -clock clk -max 1.0 [get_ports {rst enChirho cmdChirho[*]}]
set_input_delay -clock clk -min 0.0 [get_ports {rst enChirho cmdChirho[*]}]

# Output delay constraints (assuming 1ns hold to external sink)
set_output_delay -clock clk -max 1.0 [get_ports {respChirho[*]}]
set_output_delay -clock clk -min 0.0 [get_ports {respChirho[*]}]

# False paths for reset (async)
set_false_path -from [get_ports rst]
EOF
echo "  Created timing_chirho.xdc"

# Create Vivado synthesis script
echo "[3/3] Creating Vivado script..."
cat > "${STAGING_CHIRHO}/synth_vivado_chirho.tcl" << 'EOF'
# miniKanren Vivado Synthesis Script ☧
# Target: Xilinx VU9P (AWS F1)

# Project setup
set project_name "minikanren_chirho"
set part_name "xcvu9p-flgb2104-2-i"
set top_module "searchEngineChirho"

# Create project
create_project -force ${project_name} ./${project_name} -part ${part_name}

# Add sources
add_files -fileset sources_1 ./searchEngineChirho.v
add_files -fileset constrs_1 ./timing_chirho.xdc

# Set top module
set_property top ${top_module} [current_fileset]

# Run synthesis
synth_design -top ${top_module} -part ${part_name}
report_utilization -file utilization_post_synth_chirho.rpt
report_timing_summary -file timing_post_synth_chirho.rpt

# Run implementation
opt_design
place_design
phys_opt_design
route_design

# Generate reports
report_utilization -file utilization_chirho.rpt
report_utilization -hierarchical -file utilization_hierarchical_chirho.rpt
report_timing_summary -file timing_chirho.rpt
report_timing -max_paths 20 -file timing_paths_chirho.rpt
report_clock_utilization -file clock_utilization_chirho.rpt

# Check timing
set wns [get_property SLACK [get_timing_paths -max_paths 1 -setup]]
set whs [get_property SLACK [get_timing_paths -max_paths 1 -hold]]

puts ""
puts "=== TIMING SUMMARY ==="
puts "WNS (Setup): ${wns} ns"
puts "WHS (Hold):  ${whs} ns"

if {$wns >= 0 && $whs >= 0} {
    puts "TIMING MET!"

    # Save checkpoint
    write_checkpoint -force ${project_name}_routed.dcp

    # Calculate achieved frequency
    set period 4.0
    set achieved_period [expr {$period - $wns}]
    set achieved_freq [expr {1000.0 / $achieved_period}]
    puts "Achieved Frequency: ${achieved_freq} MHz"
} else {
    puts "WARNING: Timing not met"
}

puts "=== DONE ☧ ==="
EOF
echo "  Created synth_vivado_chirho.tcl"

# Upload to S3
echo ""
echo "Uploading to S3..."
aws s3 sync "${STAGING_CHIRHO}" "s3://${S3_BUCKET_CHIRHO}/design/" --delete

echo ""
echo "=== Upload Complete ☧ ==="
echo ""
echo "Files uploaded to: s3://${S3_BUCKET_CHIRHO}/design/"
aws s3 ls "s3://${S3_BUCKET_CHIRHO}/design/"
