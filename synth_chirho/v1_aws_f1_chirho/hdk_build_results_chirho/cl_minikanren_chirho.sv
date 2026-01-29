// ============================================================================
// ☧ For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// AWS F1 Custom Logic: miniKanren 1-Bit Search Engine
// Based on CL_TEMPLATE structure for HDK v2.2+ (f2 branch)
// ============================================================================

module cl_minikanren_chirho #(
    parameter EN_DDR = 0,
    parameter EN_HBM = 0
) (
    `include "cl_ports.vh"
);

`include "cl_id_defines.vh"
`include "cl_minikanren_chirho_defines.vh"

// ========================================================================
// Global Signals
// ========================================================================

always_comb begin
    cl_sh_flr_done    = 1'b1;
    cl_sh_status0     = 32'b0;
    cl_sh_status1     = 32'b0;
    cl_sh_status2     = 32'b0;
    cl_sh_id0         = `CL_SH_ID0;
    cl_sh_id1         = `CL_SH_ID1;
    cl_sh_dma_wr_full = 1'b0;
    cl_sh_dma_rd_full = 1'b0;
end

// ========================================================================
// Tie-off unused PCIM interface
// ========================================================================

always_comb begin
    cl_sh_pcim_awaddr  = 64'b0;
    cl_sh_pcim_awid    = 16'b0;
    cl_sh_pcim_awlen   = 8'b0;
    cl_sh_pcim_awsize  = 3'b0;
    cl_sh_pcim_awburst = 2'b0;
    cl_sh_pcim_awcache = 4'b0;
    cl_sh_pcim_awlock  = 1'b0;
    cl_sh_pcim_awprot  = 3'b0;
    cl_sh_pcim_awqos   = 4'b0;
    cl_sh_pcim_awuser  = 55'b0;
    cl_sh_pcim_awvalid = 1'b0;
    
    cl_sh_pcim_wid     = 16'b0;
    cl_sh_pcim_wdata   = 512'b0;
    cl_sh_pcim_wstrb   = 64'b0;
    cl_sh_pcim_wlast   = 1'b0;
    cl_sh_pcim_wuser   = 64'b0;
    cl_sh_pcim_wvalid  = 1'b0;
    
    cl_sh_pcim_bready  = 1'b0;
    
    cl_sh_pcim_arid    = 16'b0;
    cl_sh_pcim_araddr  = 64'b0;
    cl_sh_pcim_arlen   = 8'b0;
    cl_sh_pcim_arsize  = 3'b0;
    cl_sh_pcim_arburst = 2'b0;
    cl_sh_pcim_arcache = 4'b0;
    cl_sh_pcim_arlock  = 1'b0;
    cl_sh_pcim_arprot  = 3'b0;
    cl_sh_pcim_arqos   = 4'b0;
    cl_sh_pcim_aruser  = 55'b0;
    cl_sh_pcim_arvalid = 1'b0;
    
    cl_sh_pcim_rready  = 1'b0;
end

// ========================================================================
// Tie-off unused DMA PCIS interface
// ========================================================================

always_comb begin
    cl_sh_dma_pcis_awready = 1'b0;
    cl_sh_dma_pcis_wready  = 1'b0;
    cl_sh_dma_pcis_bid     = 6'b0;
    cl_sh_dma_pcis_bresp   = 2'b0;
    cl_sh_dma_pcis_bvalid  = 1'b0;
    cl_sh_dma_pcis_arready = 1'b0;
    cl_sh_dma_pcis_rid     = 6'b0;
    cl_sh_dma_pcis_rdata   = 512'b0;
    cl_sh_dma_pcis_rresp   = 2'b0;
    cl_sh_dma_pcis_rlast   = 1'b0;
    cl_sh_dma_pcis_ruser   = 18'b0;
    cl_sh_dma_pcis_rvalid  = 1'b0;
end

// ========================================================================
// Tie-off unused SDA interface
// ========================================================================

always_comb begin
    cl_sda_awready = 1'b0;
    cl_sda_wready  = 1'b0;
    cl_sda_bresp   = 2'b0;
    cl_sda_bvalid  = 1'b0;
    cl_sda_arready = 1'b0;
    cl_sda_rdata   = 32'b0;
    cl_sda_rresp   = 2'b0;
    cl_sda_rvalid  = 1'b0;
end

// ========================================================================
// DDR Stat interface tie-offs
// ========================================================================

always_comb begin
    cl_sh_ddr_stat_ack   = 1'b0;
    cl_sh_ddr_stat_rdata = 32'b0;
    cl_sh_ddr_stat_int   = 1'b0;
end

// ========================================================================
// sh_ddr module (required even if not using DDR)
// ========================================================================

sh_ddr #(
    .DDR_PRESENT(EN_DDR)
) SH_DDR (
    .clk                      (clk_main_a0),
    .rst_n                    (),
    .stat_clk                 (clk_main_a0),
    .stat_rst_n               (),
    .CLK_DIMM_DP              (CLK_DIMM_DP),
    .CLK_DIMM_DN              (CLK_DIMM_DN),
    .M_ACT_N                  (M_ACT_N),
    .M_MA                     (M_MA),
    .M_BA                     (M_BA),
    .M_BG                     (M_BG),
    .M_CKE                    (M_CKE),
    .M_ODT                    (M_ODT),
    .M_CS_N                   (M_CS_N),
    .M_CLK_DN                 (M_CLK_DN),
    .M_CLK_DP                 (M_CLK_DP),
    .M_PAR                    (M_PAR),
    .M_DQ                     (M_DQ),
    .M_ECC                    (M_ECC),
    .M_DQS_DP                 (M_DQS_DP),
    .M_DQS_DN                 (M_DQS_DN),
    .cl_RST_DIMM_N            (RST_DIMM_N),
    .cl_sh_ddr_axi_awid       (),
    .cl_sh_ddr_axi_awaddr     (),
    .cl_sh_ddr_axi_awlen      (),
    .cl_sh_ddr_axi_awsize     (),
    .cl_sh_ddr_axi_awvalid    (),
    .cl_sh_ddr_axi_awburst    (),
    .cl_sh_ddr_axi_awuser     (),
    .cl_sh_ddr_axi_awready    (),
    .cl_sh_ddr_axi_wdata      (),
    .cl_sh_ddr_axi_wstrb      (),
    .cl_sh_ddr_axi_wlast      (),
    .cl_sh_ddr_axi_wvalid     (),
    .cl_sh_ddr_axi_wready     (),
    .cl_sh_ddr_axi_bid        (),
    .cl_sh_ddr_axi_bresp      (),
    .cl_sh_ddr_axi_bvalid     (),
    .cl_sh_ddr_axi_bready     (),
    .cl_sh_ddr_axi_arid       (),
    .cl_sh_ddr_axi_araddr     (),
    .cl_sh_ddr_axi_arlen      (),
    .cl_sh_ddr_axi_arsize     (),
    .cl_sh_ddr_axi_arvalid    (),
    .cl_sh_ddr_axi_arburst    (),
    .cl_sh_ddr_axi_aruser     (),
    .cl_sh_ddr_axi_arready    (),
    .cl_sh_ddr_axi_rid        (),
    .cl_sh_ddr_axi_rdata      (),
    .cl_sh_ddr_axi_rresp      (),
    .cl_sh_ddr_axi_rlast      (),
    .cl_sh_ddr_axi_rvalid     (),
    .cl_sh_ddr_axi_rready     (),
    .sh_ddr_stat_bus_addr     (),
    .sh_ddr_stat_bus_wdata    (),
    .sh_ddr_stat_bus_wr       (),
    .sh_ddr_stat_bus_rd       (),
    .sh_ddr_stat_bus_ack      (),
    .sh_ddr_stat_bus_rdata    (),
    .ddr_sh_stat_int          (),
    .sh_cl_ddr_is_ready       ()
);

// ========================================================================
// Tie-off APP_PF_IRQ
// ========================================================================

always_comb begin
    cl_sh_apppf_irq_req = 16'b0;
end

// ========================================================================
// Tie-off JTAG
// ========================================================================

always_comb begin
    tdo = 1'b0;
end

// ========================================================================
// Tie-off HBM APB interfaces
// ========================================================================

always_comb begin
    hbm_apb_paddr_0   = 22'b0;
    hbm_apb_pprot_0   = 3'b0;
    hbm_apb_psel_0    = 1'b0;
    hbm_apb_penable_0 = 1'b0;
    hbm_apb_pwrite_0  = 1'b0;
    hbm_apb_pwdata_0  = 32'b0;
    hbm_apb_pstrb_0   = 4'b0;
    hbm_apb_pready_0  = 1'b0;
    hbm_apb_prdata_0  = 32'b0;
    hbm_apb_pslverr_0 = 1'b0;

    hbm_apb_paddr_1   = 22'b0;
    hbm_apb_pprot_1   = 3'b0;
    hbm_apb_psel_1    = 1'b0;
    hbm_apb_penable_1 = 1'b0;
    hbm_apb_pwrite_1  = 1'b0;
    hbm_apb_pwdata_1  = 32'b0;
    hbm_apb_pstrb_1   = 4'b0;
    hbm_apb_pready_1  = 1'b0;
    hbm_apb_prdata_1  = 32'b0;
    hbm_apb_pslverr_1 = 1'b0;
end

// ========================================================================
// Tie-off PCIe EP/RP
// ========================================================================

always_comb begin
    PCIE_EP_TXP    = 16'b0;
    PCIE_EP_TXN    = 16'b0;
    PCIE_RP_PERSTN = 1'b0;
    PCIE_RP_TXP    = 16'b0;
    PCIE_RP_TXN    = 16'b0;
end

// ========================================================================
// Internal Signals for miniKanren Engine
// ========================================================================

logic rst_main_n_sync_chirho;
logic engine_rst_chirho;
logic clk_engine_chirho;
logic [2:0] clk_div_chirho;

logic ctrl_enable_chirho;
logic ctrl_reset_chirho;
logic [69:0] cmd_reg_chirho;
logic [513:0] resp_wire_chirho;

// AXI-Lite OCL control signals
logic        ocl_awvalid_q_chirho;
logic        ocl_awready_q_chirho;
logic [31:0] ocl_awaddr_q_chirho;
logic        ocl_wvalid_q_chirho;
logic        ocl_wready_q_chirho;
logic [31:0] ocl_wdata_q_chirho;
logic        ocl_bvalid_q_chirho;
logic        ocl_bready_q_chirho;
logic [1:0]  ocl_bresp_q_chirho;
logic        ocl_arvalid_q_chirho;
logic        ocl_arready_q_chirho;
logic [31:0] ocl_araddr_q_chirho;
logic        ocl_rvalid_q_chirho;
logic        ocl_rready_q_chirho;
logic [31:0] ocl_rdata_q_chirho;
logic [1:0]  ocl_rresp_q_chirho;

// ========================================================================
// Reset Synchronizer  
// ========================================================================

lib_pipe #(.WIDTH(1), .STAGES(4)) rst_pipe_chirho (
    .clk    (clk_main_a0),
    .rst_n  (1'b1),
    .in_bus (rst_main_n),
    .out_bus(rst_main_n_sync_chirho)
);

// ========================================================================
// Clock Divider (250MHz -> 62.5MHz)
// ========================================================================

always_ff @(posedge clk_main_a0 or negedge rst_main_n_sync_chirho) begin
    if (!rst_main_n_sync_chirho) begin
        clk_div_chirho <= 3'b0;
    end else begin
        clk_div_chirho <= clk_div_chirho + 1'b1;
    end
end

assign clk_engine_chirho = clk_div_chirho[1];
assign engine_rst_chirho = !rst_main_n_sync_chirho || ctrl_reset_chirho;

// ========================================================================
// AXI-Lite Register Stage (OCL interface)
// ========================================================================

always_ff @(posedge clk_main_a0) begin
    if (!rst_main_n_sync_chirho) begin
        ocl_awvalid_q_chirho <= 1'b0;
        ocl_wvalid_q_chirho  <= 1'b0;
        ocl_arvalid_q_chirho <= 1'b0;
        ocl_bready_q_chirho  <= 1'b0;
        ocl_rready_q_chirho  <= 1'b0;
    end else begin
        ocl_awvalid_q_chirho <= ocl_cl_awvalid;
        ocl_awaddr_q_chirho  <= ocl_cl_awaddr;
        ocl_wvalid_q_chirho  <= ocl_cl_wvalid;
        ocl_wdata_q_chirho   <= ocl_cl_wdata;
        ocl_arvalid_q_chirho <= ocl_cl_arvalid;
        ocl_araddr_q_chirho  <= ocl_cl_araddr;
        ocl_bready_q_chirho  <= ocl_cl_bready;
        ocl_rready_q_chirho  <= ocl_cl_rready;
    end
end

// ========================================================================
// AXI-Lite Write Logic
// ========================================================================

typedef enum logic [1:0] {
    WR_IDLE_CHIRHO,
    WR_DATA_CHIRHO,
    WR_RESP_CHIRHO
} wr_state_t_chirho;

wr_state_t_chirho wr_state_chirho;
logic [31:0] wr_addr_chirho;

always_ff @(posedge clk_main_a0) begin
    if (!rst_main_n_sync_chirho) begin
        wr_state_chirho      <= WR_IDLE_CHIRHO;
        ocl_awready_q_chirho <= 1'b0;
        ocl_wready_q_chirho  <= 1'b0;
        ocl_bvalid_q_chirho  <= 1'b0;
        ocl_bresp_q_chirho   <= 2'b00;
        ctrl_enable_chirho   <= 1'b0;
        ctrl_reset_chirho    <= 1'b0;
        cmd_reg_chirho       <= 70'b0;
    end else begin
        case (wr_state_chirho)
            WR_IDLE_CHIRHO: begin
                ocl_awready_q_chirho <= 1'b1;
                ocl_bvalid_q_chirho  <= 1'b0;
                if (ocl_awvalid_q_chirho && ocl_awready_q_chirho) begin
                    wr_addr_chirho       <= ocl_awaddr_q_chirho;
                    ocl_awready_q_chirho <= 1'b0;
                    ocl_wready_q_chirho  <= 1'b1;
                    wr_state_chirho      <= WR_DATA_CHIRHO;
                end
            end

            WR_DATA_CHIRHO: begin
                if (ocl_wvalid_q_chirho && ocl_wready_q_chirho) begin
                    ocl_wready_q_chirho <= 1'b0;
                    case (wr_addr_chirho[7:2])
                        6'h01: begin
                            ctrl_enable_chirho <= ocl_wdata_q_chirho[0];
                            ctrl_reset_chirho  <= ocl_wdata_q_chirho[1];
                        end
                        6'h04: cmd_reg_chirho[31:0]   <= ocl_wdata_q_chirho;
                        6'h05: cmd_reg_chirho[63:32]  <= ocl_wdata_q_chirho;
                        6'h06: cmd_reg_chirho[69:64]  <= ocl_wdata_q_chirho[5:0];
                    endcase
                    ocl_bvalid_q_chirho <= 1'b1;
                    ocl_bresp_q_chirho  <= 2'b00;
                    wr_state_chirho     <= WR_RESP_CHIRHO;
                end
            end

            WR_RESP_CHIRHO: begin
                if (ocl_bready_q_chirho) begin
                    ocl_bvalid_q_chirho <= 1'b0;
                    wr_state_chirho     <= WR_IDLE_CHIRHO;
                end
            end
        endcase
    end
end

// ========================================================================
// AXI-Lite Read Logic
// ========================================================================

typedef enum logic [1:0] {
    RD_IDLE_CHIRHO,
    RD_DATA_CHIRHO
} rd_state_t_chirho;

rd_state_t_chirho rd_state_chirho;

always_ff @(posedge clk_main_a0) begin
    if (!rst_main_n_sync_chirho) begin
        rd_state_chirho      <= RD_IDLE_CHIRHO;
        ocl_arready_q_chirho <= 1'b0;
        ocl_rvalid_q_chirho  <= 1'b0;
        ocl_rdata_q_chirho   <= 32'b0;
        ocl_rresp_q_chirho   <= 2'b00;
    end else begin
        case (rd_state_chirho)
            RD_IDLE_CHIRHO: begin
                ocl_arready_q_chirho <= 1'b1;
                ocl_rvalid_q_chirho  <= 1'b0;
                if (ocl_arvalid_q_chirho && ocl_arready_q_chirho) begin
                    ocl_arready_q_chirho <= 1'b0;
                    case (ocl_araddr_q_chirho[7:2])
                        6'h00: ocl_rdata_q_chirho <= `MINIKANREN_VERSION_CHIRHO;
                        6'h01: ocl_rdata_q_chirho <= {30'b0, ctrl_reset_chirho, ctrl_enable_chirho};
                        6'h02: ocl_rdata_q_chirho <= {30'b0, 1'b1, 1'b1};
                        6'h04: ocl_rdata_q_chirho <= cmd_reg_chirho[31:0];
                        6'h05: ocl_rdata_q_chirho <= cmd_reg_chirho[63:32];
                        6'h06: ocl_rdata_q_chirho <= {26'b0, cmd_reg_chirho[69:64]};
                        6'h08: ocl_rdata_q_chirho <= resp_wire_chirho[31:0];
                        6'h09: ocl_rdata_q_chirho <= resp_wire_chirho[63:32];
                        6'h0A: ocl_rdata_q_chirho <= resp_wire_chirho[95:64];
                        6'h0B: ocl_rdata_q_chirho <= resp_wire_chirho[127:96];
                        6'h0C: ocl_rdata_q_chirho <= resp_wire_chirho[159:128];
                        6'h0D: ocl_rdata_q_chirho <= resp_wire_chirho[191:160];
                        6'h0E: ocl_rdata_q_chirho <= resp_wire_chirho[223:192];
                        6'h0F: ocl_rdata_q_chirho <= resp_wire_chirho[255:224];
                        6'h10: ocl_rdata_q_chirho <= resp_wire_chirho[287:256];
                        6'h11: ocl_rdata_q_chirho <= resp_wire_chirho[319:288];
                        6'h12: ocl_rdata_q_chirho <= resp_wire_chirho[351:320];
                        6'h13: ocl_rdata_q_chirho <= resp_wire_chirho[383:352];
                        6'h14: ocl_rdata_q_chirho <= resp_wire_chirho[415:384];
                        6'h15: ocl_rdata_q_chirho <= resp_wire_chirho[447:416];
                        6'h16: ocl_rdata_q_chirho <= resp_wire_chirho[479:448];
                        6'h17: ocl_rdata_q_chirho <= resp_wire_chirho[511:480];
                        6'h18: ocl_rdata_q_chirho <= {30'b0, resp_wire_chirho[513:512]};
                        default: ocl_rdata_q_chirho <= 32'hDEADBEEF;
                    endcase
                    ocl_rvalid_q_chirho <= 1'b1;
                    ocl_rresp_q_chirho  <= 2'b00;
                    rd_state_chirho     <= RD_DATA_CHIRHO;
                end
            end

            RD_DATA_CHIRHO: begin
                if (ocl_rready_q_chirho) begin
                    ocl_rvalid_q_chirho <= 1'b0;
                    rd_state_chirho     <= RD_IDLE_CHIRHO;
                end
            end
        endcase
    end
