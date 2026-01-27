#!/bin/bash
# AWS F1 Synthesis Script ☧
# Synthesizes miniKanren search engine for Xilinx VU9P
#
# Prerequisites:
# - Running on AWS FPGA Developer AMI
# - aws-fpga repo cloned and sourced

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DESIGN_NAME="minikanren_chirho"

echo "=== miniKanren F1 Synthesis ☧ ==="
echo "Design: searchEngineChirho"
echo "Target: Xilinx VU9P (AWS F1)"
echo ""

# Check environment
if [ -z "$HDK_DIR" ]; then
    echo "Error: HDK_DIR not set. Please run:"
    echo "  cd ~/aws-fpga && source hdk_setup.sh"
    exit 1
fi

# Create custom CL directory
CL_DIR="$HDK_DIR/cl/developer_designs/cl_${DESIGN_NAME}"
mkdir -p "$CL_DIR/design"
mkdir -p "$CL_DIR/build/constraints"

echo "Created CL directory: $CL_DIR"

# Copy our Verilog
cp "$SCRIPT_DIR/searchEngineChirho.v" "$CL_DIR/design/"

# Create top-level wrapper for AWS shell interface
cat > "$CL_DIR/design/cl_${DESIGN_NAME}.sv" << 'EOF'
// miniKanren CL Wrapper for AWS F1 ☧
// Wraps searchEngineChirho with AWS Shell interface

