// ============================================================================
// For God so loved the world - John 3:16
// Stub modules for standalone synthesis verification ☧
// These are placeholders for Xilinx/AWS IP that's only available during
// full AWS HDK builds.
// ============================================================================
//
// NAMING CONVENTION:
// ------------------
// - File names use '_chirho' suffix: stubs_chirho.sv, cl_minikanren_chirho.sv
// - Custom modules we create use '_chirho' suffix: cl_minikanren_chirho
// - AWS HDK module STUBS preserve original AWS names for compatibility:
//   cl_hbm_wrapper, sh_ddr, cl_axi4_to_axi3_conv, axi_register_slice, etc.
//   This ensures our code can integrate with real AWS HDK without renaming.
// - Xilinx primitive stubs preserve original names: xpm_cdc_async_rst
// - SearchEngineChirho uses PascalCase per Verilog module convention
//
// ============================================================================

// AXI Bus Interface (for internal connections)
interface axi_bus_t #(
    parameter DATA_WIDTH = 256,
    parameter ADDR_WIDTH = 64,
    parameter ID_WIDTH = 6,
    parameter LEN_WIDTH = 8
);
    // Write address
    logic [ID_WIDTH-1:0]   awid;
    logic [ADDR_WIDTH-1:0] awaddr;
    logic [LEN_WIDTH-1:0]  awlen;
    logic [2:0]   awsize;
    logic [1:0]   awburst;
    logic         awlock;
    logic [3:0]   awcache;
    logic [2:0]   awprot;
    logic [3:0]   awqos;
    logic [3:0]   awregion;
    logic         awvalid;
    logic         awready;
    // Write data
    logic [ID_WIDTH-1:0]     wid;    // AXI3 write ID (removed in AXI4 but needed for converter)
    logic [DATA_WIDTH-1:0]   wdata;
    logic [DATA_WIDTH/8-1:0] wstrb;
    logic         wlast;
    logic         wvalid;
    logic         wready;
    // Write response
    logic [ID_WIDTH-1:0]   bid;
    logic [1:0]   bresp;
    logic         bvalid;
    logic         bready;
    // Read address
    logic [ID_WIDTH-1:0]   arid;
    logic [ADDR_WIDTH-1:0] araddr;
    logic [LEN_WIDTH-1:0]  arlen;
    logic [2:0]   arsize;
    logic [1:0]   arburst;
    logic         arlock;
    logic [3:0]   arcache;
    logic [2:0]   arprot;
    logic [3:0]   arqos;
    logic [3:0]   arregion;
    logic         arvalid;
    logic         arready;
    // Read data
    logic [ID_WIDTH-1:0]   rid;
    logic [DATA_WIDTH-1:0] rdata;
    logic [1:0]   rresp;
    logic         rlast;
    logic         rvalid;
    logic         rready;

    modport master (
        output awid, awaddr, awlen, awsize, awburst, awlock, awcache, awprot, awqos, awregion, awvalid,
        input  awready,
        output wid, wdata, wstrb, wlast, wvalid,
        input  wready,
        input  bid, bresp, bvalid,
        output bready,
        output arid, araddr, arlen, arsize, arburst, arlock, arcache, arprot, arqos, arregion, arvalid,
        input  arready,
        input  rid, rdata, rresp, rlast, rvalid,
        output rready
    );

    modport slave (
        input  awid, awaddr, awlen, awsize, awburst, awlock, awcache, awprot, awqos, awregion, awvalid,
        output awready,
        input  wid, wdata, wstrb, wlast, wvalid,
        output wready,
        output bid, bresp, bvalid,
        input  bready,
        input  arid, araddr, arlen, arsize, arburst, arlock, arcache, arprot, arqos, arregion, arvalid,
        output arready,
        output rid, rdata, rresp, rlast, rvalid,
        input  rready
    );
endinterface

// Config Bus Interface (for stat/config ports)
interface cfg_bus_t;
    logic [7:0]   addr;
    logic [31:0]  wdata;
    logic         wr;
    logic         rd;
    logic         ack;
    logic [31:0]  rdata;

    modport master (output addr, wdata, wr, rd, input ack, rdata);
    modport slave (input addr, wdata, wr, rd, output ack, rdata);
endinterface

// ============================================================================
// Xilinx Primitives Stubs
// ============================================================================

// Xilinx CDC async reset
module xpm_cdc_async_rst #(
    parameter DEST_SYNC_FF = 4,
    parameter INIT_SYNC_FF = 0,
    parameter RST_ACTIVE_HIGH = 0
) (
    input  wire src_arst,
    input  wire dest_clk,
    output wire dest_arst
);
    // Stub: pass-through reset
    assign dest_arst = src_arst;
endmodule

