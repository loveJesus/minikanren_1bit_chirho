# Vivado Standalone Synthesis for miniKanren ☧
# Gets timing/utilization without full F1 flow
#
# Usage: vivado -mode batch -source synth_standalone_chirho.tcl

set design_name "searchEngineChirho"
set part "xcvu9p-flgb2104-2-i"  ;# AWS F1 VU9P

# Create project
create_project -force $design_name ./vivado_out -part $part

# Add source files
add_files searchEngineChirho.v

# Set top module
set_property top searchEngineChirho [current_fileset]

# Create timing constraints
create_clock -period 4.0 -name clk [get_ports clk]  ;# 250 MHz target

# Run synthesis
synth_design -top $design_name -part $part

# Report utilization
report_utilization -file utilization_chirho.rpt

# Run implementation (place & route)
opt_design
place_design
route_design

# Report timing
report_timing_summary -file timing_chirho.rpt
report_timing -nworst 10 -file timing_paths_chirho.rpt

# Get Fmax
set wns [get_property SLACK [get_timing_paths -max_paths 1 -nworst 1 -setup]]
set fmax [expr {1000.0 / (4.0 - $wns)}]
puts "============================================"
puts "TIMING RESULTS ☧"
puts "============================================"
puts "WNS (Worst Negative Slack): $wns ns"
puts "Estimated Fmax: $fmax MHz"
puts "============================================"

# Write checkpoint
write_checkpoint -force ${design_name}_routed.dcp

puts "Synthesis complete! Check utilization_chirho.rpt and timing_chirho.rpt"