module cl_minikanren_chirho
(
   `include "cl_ports.vh"
);

`include "cl_common_defines.vh"
`include "cl_id_defines.vh"

// Tie off unused interfaces
`include "unused_apppf_irq_template.inc"
`include "unused_cl_sda_template.inc"
`include "unused_ddr_a_b_d_template.inc"
`include "unused_ddr_c_template.inc"
`include "unused_pcim_template.inc"
`include "unused_sh_bar1_template.inc"
`include "unused_flr_template.inc"

// Internal signals
logic [69:0]  cmd_chirho;
logic [513:0] resp_chirho;
logic         engine_en_chirho;

// Instantiate our search engine
searchEngineChirho engine_inst_chirho (
    .clk        (clk_main_a0),
    .rst        (rst_main_n_sync),
    .enChirho   (engine_en_chirho),
    .cmdChirho  (cmd_chirho),
    .respChirho (resp_chirho)
);

// AXI-Lite interface for control
// OCL: 32-bit BAR
// Offset 0x00: Command register (write cmd_chirho[31:0])
// Offset 0x04: Command register (write cmd_chirho[63:32])
// Offset 0x08: Command register (write cmd_chirho[69:64] + enable)
// Offset 0x10: Response register (read resp_chirho[31:0])
// Offset 0x14-0x4C: Response registers (read resp_chirho[513:32])

axi_register_slice_light AXIL_OCL_REG_SLC (
   .aclk          (clk_main_a0),
   .aresetn       (rst_main_n_sync),
   .s_axi_awaddr  (sh_ocl_awaddr),
   .s_axi_awprot  (2'h0),
   .s_axi_awvalid (sh_ocl_awvalid),
   .s_axi_awready (ocl_sh_awready),
   .s_axi_wdata   (sh_ocl_wdata),
   .s_axi_wstrb   (sh_ocl_wstrb),
   .s_axi_wvalid  (sh_ocl_wvalid),
   .s_axi_wready  (ocl_sh_wready),
   .s_axi_bresp   (ocl_sh_bresp),
   .s_axi_bvalid  (ocl_sh_bvalid),
   .s_axi_bready  (sh_ocl_bready),
   .s_axi_araddr  (sh_ocl_araddr),
   .s_axi_arprot  (2'h0),
   .s_axi_arvalid (sh_ocl_arvalid),
   .s_axi_arready (ocl_sh_arready),
   .s_axi_rdata   (ocl_sh_rdata),
   .s_axi_rresp   (ocl_sh_rresp),
   .s_axi_rvalid  (ocl_sh_rvalid),
   .s_axi_rready  (sh_ocl_rready),
   .m_axi_awaddr  (ocl_awaddr),
   .m_axi_awprot  (),
   .m_axi_awvalid (ocl_awvalid),
   .m_axi_awready (ocl_awready),
   .m_axi_wdata   (ocl_wdata),
   .m_axi_wstrb   (),
   .m_axi_wvalid  (ocl_wvalid),
   .m_axi_wready  (ocl_wready),
   .m_axi_bresp   (ocl_bresp),
   .m_axi_bvalid  (ocl_bvalid),
   .m_axi_bready  (ocl_bready),
   .m_axi_araddr  (ocl_araddr),
   .m_axi_arprot  (),
   .m_axi_arvalid (ocl_arvalid),
   .m_axi_arready (ocl_arready),
   .m_axi_rdata   (ocl_rdata),
   .m_axi_rresp   (ocl_rresp),
   .m_axi_rvalid  (ocl_rvalid),
   .m_axi_rready  (ocl_rready)
);

logic [31:0] ocl_awaddr, ocl_araddr, ocl_wdata, ocl_rdata;
logic ocl_awvalid, ocl_awready, ocl_wvalid, ocl_wready;
logic ocl_bvalid, ocl_bready, ocl_arvalid, ocl_arready;
logic ocl_rvalid, ocl_rready;
logic [1:0] ocl_bresp, ocl_rresp;

// Simple register interface
always_ff @(posedge clk_main_a0)
begin
    if (!rst_main_n_sync) begin
        cmd_chirho <= 70'b0;
        engine_en_chirho <= 1'b0;
        ocl_bvalid <= 1'b0;
        ocl_rvalid <= 1'b0;
    end
    else begin
        // Write handling
        if (ocl_awvalid && ocl_wvalid && ocl_awready) begin
            case (ocl_awaddr[7:0])
                8'h00: cmd_chirho[31:0]  <= ocl_wdata;
                8'h04: cmd_chirho[63:32] <= ocl_wdata;
                8'h08: begin
                    cmd_chirho[69:64] <= ocl_wdata[5:0];
                    engine_en_chirho <= ocl_wdata[31];
                end
            endcase
            ocl_bvalid <= 1'b1;
        end
        else if (ocl_bready)
            ocl_bvalid <= 1'b0;

        // Read handling
        if (ocl_arvalid && ocl_arready) begin
            case (ocl_araddr[7:0])
                8'h10: ocl_rdata <= resp_chirho[31:0];
                8'h14: ocl_rdata <= resp_chirho[63:32];
                8'h18: ocl_rdata <= resp_chirho[95:64];
                8'h1C: ocl_rdata <= resp_chirho[127:96];
                8'h20: ocl_rdata <= resp_chirho[159:128];
                8'h24: ocl_rdata <= resp_chirho[191:160];
                8'h28: ocl_rdata <= resp_chirho[223:192];
                8'h2C: ocl_rdata <= resp_chirho[255:224];
                8'h30: ocl_rdata <= resp_chirho[287:256];
                8'h34: ocl_rdata <= resp_chirho[319:288];
                8'h38: ocl_rdata <= resp_chirho[351:320];
                8'h3C: ocl_rdata <= resp_chirho[383:352];
                8'h40: ocl_rdata <= resp_chirho[415:384];
                8'h44: ocl_rdata <= resp_chirho[447:416];
                8'h48: ocl_rdata <= resp_chirho[479:448];
                8'h4C: ocl_rdata <= resp_chirho[513:480];
                default: ocl_rdata <= 32'hDEADBEEF;
            endcase
            ocl_rvalid <= 1'b1;
        end
        else if (ocl_rready)
            ocl_rvalid <= 1'b0;
    end
end

assign ocl_awready = !ocl_bvalid;
assign ocl_wready = !ocl_bvalid;
assign ocl_arready = !ocl_rvalid;
assign ocl_bresp = 2'b00;
assign ocl_rresp = 2'b00;

endmodule
EOF

echo "Created CL wrapper"

# Create build scripts
cat > "$CL_DIR/build/scripts/encrypt.tcl" << 'EOF'
# Encryption disabled for development
EOF

# Run synthesis
echo ""
echo "Starting Vivado synthesis..."
echo "This takes 2-4 hours for full VU9P implementation."
echo ""

cd "$CL_DIR/build/scripts"
./aws_build_dcp_from_cl.sh -clock_recipe_a A1 -foreground

echo ""
echo "=== Synthesis Complete ==="
echo "DCP file: $CL_DIR/build/checkpoints/to_aws/*.Developer_CL.dcp"
echo ""
echo "Next step: Create AFI with:"
echo "  aws ec2 create-fpga-image --name minikanren_chirho \\"
echo "    --input-storage-location Bucket=<bucket>,Key=<dcp-path>"