// Sync module stub (matching cl_hbm_wrapper port names)
module sync #(
    parameter WIDTH = 1
) (
    input  wire             clk,
    input  wire             rst_n,
    input  wire [WIDTH-1:0] in,
    output reg  [WIDTH-1:0] sync_out
);
    always @(posedge clk) sync_out <= in;
endmodule

// ============================================================================
// AWS HDK Utility Stubs
// ============================================================================

// lib_pipe - Pipeline Register Stub
module lib_pipe #(
    parameter WIDTH  = 1,
    parameter STAGES = 1
) (
    input  wire             clk,
    input  wire             rst_n,
    input  wire [WIDTH-1:0] in_bus,
    output wire [WIDTH-1:0] out_bus
);
    // Stub: pass-through (no actual pipelining for synthesis verification)
    assign out_bus = in_bus;
endmodule

// ============================================================================
// HBM IP Stubs
// ============================================================================

// HBM MMCM stub - generates 450MHz from 100MHz
module cl_hbm_mmcm (
    input  wire clk_in1,
    output wire clk_out1,
    output wire locked
);
    // Stub: pass-through clock
    assign clk_out1 = clk_in1;
    assign locked = 1'b1;
endmodule

// ============================================================================
// cl_hbm_wrapper - Complete Stub for Standalone Synthesis
// This stub replaces the real HBM wrapper for out-of-context synthesis.
// It provides the same interface but with dummy responses.
// ============================================================================
module cl_hbm_wrapper #(
    parameter NUM_OF_AXI_PORTS = 1,
    parameter AXI4_INTERFACE   = 0,
    parameter AXLEN_WIDTH      = AXI4_INTERFACE ? 8 : 4
) (
    input logic                          apb_clk,
    input logic                          i_clk_250m,
    input logic                          i_rst_250m_n,

    output logic                         o_clk_450m,
    output logic                         o_rst_450m_n,
    output logic                         o_clk_100m,
    output logic                         o_rst_100m_n,

    // AXI3/4 bus to HBM (arrays)
    input  logic  [33:0]                 i_axi_araddr   [0:NUM_OF_AXI_PORTS-1],
    input  logic  [1:0]                  i_axi_arburst  [0:NUM_OF_AXI_PORTS-1],
    input  logic  [5:0]                  i_axi_arid     [0:NUM_OF_AXI_PORTS-1],
    input  logic  [AXLEN_WIDTH-1:0]      i_axi_arlen    [0:NUM_OF_AXI_PORTS-1],
    input  logic  [2:0]                  i_axi_arsize   [0:NUM_OF_AXI_PORTS-1],
    input  logic                         i_axi_arvalid  [0:NUM_OF_AXI_PORTS-1],
    input  logic  [33:0]                 i_axi_awaddr   [0:NUM_OF_AXI_PORTS-1],
    input  logic  [1:0]                  i_axi_awburst  [0:NUM_OF_AXI_PORTS-1],
    input  logic  [5:0]                  i_axi_awid     [0:NUM_OF_AXI_PORTS-1],
    input  logic  [AXLEN_WIDTH-1:0]      i_axi_awlen    [0:NUM_OF_AXI_PORTS-1],
    input  logic  [2:0]                  i_axi_awsize   [0:NUM_OF_AXI_PORTS-1],
    input  logic                         i_axi_awvalid  [0:NUM_OF_AXI_PORTS-1],
    input  logic                         i_axi_rready   [0:NUM_OF_AXI_PORTS-1],
    input  logic                         i_axi_bready   [0:NUM_OF_AXI_PORTS-1],
    input  logic  [255:0]                i_axi_wdata    [0:NUM_OF_AXI_PORTS-1],
    input  logic                         i_axi_wlast    [0:NUM_OF_AXI_PORTS-1],
    input  logic  [31:0]                 i_axi_wstrb    [0:NUM_OF_AXI_PORTS-1],
    input  logic                         i_axi_wvalid   [0:NUM_OF_AXI_PORTS-1],
    output logic                         o_axi_arready  [0:NUM_OF_AXI_PORTS-1],
    output logic                         o_axi_awready  [0:NUM_OF_AXI_PORTS-1],
    output logic  [255:0]                o_axi_rdata    [0:NUM_OF_AXI_PORTS-1],
    output logic  [5:0]                  o_axi_rid      [0:NUM_OF_AXI_PORTS-1],
    output logic                         o_axi_rlast    [0:NUM_OF_AXI_PORTS-1],
    output logic  [1:0]                  o_axi_rresp    [0:NUM_OF_AXI_PORTS-1],
    output logic                         o_axi_rvalid   [0:NUM_OF_AXI_PORTS-1],
    output logic                         o_axi_wready   [0:NUM_OF_AXI_PORTS-1],
    output logic  [5:0]                  o_axi_bid      [0:NUM_OF_AXI_PORTS-1],
    output logic  [1:0]                  o_axi_bresp    [0:NUM_OF_AXI_PORTS-1],
    output logic                         o_axi_bvalid   [0:NUM_OF_AXI_PORTS-1],

    // APB interfaces to shell (stack 0)
    input  logic                         i_hbm_apb_preset_n_0,
    output logic [21:0]                  o_hbm_apb_paddr_0,
    output logic [2:0]                   o_hbm_apb_pprot_0,
    output logic                         o_hbm_apb_psel_0,
    output logic                         o_hbm_apb_penable_0,
    output logic                         o_hbm_apb_pwrite_0,
    output logic [31:0]                  o_hbm_apb_pwdata_0,
    output logic [3:0]                   o_hbm_apb_pstrb_0,
    output logic                         o_hbm_apb_pready_0,
    output logic [31:0]                  o_hbm_apb_prdata_0,
    output logic                         o_hbm_apb_pslverr_0,

    // APB interfaces to shell (stack 1)
    input  logic                         i_hbm_apb_preset_n_1,
    output logic [21:0]                  o_hbm_apb_paddr_1,
    output logic [2:0]                   o_hbm_apb_pprot_1,
    output logic                         o_hbm_apb_psel_1,
    output logic                         o_hbm_apb_penable_1,
    output logic                         o_hbm_apb_pwrite_1,
    output logic [31:0]                  o_hbm_apb_pwdata_1,
    output logic [3:0]                   o_hbm_apb_pstrb_1,
    output logic                         o_hbm_apb_pready_1,
    output logic [31:0]                  o_hbm_apb_prdata_1,
    output logic                         o_hbm_apb_pslverr_1,

    // Stats bus
    cfg_bus_t.slave                      hbm_stat_bus,
    output logic [7:0]                   o_cl_sh_hbm_stat_int,
    output logic                         o_hbm_ready
);

    // Clock outputs - pass through for stub
    assign o_clk_450m   = i_clk_250m;
    assign o_rst_450m_n = i_rst_250m_n;
    assign o_clk_100m   = apb_clk;
    assign o_rst_100m_n = i_rst_250m_n;

    // HBM always ready in stub
    assign o_hbm_ready = 1'b1;
    assign o_cl_sh_hbm_stat_int = 8'b0;

    // Stats bus - ack immediately
    assign hbm_stat_bus.ack = hbm_stat_bus.wr | hbm_stat_bus.rd;
    assign hbm_stat_bus.rdata = 32'h0000_0006;  // Bits [2:1] = 2'b11 (both stacks ready)

    // APB - tie off
    assign o_hbm_apb_paddr_0   = 22'b0;
    assign o_hbm_apb_pprot_0   = 3'b0;
    assign o_hbm_apb_psel_0    = 1'b0;
    assign o_hbm_apb_penable_0 = 1'b0;
    assign o_hbm_apb_pwrite_0  = 1'b0;
    assign o_hbm_apb_pwdata_0  = 32'b0;
    assign o_hbm_apb_pstrb_0   = 4'b0;
    assign o_hbm_apb_pready_0  = 1'b1;
    assign o_hbm_apb_prdata_0  = 32'b0;
    assign o_hbm_apb_pslverr_0 = 1'b0;

    assign o_hbm_apb_paddr_1   = 22'b0;
    assign o_hbm_apb_pprot_1   = 3'b0;
    assign o_hbm_apb_psel_1    = 1'b0;
    assign o_hbm_apb_penable_1 = 1'b0;
    assign o_hbm_apb_pwrite_1  = 1'b0;
    assign o_hbm_apb_pwdata_1  = 32'b0;
    assign o_hbm_apb_pstrb_1   = 4'b0;
    assign o_hbm_apb_pready_1  = 1'b1;
    assign o_hbm_apb_prdata_1  = 32'b0;
    assign o_hbm_apb_pslverr_1 = 1'b0;

    // AXI ports - simple stub responses
    genvar i;
    generate
        for (i = 0; i < NUM_OF_AXI_PORTS; i++) begin : AXI_STUB
            // Always ready to accept commands
            assign o_axi_awready[i] = 1'b1;
            assign o_axi_wready[i]  = 1'b1;
            assign o_axi_arready[i] = 1'b1;

            // No read data or write responses in stub (they would need state machines)
            assign o_axi_rdata[i]  = 256'b0;
            assign o_axi_rid[i]    = 6'b0;
            assign o_axi_rlast[i]  = 1'b0;
            assign o_axi_rresp[i]  = 2'b0;
            assign o_axi_rvalid[i] = 1'b0;
            assign o_axi_bid[i]    = 6'b0;
            assign o_axi_bresp[i]  = 2'b0;
            assign o_axi_bvalid[i] = 1'b0;
        end
    endgenerate

