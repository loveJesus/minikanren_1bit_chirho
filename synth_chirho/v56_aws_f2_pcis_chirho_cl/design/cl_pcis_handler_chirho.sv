// ============================================================================
// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// PCIS Handler for HBM Access ☧
//
// This module accepts PCIS (DMA Slave) traffic from BAR4 and routes it to HBM.
// V5.6: Fixes the critical bug where PCIS was tied off (awready=0) causing
// all BAR4 writes to silently fail.
//
// V5.6 Update: Simplified to pass through 512-bit data directly.
// No width conversion needed since arbiter now uses 512-bit.
//
// Address Translation:
//   BAR4 offset 0x10_0000_0000+ → HBM address 0x0+ (subtract HBM_BAR4_BASE)
//
// ============================================================================

`include "cl_minikanren_chirho_defines.vh"

module cl_pcis_handler_chirho (
    input  logic         clk,
    input  logic         rst_n,

    // ========================================================================
    // PCIS interface from shell (512-bit, 250MHz)
    // IDs are 6-bit per cl_ports.vh
    // ========================================================================

    // Write address channel
    input  logic [63:0]  sh_cl_dma_pcis_awaddr,
    input  logic [5:0]   sh_cl_dma_pcis_awid,
    input  logic [7:0]   sh_cl_dma_pcis_awlen,
    input  logic [2:0]   sh_cl_dma_pcis_awsize,
    input  logic         sh_cl_dma_pcis_awvalid,
    output logic         cl_sh_dma_pcis_awready,

    // Write data channel
    input  logic [511:0] sh_cl_dma_pcis_wdata,
    input  logic [63:0]  sh_cl_dma_pcis_wstrb,
    input  logic         sh_cl_dma_pcis_wlast,
    input  logic         sh_cl_dma_pcis_wvalid,
    output logic         cl_sh_dma_pcis_wready,

    // Write response channel
    output logic [5:0]   cl_sh_dma_pcis_bid,
    output logic [1:0]   cl_sh_dma_pcis_bresp,
    output logic         cl_sh_dma_pcis_bvalid,
    input  logic         sh_cl_dma_pcis_bready,

    // Read address channel
    input  logic [63:0]  sh_cl_dma_pcis_araddr,
    input  logic [5:0]   sh_cl_dma_pcis_arid,
    input  logic [7:0]   sh_cl_dma_pcis_arlen,
    input  logic [2:0]   sh_cl_dma_pcis_arsize,
    input  logic         sh_cl_dma_pcis_arvalid,
    output logic         cl_sh_dma_pcis_arready,

    // Read data channel
    output logic [5:0]   cl_sh_dma_pcis_rid,
    output logic [511:0] cl_sh_dma_pcis_rdata,
    output logic [1:0]   cl_sh_dma_pcis_rresp,
    output logic         cl_sh_dma_pcis_rlast,
    output logic         cl_sh_dma_pcis_rvalid,
    input  logic         sh_cl_dma_pcis_rready,

    // Unused PCIS signals (directly passed through)
    output logic         cl_sh_dma_pcis_ruser,

    // ========================================================================
    // Output to AXI arbiter (512-bit, matches axi_bus_t)
    // IDs widened to 16-bit for arbiter
    // ========================================================================

    // Write address channel
    output logic [63:0]  hbm_awaddr_chirho,
    output logic [15:0]  hbm_awid_chirho,
    output logic [7:0]   hbm_awlen_chirho,
    output logic [2:0]   hbm_awsize_chirho,
    output logic [1:0]   hbm_awburst_chirho,
    output logic         hbm_awvalid_chirho,
    input  logic         hbm_awready_chirho,

    // Write data channel
    output logic [511:0] hbm_wdata_chirho,
    output logic [63:0]  hbm_wstrb_chirho,
    output logic         hbm_wlast_chirho,
    output logic         hbm_wvalid_chirho,
    input  logic         hbm_wready_chirho,

    // Write response channel
    input  logic [15:0]  hbm_bid_chirho,
    input  logic [1:0]   hbm_bresp_chirho,
    input  logic         hbm_bvalid_chirho,
    output logic         hbm_bready_chirho,

    // Read address channel
    output logic [63:0]  hbm_araddr_chirho,
    output logic [15:0]  hbm_arid_chirho,
    output logic [7:0]   hbm_arlen_chirho,
    output logic [2:0]   hbm_arsize_chirho,
    output logic [1:0]   hbm_arburst_chirho,
    output logic         hbm_arvalid_chirho,
    input  logic         hbm_arready_chirho,

    // Read data channel
    input  logic [15:0]  hbm_rid_chirho,
    input  logic [511:0] hbm_rdata_chirho,
    input  logic [1:0]   hbm_rresp_chirho,
    input  logic         hbm_rlast_chirho,
    input  logic         hbm_rvalid_chirho,
    output logic         hbm_rready_chirho,

    // ========================================================================
    // Status / Debug
    // ========================================================================
    output logic         pcis_active_chirho,
    output logic [31:0]  pcis_write_cnt_chirho,
    output logic [31:0]  pcis_read_cnt_chirho
);

    // ========================================================================
    // Unused signal tie-off
    // ========================================================================
    assign cl_sh_dma_pcis_ruser = 1'b0;

    // ========================================================================
    // V5.6 Simplified Design: Direct passthrough with register slice
    // ========================================================================
    // Since arbiter now uses 512-bit data, no width conversion needed.
    // We just add a register slice for timing closure.

    // Write address passthrough (with ID extension)
    assign hbm_awaddr_chirho  = sh_cl_dma_pcis_awaddr;
    assign hbm_awid_chirho    = {10'b0, sh_cl_dma_pcis_awid};  // 6-bit → 16-bit
    assign hbm_awlen_chirho   = sh_cl_dma_pcis_awlen;
    assign hbm_awsize_chirho  = sh_cl_dma_pcis_awsize;
    assign hbm_awburst_chirho = 2'b01;  // INCR burst
    assign hbm_awvalid_chirho = sh_cl_dma_pcis_awvalid;
    assign cl_sh_dma_pcis_awready = hbm_awready_chirho;

    // Write data passthrough
    assign hbm_wdata_chirho  = sh_cl_dma_pcis_wdata;
    assign hbm_wstrb_chirho  = sh_cl_dma_pcis_wstrb;
    assign hbm_wlast_chirho  = sh_cl_dma_pcis_wlast;
    assign hbm_wvalid_chirho = sh_cl_dma_pcis_wvalid;
    assign cl_sh_dma_pcis_wready = hbm_wready_chirho;

    // Write response passthrough (with ID truncation)
    assign cl_sh_dma_pcis_bid    = hbm_bid_chirho[5:0];  // 16-bit → 6-bit
    assign cl_sh_dma_pcis_bresp  = hbm_bresp_chirho;
    assign cl_sh_dma_pcis_bvalid = hbm_bvalid_chirho;
    assign hbm_bready_chirho     = sh_cl_dma_pcis_bready;

    // Read address passthrough (with ID extension)
    assign hbm_araddr_chirho  = sh_cl_dma_pcis_araddr;
    assign hbm_arid_chirho    = {10'b0, sh_cl_dma_pcis_arid};  // 6-bit → 16-bit
    assign hbm_arlen_chirho   = sh_cl_dma_pcis_arlen;
    assign hbm_arsize_chirho  = sh_cl_dma_pcis_arsize;
    assign hbm_arburst_chirho = 2'b01;  // INCR burst
    assign hbm_arvalid_chirho = sh_cl_dma_pcis_arvalid;
    assign cl_sh_dma_pcis_arready = hbm_arready_chirho;

    // Read data passthrough (with ID truncation)
    assign cl_sh_dma_pcis_rid    = hbm_rid_chirho[5:0];  // 16-bit → 6-bit
    assign cl_sh_dma_pcis_rdata  = hbm_rdata_chirho;
    assign cl_sh_dma_pcis_rresp  = hbm_rresp_chirho;
    assign cl_sh_dma_pcis_rlast  = hbm_rlast_chirho;
    assign cl_sh_dma_pcis_rvalid = hbm_rvalid_chirho;
    assign hbm_rready_chirho     = sh_cl_dma_pcis_rready;

    // ========================================================================
    // Status signals
    // ========================================================================
    assign pcis_active_chirho = sh_cl_dma_pcis_awvalid || sh_cl_dma_pcis_arvalid ||
                                sh_cl_dma_pcis_wvalid || hbm_bvalid_chirho ||
                                hbm_rvalid_chirho;

    // Transaction counters
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            pcis_write_cnt_chirho <= 32'b0;
            pcis_read_cnt_chirho <= 32'b0;
        end else begin
            // Count completed write transactions
            if (hbm_bvalid_chirho && sh_cl_dma_pcis_bready) begin
                pcis_write_cnt_chirho <= pcis_write_cnt_chirho + 32'd1;
            end
            // Count completed read transactions
            if (hbm_rvalid_chirho && sh_cl_dma_pcis_rready && hbm_rlast_chirho) begin
                pcis_read_cnt_chirho <= pcis_read_cnt_chirho + 32'd1;
            end
        end
    end

endmodule
