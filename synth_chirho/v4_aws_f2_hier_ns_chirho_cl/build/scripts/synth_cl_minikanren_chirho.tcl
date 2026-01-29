# ============================================================================
# For God so loved the world - John 3:16
# miniKanren F2 Synthesis Script - Hierarchical Domains + Neurosymbolic ☧
# Includes: 256², 512², 256³, 512³ modes + Training + Probabilistic Inference
# ============================================================================

set CL_MODULE cl_minikanren_chirho
set CL_DIR [pwd]/../..

puts "============================================"
puts "miniKanren HBM Synthesis Starting"
puts "  Hierarchical: 256², 512², 256³, 512³"
puts "  Neurosymbolic: Training + Inference"
puts "CL_DIR: $CL_DIR"
puts "============================================"

# Create in-memory project
create_project -in_memory -part xcvu47p-fsvh2892-2-e

# Set include directories for defines BEFORE reading files
set_property include_dirs [list $CL_DIR/design] [current_fileset]

# Read all design files (stubs and interfaces first)
puts "Reading design files..."
read_verilog -sv $CL_DIR/design/stubs_chirho.sv
read_verilog -sv $CL_DIR/design/cl_minikanren_chirho_defines.vh
read_verilog -sv $CL_DIR/design/cl_dram_dma_defines.vh
read_verilog -sv $CL_DIR/design/cl_id_defines.vh
read_verilog -sv $CL_DIR/design/cl_ports.vh
read_verilog -sv $CL_DIR/design/cl_hbm_axi4.sv
# cl_hbm_wrapper.sv excluded - using stub version for out-of-context synthesis
# read_verilog -sv $CL_DIR/design/cl_hbm_wrapper.sv
read_verilog -sv $CL_DIR/design/cl_minikanren_chirho.sv

# Base search engines (Clash-generated)
puts "Reading base search engines..."
read_verilog $CL_DIR/design/searchEngineChirho.v
read_verilog $CL_DIR/design/searchEngine64BitChirho.v

# Hierarchical domain intersection modules (256², 512², 256³, 512³)
puts "Reading hierarchical intersection modules..."
read_verilog $CL_DIR/design/intersect_512_chirho.v
read_verilog $CL_DIR/design/intersect_hier_65k_chirho.v
read_verilog $CL_DIR/design/intersect_hier_262k_chirho.v
read_verilog $CL_DIR/design/intersect_hier_16m_chirho.v
read_verilog $CL_DIR/design/intersect_hier_134m_chirho.v

# Neurosymbolic modules (training + inference)
puts "Reading neurosymbolic modules..."
read_verilog $CL_DIR/design/diffTrainChirho.v
read_verilog $CL_DIR/design/soft_and_32_chirho.v
read_verilog $CL_DIR/design/soft_and_16_chirho.v
read_verilog $CL_DIR/design/intersect_prob_domain_64_chirho.v

# Set top module
set_property top $CL_MODULE [current_fileset]

puts "Starting synthesis (this may take 2-4 hours)..."
synth_design -top $CL_MODULE -part xcvu47p-fsvh2892-2-e -mode out_of_context -flatten_hierarchy rebuilt -generic EN_DDR=0 -generic EN_HBM=1

# Post-synthesis reports
puts "Generating post-synthesis reports..."
report_timing_summary -file timing_post_synth_chirho.rpt
report_utilization -file utilization_post_synth_chirho.rpt

# Write synthesis checkpoint
write_checkpoint -force post_synth_chirho.dcp

puts "Optimizing design..."
opt_design

puts "Placing design..."
place_design

puts "Physical optimization..."
phys_opt_design

puts "Routing design..."
route_design

# Final reports
puts "Generating final reports..."
report_timing_summary -file timing_summary_chirho.rpt
report_utilization -file utilization_chirho.rpt
report_power -file power_chirho.rpt

# Write final checkpoint
write_checkpoint -force post_route_chirho.dcp

puts "============================================"
puts "miniKanren HBM Synthesis Complete"
puts "  Hierarchical modes: 65K, 262K, 16.7M, 134M"
puts "  Neurosymbolic: Training + Prob Inference"
puts "============================================"
puts ""
puts "Soli Deo Gloria ☧"