endmodule

// ============================================================================
// DDR Stubs
// ============================================================================

// Shell DDR stub (comprehensive port list for synthesis)
module sh_ddr #(
    parameter DDR_PRESENT = 0,
    parameter DDR_A_PRESENT = 0,
    parameter DDR_B_PRESENT = 0,
    parameter DDR_D_PRESENT = 0
) (
    input  wire clk,
    input  wire rst_n,
    input  wire stat_clk,
    input  wire stat_rst_n,
    // DDR4 DIMM interface
    output wire CLK_DIMM_DP,
    output wire CLK_DIMM_DN,
    output wire M_ACT_N,
    output wire [16:0] M_MA,
    output wire [1:0]  M_BA,
    output wire [1:0]  M_BG,
    output wire [1:0]  M_CKE,
    output wire [1:0]  M_ODT,
    output wire [1:0]  M_CS_N,
    output wire M_CLK_DN,
    output wire M_CLK_DP,
    output wire M_PAR,
    inout  wire [71:0] M_DQ,
    inout  wire [8:0]  M_ECC,
    inout  wire [17:0] M_DQS_DP,
    inout  wire [17:0] M_DQS_DN,
    output wire cl_RST_DIMM_N,
    // AXI interface
    input  wire [15:0] cl_sh_ddr_axi_awid,
    input  wire [63:0] cl_sh_ddr_axi_awaddr,
    input  wire [7:0]  cl_sh_ddr_axi_awlen,
    input  wire [2:0]  cl_sh_ddr_axi_awsize,
    input  wire        cl_sh_ddr_axi_awvalid,
    input  wire [1:0]  cl_sh_ddr_axi_awburst,
    input  wire [10:0] cl_sh_ddr_axi_awuser,
    output wire        cl_sh_ddr_axi_awready,
    input  wire [511:0] cl_sh_ddr_axi_wdata,
    input  wire [63:0] cl_sh_ddr_axi_wstrb,
    input  wire        cl_sh_ddr_axi_wlast,
    input  wire        cl_sh_ddr_axi_wvalid,
    output wire        cl_sh_ddr_axi_wready,
    output wire [15:0] cl_sh_ddr_axi_bid,
    output wire [1:0]  cl_sh_ddr_axi_bresp,
    output wire        cl_sh_ddr_axi_bvalid,
    input  wire        cl_sh_ddr_axi_bready,
    input  wire [15:0] cl_sh_ddr_axi_arid,
    input  wire [63:0] cl_sh_ddr_axi_araddr,
    input  wire [7:0]  cl_sh_ddr_axi_arlen,
    input  wire [2:0]  cl_sh_ddr_axi_arsize,
    input  wire        cl_sh_ddr_axi_arvalid,
    input  wire [1:0]  cl_sh_ddr_axi_arburst,
    input  wire [10:0] cl_sh_ddr_axi_aruser,
    output wire        cl_sh_ddr_axi_arready,
    output wire [15:0] cl_sh_ddr_axi_rid,
    output wire [511:0] cl_sh_ddr_axi_rdata,
    output wire [1:0]  cl_sh_ddr_axi_rresp,
    output wire        cl_sh_ddr_axi_rlast,
    output wire        cl_sh_ddr_axi_rvalid,
    input  wire        cl_sh_ddr_axi_rready,
    // Stat bus
    input  wire [7:0]  sh_ddr_stat_bus_addr,
    input  wire [31:0] sh_ddr_stat_bus_wdata,
    input  wire        sh_ddr_stat_bus_wr,
    input  wire        sh_ddr_stat_bus_rd,
    output wire        sh_ddr_stat_bus_ack,
    output wire [31:0] sh_ddr_stat_bus_rdata,
    output wire [7:0]  ddr_sh_stat_int,
    output wire        sh_cl_ddr_is_ready
);
    assign CLK_DIMM_DP = 1'b0;
    assign CLK_DIMM_DN = 1'b0;
    assign M_ACT_N = 1'b1;
    assign M_MA = 17'b0;
    assign M_BA = 2'b0;
    assign M_BG = 2'b0;
    assign M_CKE = 2'b0;
    assign M_ODT = 2'b0;
    assign M_CS_N = 2'b11;
    assign M_CLK_DN = 1'b0;
    assign M_CLK_DP = 1'b0;
    assign M_PAR = 1'b0;
    assign cl_RST_DIMM_N = 1'b1;
    assign cl_sh_ddr_axi_awready = 1'b0;
    assign cl_sh_ddr_axi_wready = 1'b0;
    assign cl_sh_ddr_axi_bid = 16'b0;
    assign cl_sh_ddr_axi_bresp = 2'b0;
    assign cl_sh_ddr_axi_bvalid = 1'b0;
    assign cl_sh_ddr_axi_arready = 1'b0;
    assign cl_sh_ddr_axi_rid = 16'b0;
    assign cl_sh_ddr_axi_rdata = 512'b0;
    assign cl_sh_ddr_axi_rresp = 2'b0;
    assign cl_sh_ddr_axi_rlast = 1'b0;
    assign cl_sh_ddr_axi_rvalid = 1'b0;
    assign sh_ddr_stat_bus_ack = 1'b1;
    assign sh_ddr_stat_bus_rdata = 32'b0;
    assign ddr_sh_stat_int = 8'b0;
    assign sh_cl_ddr_is_ready = 1'b0;