end

// ========================================================================
// AXI-Lite Output Assignments (OCL)
// ========================================================================

assign cl_ocl_awready = ocl_awready_q_chirho;
assign cl_ocl_wready  = ocl_wready_q_chirho;
assign cl_ocl_bvalid  = ocl_bvalid_q_chirho;
assign cl_ocl_bresp   = ocl_bresp_q_chirho;
assign cl_ocl_arready = ocl_arready_q_chirho;
assign cl_ocl_rvalid  = ocl_rvalid_q_chirho;
assign cl_ocl_rdata   = ocl_rdata_q_chirho;
assign cl_ocl_rresp   = ocl_rresp_q_chirho;

// ========================================================================
// Virtual LED
// ========================================================================

assign cl_sh_status_vled[0]    = ctrl_enable_chirho;
assign cl_sh_status_vled[1]    = engine_rst_chirho;
assign cl_sh_status_vled[7:2]  = cmd_reg_chirho[69:64];
assign cl_sh_status_vled[15:8] = resp_wire_chirho[7:0];

// ========================================================================
// miniKanren Search Engine Instance
// ========================================================================

searchEngineChirho u_engine_chirho (
    .clk        (clk_engine_chirho),
    .rst        (engine_rst_chirho),
    .enChirho   (ctrl_enable_chirho),
    .cmdChirho  (cmd_reg_chirho),
    .respChirho (resp_wire_chirho)
);

endmodule
