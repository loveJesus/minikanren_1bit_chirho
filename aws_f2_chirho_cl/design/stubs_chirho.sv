// ============================================================================
// For God so loved the world - John 3:16
// Stub modules for local Verilator simulation ☧
// These are placeholders for Xilinx/AWS IP that's only available in Vivado
// ============================================================================

// Xilinx CDC async reset
module xpm_cdc_async_rst #(
    parameter DEST_SYNC_FF = 2,
    parameter INIT_SYNC_FF = 0,
    parameter RST_ACTIVE_HIGH = 0
) (
    input  wire src_arst,
    input  wire dest_clk,
    output wire dest_arst
);
    assign dest_arst = src_arst;
endmodule

// Sync module stub
module sync #(
    parameter WIDTH = 1
) (
    input  wire clk,
    input  wire [WIDTH-1:0] in,
    output reg  [WIDTH-1:0] out
);
    always @(posedge clk) out <= in;
endmodule

// HBM MMCM stub
module cl_hbm_mmcm (
    input  wire clk_in1,
    input  wire resetn,
    output wire clk_out1,
    output wire clk_out2,
    output wire locked
);
    assign clk_out1 = clk_in1;
    assign clk_out2 = clk_in1;
    assign locked = ~resetn;
endmodule

// HBM IP stub (simplified - actual HBM has 32 AXI ports)
module cl_hbm #(
    parameter HBM_REF_CLK_FREQ = 100
) (
    input  wire HBM_REF_CLK_0,
    input  wire AXI_00_ACLK,
    input  wire AXI_00_ARESET_N,
    // AXI port 0 (simplified)
    input  wire [32:0] AXI_00_ARADDR,
    input  wire [1:0]  AXI_00_ARBURST,
    input  wire [5:0]  AXI_00_ARID,
    input  wire [3:0]  AXI_00_ARLEN,
    input  wire [2:0]  AXI_00_ARSIZE,
    input  wire        AXI_00_ARVALID,
    output wire        AXI_00_ARREADY,
    output wire [255:0] AXI_00_RDATA,
    output wire [5:0]  AXI_00_RID,
    output wire        AXI_00_RLAST,
    output wire [1:0]  AXI_00_RRESP,
    output wire        AXI_00_RVALID,
    input  wire        AXI_00_RREADY,
    // Write channel
    input  wire [32:0] AXI_00_AWADDR,
    input  wire [1:0]  AXI_00_AWBURST,
    input  wire [5:0]  AXI_00_AWID,
    input  wire [3:0]  AXI_00_AWLEN,
    input  wire [2:0]  AXI_00_AWSIZE,
    input  wire        AXI_00_AWVALID,
    output wire        AXI_00_AWREADY,
    input  wire [255:0] AXI_00_WDATA,
    input  wire        AXI_00_WLAST,
    input  wire [31:0] AXI_00_WSTRB,
    input  wire        AXI_00_WVALID,
    output wire        AXI_00_WREADY,
    output wire [5:0]  AXI_00_BID,
    output wire [1:0]  AXI_00_BRESP,
    output wire        AXI_00_BVALID,
    input  wire        AXI_00_BREADY,
    // APB interface
    input  wire        APB_0_PCLK,
    input  wire        APB_0_PRESET_N,
    output wire        apb_complete_0
);
    // Stub - always ready, return zeros
    assign AXI_00_ARREADY = 1'b1;
    assign AXI_00_AWREADY = 1'b1;
    assign AXI_00_WREADY = 1'b1;
    assign AXI_00_RDATA = 256'b0;
    assign AXI_00_RID = 6'b0;
    assign AXI_00_RLAST = 1'b1;
    assign AXI_00_RRESP = 2'b0;
    assign AXI_00_RVALID = 1'b0;
    assign AXI_00_BID = 6'b0;
    assign AXI_00_BRESP = 2'b0;
    assign AXI_00_BVALID = 1'b0;
    assign apb_complete_0 = 1'b1;
endmodule

// Shell DDR stub
module sh_ddr #(
    parameter DDR_A_PRESENT = 0,
    parameter DDR_B_PRESENT = 0,
    parameter DDR_D_PRESENT = 0
) (
    input  wire clk,
    input  wire rst_n,
    // All DDR ports tied off in stub
    input  wire CLK_300M_DIMM0_DP,
    input  wire CLK_300M_DIMM0_DN,
    output wire cl_RST_DIMM_A_N,
    output wire cl_RST_DIMM_B_N,
    output wire cl_RST_DIMM_D_N
);
    assign cl_RST_DIMM_A_N = 1'b1;
    assign cl_RST_DIMM_B_N = 1'b1;
    assign cl_RST_DIMM_D_N = 1'b1;
endmodule

// AXI register slice stub
module axi_register_slice #(
    parameter DATA_WIDTH = 512,
    parameter ADDR_WIDTH = 64,
    parameter ID_WIDTH = 16
) (
    input  wire aclk,
    input  wire aresetn,
    // Simplified - pass through
    axi_bus_t.slave s_axi,
    axi_bus_t.master m_axi
);
    // Pass-through stub
endmodule

// AXI smartconnect wrapper stub
module cl_axi_sc_1x1_wrapper #(
    parameter AXI_ID_WIDTH = 16
) (
    input  wire aclk,
    input  wire aresetn,
    axi_bus_t.slave s_axi,
    axi_bus_t.master m_axi
);
endmodule

// AXI3 256b register slice stub
module cl_axi3_256b_reg_slice (
    input  wire aclk,
    input  wire aresetn,
    axi_bus_t.slave s_axi,
    axi_bus_t.master m_axi
);
endmodule