endmodule

// ============================================================================
// AXI Register Slice Stubs
// ============================================================================

// AXI4 register slice stub - pass through all signals
module axi_register_slice #(
    parameter DATA_WIDTH = 256,
    parameter ADDR_WIDTH = 64,
    parameter ID_WIDTH = 6
) (
    input  wire aclk,
    input  wire aresetn,
    // Slave interface (input)
    input  wire [ID_WIDTH-1:0]   s_axi_awid,
    input  wire [ADDR_WIDTH-1:0] s_axi_awaddr,
    input  wire [7:0]            s_axi_awlen,
    input  wire [2:0]            s_axi_awsize,
    input  wire [1:0]            s_axi_awburst,
    input  wire                  s_axi_awlock,
    input  wire [3:0]            s_axi_awcache,
    input  wire [2:0]            s_axi_awprot,
    input  wire [3:0]            s_axi_awregion,
    input  wire [3:0]            s_axi_awqos,
    input  wire                  s_axi_awvalid,
    output wire                  s_axi_awready,
    input  wire [DATA_WIDTH-1:0] s_axi_wdata,
    input  wire [DATA_WIDTH/8-1:0] s_axi_wstrb,
    input  wire                  s_axi_wlast,
    input  wire                  s_axi_wvalid,
    output wire                  s_axi_wready,
    output wire [ID_WIDTH-1:0]   s_axi_bid,
    output wire [1:0]            s_axi_bresp,
    output wire                  s_axi_bvalid,
    input  wire                  s_axi_bready,
    input  wire [ID_WIDTH-1:0]   s_axi_arid,
    input  wire [ADDR_WIDTH-1:0] s_axi_araddr,
    input  wire [7:0]            s_axi_arlen,
    input  wire [2:0]            s_axi_arsize,
    input  wire [1:0]            s_axi_arburst,
    input  wire                  s_axi_arlock,
    input  wire [3:0]            s_axi_arcache,
    input  wire [2:0]            s_axi_arprot,
    input  wire [3:0]            s_axi_arregion,
    input  wire [3:0]            s_axi_arqos,
    input  wire                  s_axi_arvalid,
    output wire                  s_axi_arready,
    output wire [ID_WIDTH-1:0]   s_axi_rid,
    output wire [DATA_WIDTH-1:0] s_axi_rdata,
    output wire [1:0]            s_axi_rresp,
    output wire                  s_axi_rlast,
    output wire                  s_axi_rvalid,
    input  wire                  s_axi_rready,
    // Master interface (output)
    output wire [ID_WIDTH-1:0]   m_axi_awid,
    output wire [ADDR_WIDTH-1:0] m_axi_awaddr,
    output wire [7:0]            m_axi_awlen,
    output wire [2:0]            m_axi_awsize,
    output wire [1:0]            m_axi_awburst,
    output wire                  m_axi_awlock,
    output wire [3:0]            m_axi_awcache,
    output wire [2:0]            m_axi_awprot,
    output wire [3:0]            m_axi_awregion,
    output wire [3:0]            m_axi_awqos,
    output wire                  m_axi_awvalid,
    input  wire                  m_axi_awready,
    output wire [DATA_WIDTH-1:0] m_axi_wdata,
    output wire [DATA_WIDTH/8-1:0] m_axi_wstrb,
    output wire                  m_axi_wlast,
    output wire                  m_axi_wvalid,
    input  wire                  m_axi_wready,
    input  wire [ID_WIDTH-1:0]   m_axi_bid,
    input  wire [1:0]            m_axi_bresp,
    input  wire                  m_axi_bvalid,
    output wire                  m_axi_bready,
    output wire [ID_WIDTH-1:0]   m_axi_arid,
    output wire [ADDR_WIDTH-1:0] m_axi_araddr,
    output wire [7:0]            m_axi_arlen,
    output wire [2:0]            m_axi_arsize,
    output wire [1:0]            m_axi_arburst,
    output wire                  m_axi_arlock,
    output wire [3:0]            m_axi_arcache,
    output wire [2:0]            m_axi_arprot,
    output wire [3:0]            m_axi_arregion,
    output wire [3:0]            m_axi_arqos,
    output wire                  m_axi_arvalid,
    input  wire                  m_axi_arready,
    input  wire [ID_WIDTH-1:0]   m_axi_rid,
    input  wire [DATA_WIDTH-1:0] m_axi_rdata,
    input  wire [1:0]            m_axi_rresp,
    input  wire                  m_axi_rlast,
    input  wire                  m_axi_rvalid,
    output wire                  m_axi_rready
);
    // Pass-through connections
    assign m_axi_awid     = s_axi_awid;
    assign m_axi_awaddr   = s_axi_awaddr;
    assign m_axi_awlen    = s_axi_awlen;
    assign m_axi_awsize   = s_axi_awsize;
    assign m_axi_awburst  = s_axi_awburst;
    assign m_axi_awlock   = s_axi_awlock;
    assign m_axi_awcache  = s_axi_awcache;
    assign m_axi_awprot   = s_axi_awprot;
    assign m_axi_awregion = s_axi_awregion;
    assign m_axi_awqos    = s_axi_awqos;
    assign m_axi_awvalid  = s_axi_awvalid;
    assign s_axi_awready  = m_axi_awready;
    assign m_axi_wdata    = s_axi_wdata;
    assign m_axi_wstrb    = s_axi_wstrb;
    assign m_axi_wlast    = s_axi_wlast;
    assign m_axi_wvalid   = s_axi_wvalid;
    assign s_axi_wready   = m_axi_wready;
    assign s_axi_bid      = m_axi_bid;
    assign s_axi_bresp    = m_axi_bresp;
    assign s_axi_bvalid   = m_axi_bvalid;
    assign m_axi_bready   = s_axi_bready;
    assign m_axi_arid     = s_axi_arid;
    assign m_axi_araddr   = s_axi_araddr;
    assign m_axi_arlen    = s_axi_arlen;
    assign m_axi_arsize   = s_axi_arsize;
    assign m_axi_arburst  = s_axi_arburst;
    assign m_axi_arlock   = s_axi_arlock;
    assign m_axi_arcache  = s_axi_arcache;
    assign m_axi_arprot   = s_axi_arprot;
    assign m_axi_arregion = s_axi_arregion;
    assign m_axi_arqos    = s_axi_arqos;
    assign m_axi_arvalid  = s_axi_arvalid;
    assign s_axi_arready  = m_axi_arready;
    assign s_axi_rid      = m_axi_rid;
    assign s_axi_rdata    = m_axi_rdata;
    assign s_axi_rresp    = m_axi_rresp;
    assign s_axi_rlast    = m_axi_rlast;
    assign s_axi_rvalid   = m_axi_rvalid;
    assign m_axi_rready   = s_axi_rready;
