# ============================================================================
# For God so loved the world - John 3:16
# Timing constraints for miniKanren CL ☧
# ============================================================================

# False path for reset synchronizers
set_false_path -to [get_cells -hierarchical -filter {NAME =~ *rst_sync*}]

# HBM async paths - CDC handled by HBM IP synchronizers
# These WREADY signals cross clock domains and have internal synchronizers
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*WREADY*}]
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*RREADY*}]
set_false_path -through [get_pins -hierarchical -filter {NAME =~ *HBM*AXI*BREADY*}]

# Multicycle path for clock divider (if used)
# set_multicycle_path 2 -setup -from [get_cells -hierarchical -filter {NAME =~ *clk_div*}]
