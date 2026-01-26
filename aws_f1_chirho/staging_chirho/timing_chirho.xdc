# miniKanren FPGA Timing Constraints ☧
# Target: 50 MHz (20ns period) - relaxed for timing closure

# Clock definition
create_clock -period 20.000 -name clk [get_ports clk]

# Input delay constraints (assuming 1ns setup from external source)
set_input_delay -clock clk -max 1.0 [get_ports {rst enChirho cmdChirho[*]}]
set_input_delay -clock clk -min 0.0 [get_ports {rst enChirho cmdChirho[*]}]

# Output delay constraints (assuming 1ns hold to external sink)
set_output_delay -clock clk -max 1.0 [get_ports {respChirho[*]}]
set_output_delay -clock clk -min 0.0 [get_ports {respChirho[*]}]

# False paths for reset (async)
set_false_path -from [get_ports rst]