endmodule

// ============================================================================
// AXI SmartConnect Stubs
// ============================================================================

// AXI4 to AXI3 SmartConnect Wrapper Stub
module cl_axi_sc_1x1_wrapper (
    // Clock and reset
    input  wire         aclk_250,
    input  wire         aresetn_250,
    input  wire         aclk_450,

    // AXI4 Slave Interface (256-bit, 250MHz)
    input  wire [63:0]  AXI4_araddr,
    input  wire [1:0]   AXI4_arburst,
    input  wire [3:0]   AXI4_arcache,
    input  wire [5:0]   AXI4_arid,
    input  wire [7:0]   AXI4_arlen,
    input  wire         AXI4_arlock,
    input  wire [2:0]   AXI4_arprot,
    input  wire [3:0]   AXI4_arqos,
    output wire         AXI4_arready,
    input  wire [2:0]   AXI4_arsize,
    input  wire         AXI4_arvalid,
    input  wire [63:0]  AXI4_awaddr,
    input  wire [1:0]   AXI4_awburst,
    input  wire [3:0]   AXI4_awcache,
    input  wire [5:0]   AXI4_awid,
    input  wire [7:0]   AXI4_awlen,
    input  wire         AXI4_awlock,
    input  wire [2:0]   AXI4_awprot,
    input  wire [3:0]   AXI4_awqos,
    output wire         AXI4_awready,
    input  wire [2:0]   AXI4_awsize,
    input  wire         AXI4_awvalid,
    output wire [5:0]   AXI4_bid,
    input  wire         AXI4_bready,
    output wire [1:0]   AXI4_bresp,
    output wire         AXI4_bvalid,
    output wire [255:0] AXI4_rdata,
    output wire [5:0]   AXI4_rid,
    output wire         AXI4_rlast,
    input  wire         AXI4_rready,
    output wire [1:0]   AXI4_rresp,
    output wire         AXI4_rvalid,
    input  wire [255:0] AXI4_wdata,
    input  wire         AXI4_wlast,
    output wire         AXI4_wready,
    input  wire [31:0]  AXI4_wstrb,
    input  wire         AXI4_wvalid,

    // AXI3 Master Interface (256-bit, 450MHz)
    output wire [63:0]  AXI3_araddr,
    output wire [1:0]   AXI3_arburst,
    output wire [3:0]   AXI3_arcache,
    output wire [3:0]   AXI3_arlen,
    output wire [1:0]   AXI3_arlock,
    output wire [2:0]   AXI3_arprot,
    output wire [3:0]   AXI3_arqos,
    input  wire         AXI3_arready,
    output wire [2:0]   AXI3_arsize,
    output wire         AXI3_arvalid,
    output wire [63:0]  AXI3_awaddr,
    output wire [1:0]   AXI3_awburst,
    output wire [3:0]   AXI3_awcache,
    output wire [3:0]   AXI3_awlen,
    output wire [1:0]   AXI3_awlock,
    output wire [2:0]   AXI3_awprot,
    output wire [3:0]   AXI3_awqos,
    input  wire         AXI3_awready,
    output wire [2:0]   AXI3_awsize,
    output wire         AXI3_awvalid,
    output wire         AXI3_bready,
    input  wire [1:0]   AXI3_bresp,
    input  wire         AXI3_bvalid,
    input  wire [255:0] AXI3_rdata,
    input  wire         AXI3_rlast,
    output wire         AXI3_rready,
    input  wire [1:0]   AXI3_rresp,
    input  wire         AXI3_rvalid,
    output wire [255:0] AXI3_wdata,
    output wire         AXI3_wlast,
    input  wire         AXI3_wready,
    output wire [31:0]  AXI3_wstrb,
    output wire         AXI3_wvalid
);
    // Stub: pass-through connections
    assign AXI3_awaddr  = AXI4_awaddr;
    assign AXI3_awburst = AXI4_awburst;
    assign AXI3_awcache = AXI4_awcache;
    assign AXI3_awlen   = AXI4_awlen[3:0];
    assign AXI3_awlock  = {1'b0, AXI4_awlock};
    assign AXI3_awprot  = AXI4_awprot;
    assign AXI3_awqos   = AXI4_awqos;
    assign AXI4_awready = AXI3_awready;
    assign AXI3_awsize  = AXI4_awsize;
    assign AXI3_awvalid = AXI4_awvalid;
    assign AXI3_wdata   = AXI4_wdata;
    assign AXI3_wstrb   = AXI4_wstrb;
    assign AXI3_wlast   = AXI4_wlast;
    assign AXI3_wvalid  = AXI4_wvalid;
    assign AXI4_wready  = AXI3_wready;
    assign AXI4_bid     = 6'b0;
    assign AXI4_bresp   = AXI3_bresp;
    assign AXI4_bvalid  = AXI3_bvalid;
    assign AXI3_bready  = AXI4_bready;
    assign AXI3_araddr  = AXI4_araddr;
    assign AXI3_arburst = AXI4_arburst;
    assign AXI3_arcache = AXI4_arcache;
    assign AXI3_arlen   = AXI4_arlen[3:0];
    assign AXI3_arlock  = {1'b0, AXI4_arlock};
    assign AXI3_arprot  = AXI4_arprot;
    assign AXI3_arqos   = AXI4_arqos;
    assign AXI4_arready = AXI3_arready;
    assign AXI3_arsize  = AXI4_arsize;
    assign AXI3_arvalid = AXI4_arvalid;
    assign AXI4_rdata   = AXI3_rdata;
    assign AXI4_rid     = 6'b0;
    assign AXI4_rresp   = AXI3_rresp;
    assign AXI4_rlast   = AXI3_rlast;
    assign AXI4_rvalid  = AXI3_rvalid;
    assign AXI3_rready  = AXI4_rready;
endmodule

// AXI3 256-bit Register Slice Stub
module cl_axi3_256b_reg_slice (
    input  wire         aclk,
    input  wire         aresetn,
    // Slave interface
    input  wire [5:0]   s_axi_awid,
    input  wire [33:0]  s_axi_awaddr,
    input  wire [3:0]   s_axi_awlen,
    input  wire [2:0]   s_axi_awsize,
    input  wire [1:0]   s_axi_awburst,
    input  wire [1:0]   s_axi_awlock,
    input  wire [3:0]   s_axi_awcache,
    input  wire [2:0]   s_axi_awprot,
    input  wire [3:0]   s_axi_awqos,
    input  wire         s_axi_awvalid,
    output wire         s_axi_awready,
    input  wire [5:0]   s_axi_wid,
    input  wire [255:0] s_axi_wdata,
    input  wire [31:0]  s_axi_wstrb,
    input  wire         s_axi_wlast,
    input  wire         s_axi_wvalid,
    output wire         s_axi_wready,
    output wire [5:0]   s_axi_bid,
    output wire [1:0]   s_axi_bresp,
    output wire         s_axi_bvalid,
    input  wire         s_axi_bready,
    input  wire [5:0]   s_axi_arid,
    input  wire [33:0]  s_axi_araddr,
    input  wire [3:0]   s_axi_arlen,
    input  wire [2:0]   s_axi_arsize,
    input  wire [1:0]   s_axi_arburst,
    input  wire [1:0]   s_axi_arlock,
    input  wire [3:0]   s_axi_arcache,
    input  wire [2:0]   s_axi_arprot,
    input  wire [3:0]   s_axi_arqos,
    input  wire         s_axi_arvalid,
    output wire         s_axi_arready,
    output wire [5:0]   s_axi_rid,
    output wire [255:0] s_axi_rdata,
    output wire [1:0]   s_axi_rresp,
    output wire         s_axi_rlast,
    output wire         s_axi_rvalid,
    input  wire         s_axi_rready,
    // Master interface
    output wire [5:0]   m_axi_awid,
    output wire [33:0]  m_axi_awaddr,
    output wire [3:0]   m_axi_awlen,
    output wire [2:0]   m_axi_awsize,
    output wire [1:0]   m_axi_awburst,
    output wire [1:0]   m_axi_awlock,
    output wire [3:0]   m_axi_awcache,
    output wire [2:0]   m_axi_awprot,
    output wire [3:0]   m_axi_awqos,
    output wire         m_axi_awvalid,
    input  wire         m_axi_awready,
    output wire [5:0]   m_axi_wid,
    output wire [255:0] m_axi_wdata,
    output wire [31:0]  m_axi_wstrb,
    output wire         m_axi_wlast,
    output wire         m_axi_wvalid,
    input  wire         m_axi_wready,
    input  wire [5:0]   m_axi_bid,
    input  wire [1:0]   m_axi_bresp,
    input  wire         m_axi_bvalid,
    output wire         m_axi_bready,
    output wire [5:0]   m_axi_arid,
    output wire [33:0]  m_axi_araddr,
    output wire [3:0]   m_axi_arlen,
    output wire [2:0]   m_axi_arsize,
    output wire [1:0]   m_axi_arburst,
    output wire [1:0]   m_axi_arlock,
    output wire [3:0]   m_axi_arcache,
    output wire [2:0]   m_axi_arprot,
    output wire [3:0]   m_axi_arqos,
    output wire         m_axi_arvalid,
    input  wire         m_axi_arready,
    input  wire [5:0]   m_axi_rid,
    input  wire [255:0] m_axi_rdata,
    input  wire [1:0]   m_axi_rresp,
    input  wire         m_axi_rlast,
    input  wire         m_axi_rvalid,
    output wire         m_axi_rready
);
    // Pass-through stub
    assign m_axi_awid     = s_axi_awid;
    assign m_axi_awaddr   = s_axi_awaddr;
    assign m_axi_awlen    = s_axi_awlen;
    assign m_axi_awsize   = s_axi_awsize;
    assign m_axi_awburst  = s_axi_awburst;
    assign m_axi_awlock   = s_axi_awlock;
    assign m_axi_awcache  = s_axi_awcache;
    assign m_axi_awprot   = s_axi_awprot;
    assign m_axi_awqos    = s_axi_awqos;
    assign m_axi_awvalid  = s_axi_awvalid;
    assign s_axi_awready  = m_axi_awready;
    assign m_axi_wid      = s_axi_wid;
    assign m_axi_wdata    = s_axi_wdata;
    assign m_axi_wstrb    = s_axi_wstrb;
    assign m_axi_wlast    = s_axi_wlast;
    assign m_axi_wvalid   = s_axi_wvalid;
    assign s_axi_wready   = m_axi_wready;
    assign s_axi_bid      = m_axi_bid;
    assign s_axi_bresp    = m_axi_bresp;
    assign s_axi_bvalid   = m_axi_bvalid;
    assign m_axi_bready   = s_axi_bready;
    assign m_axi_arid     = s_axi_arid;
    assign m_axi_araddr   = s_axi_araddr;
    assign m_axi_arlen    = s_axi_arlen;
    assign m_axi_arsize   = s_axi_arsize;
    assign m_axi_arburst  = s_axi_arburst;
    assign m_axi_arlock   = s_axi_arlock;
    assign m_axi_arcache  = s_axi_arcache;
    assign m_axi_arprot   = s_axi_arprot;
    assign m_axi_arqos    = s_axi_arqos;
    assign m_axi_arvalid  = s_axi_arvalid;
    assign s_axi_arready  = m_axi_arready;
    assign s_axi_rid      = m_axi_rid;
    assign s_axi_rdata    = m_axi_rdata;
    assign s_axi_rresp    = m_axi_rresp;
    assign s_axi_rlast    = m_axi_rlast;
    assign s_axi_rvalid   = m_axi_rvalid;
    assign m_axi_rready   = s_axi_rready;
endmodule
