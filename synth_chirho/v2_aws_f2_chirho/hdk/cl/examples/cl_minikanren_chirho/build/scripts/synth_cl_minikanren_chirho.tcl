# ============================================================================
# For God so loved the world - John 3:16
# miniKanren F2 Synthesis Script with HBM Support
# ============================================================================

set CL_MODULE cl_minikanren_chirho
set HDK_DIR $::env(HDK_DIR)
set CL_DIR $HDK_DIR/cl/examples/cl_minikanren_chirho

# Set include directories
set_property include_dirs [list \
    $CL_DIR/design \
    $HDK_DIR/common/shell_stable/design/interfaces \
    $HDK_DIR/common/lib \
] [current_fileset]

# Clock constraints
create_clock -period 4.0 -name clk_main_a0 [get_ports clk_main_a0]
create_clock -period 10.0 -name clk_hbm_ref [get_ports clk_hbm_ref]

# Set design hierarchy
set_property top $CL_MODULE [current_fileset]

# Read common library files
read_verilog -sv [glob $HDK_DIR/common/lib/*.sv]

# Read design files (includes HBM wrappers)
read_verilog -sv [glob $CL_DIR/design/*.sv]
read_verilog [glob $CL_DIR/design/*.v]

# Read interface definitions
read_verilog -sv $HDK_DIR/common/shell_stable/design/interfaces/cl_ports.vh

# Parameters for HBM-enabled build
set_property generic {EN_DDR=0 EN_HBM=1} [current_fileset]

# Synthesis
synth_design -top $CL_MODULE -part xcvu47p-fsvh2892-2-e -mode out_of_context

# Optimize
opt_design
place_design
phys_opt_design
route_design

# Reports
report_timing_summary -file timing_summary.rpt
report_utilization -file utilization.rpt
report_power -file power.rpt

# Write checkpoint
write_checkpoint -force post_route.dcp

# Write bitstream (for standalone testing)
# write_bitstream -force $CL_MODULE.bit

puts "============================================"
puts "miniKanren HBM Synthesis Complete"
puts "============================================"
