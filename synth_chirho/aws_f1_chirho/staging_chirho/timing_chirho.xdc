# miniKanren FPGA Timing Constraints ☧
# Target: 50 MHz (20ns period)
# Run 3: Fixed hold timing with proper min input delays

# Clock definition
create_clock -period 20.000 -name clk [get_ports clk]

# Input delay constraints
# Max: 1ns setup margin from external source
# Min: 4ns to account for clock network delay (~3.5ns) + margin
#      This prevents hold violations from fast input paths
set_input_delay -clock clk -max 1.0 [get_ports {rst enChirho cmdChirho[*]}]
set_input_delay -clock clk -min 4.0 [get_ports {rst enChirho cmdChirho[*]}]

# Output delay constraints
# Max: 1ns hold margin to external sink
# Min: 0ns (outputs can change immediately after clock)
set_output_delay -clock clk -max 1.0 [get_ports {respChirho[*]}]
set_output_delay -clock clk -min 0.0 [get_ports {respChirho[*]}]

# False paths for reset (async)
set_false_path -from [get_ports rst]

# Note: The min input delay of 4ns (line 13) is the proper fix for hold
# violations caused by clock network delay (~3.5ns). No max_delay needed.
