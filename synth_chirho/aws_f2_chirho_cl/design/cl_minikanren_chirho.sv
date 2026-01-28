// ============================================================================
// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// AWS F2 Custom Logic: miniKanren 1-Bit Search Engine with HBM
//
// This module integrates the Clash-generated miniKanren search engine with
// the AWS F2 shell and HBM for high-bandwidth domain storage.
//
// Architecture:
//   - OCL AXI-Lite: Control/status registers from host
//   - HBM: Variable domains (256K values per var), term store, tabling cache
//   - Search Engine: Domain intersection, unification FSM
//
// HBM Layout (16 GB):
//   0x0_0000_0000 - 0x0_1FFF_FFFF: Variable headers (512 MB, 32M vars × 16B)
//   0x0_2000_0000 - 0x2_1FFF_FFFF: Variable domains (8 GB)
//   0x1_0000_0000 - 0x1_FFFF_FFFF: Term store (4 GB, 64M terms × 64B)
//   0x2_0000_0000 - 0x2_0FFF_FFFF: Hash table (256 MB)
//   0x2_1000_0000 - 0x2_8FFF_FFFF: Tabling cache (2 GB)
//
// Register Map (via OCL AXI-Lite):
//   0x000: VERSION   - Read-only version register
//   0x004: CONTROL   - bit0=enable, bit1=reset, bit2=hbm_mode
//   0x008: STATUS    - bit0=done, bit1=valid, bit2=hbm_ready
//   0x010: CMD_LO    - cmdChirho[31:0]
//   0x014: CMD_MID   - cmdChirho[63:32]
//   0x018: CMD_HI    - cmdChirho[69:64]
//   0x020-0x060: RESP[0-8] - respChirho[513:0] (9 x 64-bit)
//
// Naming Conventions (per AGENTS.md):
//   - Internal signals:    snake_chirho      (e.g., clk_engine_chirho)
//   - Constants/enums:     UPPER_CHIRHO      (e.g., FSM_IDLE_CHIRHO)
//   - Types:               snake_chirho_t    (e.g., wr_state_t_chirho)
//   - Module instances:    snake_chirho      (e.g., u_engine_chirho)
//   - Clash-generated:     camelChirho       (e.g., enChirho, cmdChirho)
//   - AWS HDK primitives:  original names    (e.g., clk_main_a0, rst_main_n)
// ============================================================================

`include "cl_minikanren_chirho_defines.vh"

module cl_minikanren_chirho
    #(
      parameter EN_DDR = 0,
      parameter EN_HBM = 1  // Enable HBM for domain storage
    )
    (
      `include "cl_ports.vh"
    );

`include "cl_id_defines.vh"
`include "cl_dram_dma_defines.vh"

    // ========================================================================
    // Global Tie-offs (required by F2 shell)
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
    // PCIM Tie-offs (not using DMA master)
    // ========================================================================

    always_comb begin
        cl_sh_pcim_awid    = 16'b0;
        cl_sh_pcim_awaddr  = 64'b0;
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

        cl_sh_pcim_bready  = 1'b1;

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

        cl_sh_pcim_rready  = 1'b1;
    end

    // ========================================================================
    // PCIS (DMA Slave) Tie-offs
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
        cl_sh_dma_pcis_ruser   = 1'b0;
        cl_sh_dma_pcis_rvalid  = 1'b0;
    end

    // ========================================================================
    // SDA Tie-offs
    // ========================================================================

    always_comb begin
        cl_sda_awready = 1'b0;
        cl_sda_wready  = 1'b0;
        cl_sda_bvalid  = 1'b0;
        cl_sda_bresp   = 2'b0;
        cl_sda_arready = 1'b0;
        cl_sda_rdata   = 32'b0;
        cl_sda_rresp   = 2'b0;
        cl_sda_rvalid  = 1'b0;
    end

    // ========================================================================
    // DDR Tie-offs
    // ========================================================================

    sh_ddr
      #(
        .DDR_PRESENT (EN_DDR)
      )
    SH_DDR
      (
       .clk                       (clk_main_a0 ),
       .rst_n                     (rst_main_n  ),
       .stat_clk                  (clk_main_a0 ),
       .stat_rst_n                (rst_main_n  ),
       .CLK_DIMM_DP               (CLK_DIMM_DP ),
       .CLK_DIMM_DN               (CLK_DIMM_DN ),
       .M_ACT_N                   (M_ACT_N     ),
       .M_MA                      (M_MA        ),
       .M_BA                      (M_BA        ),
       .M_BG                      (M_BG        ),
       .M_CKE                     (M_CKE       ),
       .M_ODT                     (M_ODT       ),
       .M_CS_N                    (M_CS_N      ),
       .M_CLK_DN                  (M_CLK_DN    ),
       .M_CLK_DP                  (M_CLK_DP    ),
       .M_PAR                     (M_PAR       ),
       .M_DQ                      (M_DQ        ),
       .M_ECC                     (M_ECC       ),
       .M_DQS_DP                  (M_DQS_DP    ),
       .M_DQS_DN                  (M_DQS_DN    ),
       .cl_RST_DIMM_N             (RST_DIMM_N  ),
       .cl_sh_ddr_axi_awid        (            ),
       .cl_sh_ddr_axi_awaddr      (            ),
       .cl_sh_ddr_axi_awlen       (            ),
       .cl_sh_ddr_axi_awsize      (            ),
       .cl_sh_ddr_axi_awvalid     (1'b0        ),
       .cl_sh_ddr_axi_awburst     (            ),
       .cl_sh_ddr_axi_awuser      (            ),
       .cl_sh_ddr_axi_awready     (            ),
       .cl_sh_ddr_axi_wdata       (            ),
       .cl_sh_ddr_axi_wstrb       (            ),
       .cl_sh_ddr_axi_wlast       (            ),
       .cl_sh_ddr_axi_wvalid      (1'b0        ),
       .cl_sh_ddr_axi_wready      (            ),
       .cl_sh_ddr_axi_bid         (            ),
       .cl_sh_ddr_axi_bresp       (            ),
       .cl_sh_ddr_axi_bvalid      (            ),
       .cl_sh_ddr_axi_bready      (1'b1        ),
       .cl_sh_ddr_axi_arid        (            ),
       .cl_sh_ddr_axi_araddr      (            ),
       .cl_sh_ddr_axi_arlen       (            ),
       .cl_sh_ddr_axi_arsize      (            ),
       .cl_sh_ddr_axi_arvalid     (1'b0        ),
       .cl_sh_ddr_axi_arburst     (            ),
       .cl_sh_ddr_axi_aruser      (            ),
       .cl_sh_ddr_axi_arready     (            ),
       .cl_sh_ddr_axi_rid         (            ),
       .cl_sh_ddr_axi_rdata       (            ),
       .cl_sh_ddr_axi_rresp       (            ),
       .cl_sh_ddr_axi_rlast       (            ),
       .cl_sh_ddr_axi_rvalid      (            ),
       .cl_sh_ddr_axi_rready      (1'b1        ),
       .sh_ddr_stat_bus_addr      (            ),
       .sh_ddr_stat_bus_wdata     (            ),
       .sh_ddr_stat_bus_wr        (            ),
       .sh_ddr_stat_bus_rd        (            ),
       .sh_ddr_stat_bus_ack       (            ),
       .sh_ddr_stat_bus_rdata     (            ),
       .ddr_sh_stat_int           (            ),
       .sh_cl_ddr_is_ready        (            )
      );

    always_comb begin
        cl_sh_ddr_stat_ack   = 1'b0;
        cl_sh_ddr_stat_rdata = 32'b0;
        cl_sh_ddr_stat_int   = 1'b0;
    end

    // ========================================================================
    // Interrupt Tie-offs
    // ========================================================================

    always_comb begin
        cl_sh_apppf_irq_req = 16'b0;
    end

    // ========================================================================
    // Virtual JTAG Tie-off
    // ========================================================================

    always_comb begin
        tdo = 1'b0;
    end

    // ========================================================================
    // PCIe EP/RP Tie-offs
    // ========================================================================

    always_comb begin
        PCIE_EP_TXP    = 16'b0;
        PCIE_EP_TXN    = 16'b0;
        PCIE_RP_PERSTN = 1'b0;
        PCIE_RP_TXP    = 16'b0;
        PCIE_RP_TXN    = 16'b0;
    end

    // ========================================================================
    // Internal Signals
    // ========================================================================

    logic rst_main_n_sync_chirho;
    logic engine_rst_chirho;
    logic clk_engine_chirho;
    logic [2:0] clk_div_chirho;

    logic ctrl_enable_chirho;
    logic ctrl_reset_chirho;
    logic ctrl_hbm_mode_chirho;
    logic [69:0] cmd_reg_chirho;
    logic [513:0] resp_wire_chirho;

    // HBM interface signals
    logic hbm_ready_chirho;
    logic [7:0] hbm_stat_int_chirho;

    // AXI4 bus interface for HBM
    axi_bus_t hbm_axi4_bus_chirho();
    cfg_bus_t hbm_stat_bus_chirho();

    // ========================================================================
    // Reset Synchronizer
    // ========================================================================

    logic rst_sync_1_chirho, rst_sync_2_chirho;

    always_ff @(posedge clk_main_a0 or negedge rst_main_n) begin
        if (!rst_main_n) begin
            rst_sync_1_chirho <= 1'b0;
            rst_sync_2_chirho <= 1'b0;
        end else begin
            rst_sync_1_chirho <= 1'b1;
            rst_sync_2_chirho <= rst_sync_1_chirho;
        end
    end

    assign rst_main_n_sync_chirho = rst_sync_2_chirho;

    // ========================================================================
    // Clock Divider (250MHz -> ~62.5MHz for legacy engine)
    // ========================================================================

    always_ff @(posedge clk_main_a0 or negedge rst_main_n_sync_chirho) begin
        if (!rst_main_n_sync_chirho) begin
            clk_div_chirho <= 3'b0;
        end else begin
            clk_div_chirho <= clk_div_chirho + 1'b1;
        end
    end

    assign clk_engine_chirho = clk_div_chirho[1]; // Divide by 4 for legacy
    assign engine_rst_chirho = !rst_main_n_sync_chirho || ctrl_reset_chirho;

    // ========================================================================
    // HBM Interface (when EN_HBM=1)
    // ========================================================================

if (EN_HBM) begin : HBM_ENABLED

    // HBM stats bus tie-off (no dynamic control needed)
    assign hbm_stat_bus_chirho.wr    = 1'b0;
    assign hbm_stat_bus_chirho.rd    = 1'b0;
    assign hbm_stat_bus_chirho.addr  = 8'b0;
    assign hbm_stat_bus_chirho.wdata = 32'b0;

    // Instantiate HBM AXI4 interface
    cl_hbm_axi4
    #(
        .HBM_PRESENT(1)
    )
    HBM_AXI4_CHIRHO
    (
        .clk_hbm_ref            (clk_hbm_ref              ),
        .clk                    (clk_main_a0              ),
        .rst_n                  (rst_main_n_sync_chirho   ),

        // AXI4 bus from our engine
        .hbm_axi4_bus           (hbm_axi4_bus_chirho      ),

        // Stats bus
        .hbm_stat_bus           (hbm_stat_bus_chirho      ),

        // APB monitor ports to shell
        .i_hbm_apb_preset_n_1   (hbm_apb_preset_n_1       ),
        .o_hbm_apb_paddr_1      (hbm_apb_paddr_1          ),
        .o_hbm_apb_pprot_1      (hbm_apb_pprot_1          ),
        .o_hbm_apb_psel_1       (hbm_apb_psel_1           ),
        .o_hbm_apb_penable_1    (hbm_apb_penable_1        ),
        .o_hbm_apb_pwrite_1     (hbm_apb_pwrite_1         ),
        .o_hbm_apb_pwdata_1     (hbm_apb_pwdata_1         ),
        .o_hbm_apb_pstrb_1      (hbm_apb_pstrb_1          ),
        .o_hbm_apb_pready_1     (hbm_apb_pready_1         ),
        .o_hbm_apb_prdata_1     (hbm_apb_prdata_1         ),
        .o_hbm_apb_pslverr_1    (hbm_apb_pslverr_1        ),

        .i_hbm_apb_preset_n_0   (hbm_apb_preset_n_0       ),
        .o_hbm_apb_paddr_0      (hbm_apb_paddr_0          ),
        .o_hbm_apb_pprot_0      (hbm_apb_pprot_0          ),
        .o_hbm_apb_psel_0       (hbm_apb_psel_0           ),
        .o_hbm_apb_penable_0    (hbm_apb_penable_0        ),
        .o_hbm_apb_pwrite_0     (hbm_apb_pwrite_0         ),
        .o_hbm_apb_pwdata_0     (hbm_apb_pwdata_0         ),
        .o_hbm_apb_pstrb_0      (hbm_apb_pstrb_0          ),
        .o_hbm_apb_pready_0     (hbm_apb_pready_0         ),
        .o_hbm_apb_prdata_0     (hbm_apb_prdata_0         ),
        .o_hbm_apb_pslverr_0    (hbm_apb_pslverr_0        ),

        .o_cl_sh_hbm_stat_int   (hbm_stat_int_chirho      ),
        .o_hbm_ready            (hbm_ready_chirho         )
    );

end : HBM_ENABLED
else begin : HBM_DISABLED

    // HBM APB tie-offs when HBM is disabled
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

    assign hbm_ready_chirho = 1'b0;
    assign hbm_stat_int_chirho = 8'b0;

end : HBM_DISABLED

    // ========================================================================
    // OCL AXI-Lite Write Logic (F2 port names)
    // ========================================================================

    typedef enum logic [1:0] {
        WR_IDLE_CHIRHO,
        WR_DATA_CHIRHO,
        WR_RESP_CHIRHO
    } wr_state_t_chirho;

    wr_state_t_chirho wr_state_chirho;
    logic [31:0] wr_addr_chirho;
    logic ocl_awready_chirho;
    logic ocl_wready_chirho;
    logic ocl_bvalid_chirho;
    logic [1:0] ocl_bresp_chirho;

    always_ff @(posedge clk_main_a0) begin
        if (!rst_main_n_sync_chirho) begin
            wr_state_chirho     <= WR_IDLE_CHIRHO;
            ocl_awready_chirho  <= 1'b0;
            ocl_wready_chirho   <= 1'b0;
            ocl_bvalid_chirho   <= 1'b0;
            ocl_bresp_chirho    <= 2'b00;
            ctrl_enable_chirho  <= 1'b0;
            ctrl_reset_chirho   <= 1'b0;
            ctrl_hbm_mode_chirho <= 1'b0;
            cmd_reg_chirho      <= 70'b0;
        end else begin
            case (wr_state_chirho)
                WR_IDLE_CHIRHO: begin
                    ocl_awready_chirho <= 1'b1;
                    ocl_bvalid_chirho  <= 1'b0;
                    if (ocl_cl_awvalid && ocl_awready_chirho) begin
                        wr_addr_chirho     <= ocl_cl_awaddr;
                        ocl_awready_chirho <= 1'b0;
                        ocl_wready_chirho  <= 1'b1;
                        wr_state_chirho    <= WR_DATA_CHIRHO;
                    end
                end

                WR_DATA_CHIRHO: begin
                    if (ocl_cl_wvalid && ocl_wready_chirho) begin
                        ocl_wready_chirho <= 1'b0;
                        case (wr_addr_chirho[7:2])
                            6'h01: begin // 0x004: CONTROL
                                ctrl_enable_chirho   <= ocl_cl_wdata[0];
                                ctrl_reset_chirho    <= ocl_cl_wdata[1];
                                ctrl_hbm_mode_chirho <= ocl_cl_wdata[2];
                            end
                            6'h04: cmd_reg_chirho[31:0]  <= ocl_cl_wdata;
                            6'h05: cmd_reg_chirho[63:32] <= ocl_cl_wdata;
                            6'h06: cmd_reg_chirho[69:64] <= ocl_cl_wdata[5:0];
                        endcase
                        ocl_bvalid_chirho <= 1'b1;
                        ocl_bresp_chirho  <= 2'b00;
                        wr_state_chirho   <= WR_RESP_CHIRHO;
                    end
                end

                WR_RESP_CHIRHO: begin
                    if (ocl_cl_bready) begin
                        ocl_bvalid_chirho <= 1'b0;
                        wr_state_chirho   <= WR_IDLE_CHIRHO;
                    end
                end
            endcase
        end
    end

    // ========================================================================
    // OCL AXI-Lite Read Logic (F2 port names)
    // ========================================================================

    typedef enum logic [1:0] {
        RD_IDLE_CHIRHO,
        RD_DECODE_CHIRHO,  // Pipeline stage for timing closure
        RD_DATA_CHIRHO
    } rd_state_t_chirho;

    rd_state_t_chirho rd_state_chirho;
    logic ocl_arready_chirho;
    logic ocl_rvalid_chirho;
    logic [31:0] ocl_rdata_chirho;
    logic [1:0] ocl_rresp_chirho;
    logic [5:0] rd_addr_chirho;  // Registered address for pipeline

    always_ff @(posedge clk_main_a0) begin
        if (!rst_main_n_sync_chirho) begin
            rd_state_chirho    <= RD_IDLE_CHIRHO;
            ocl_arready_chirho <= 1'b0;
            ocl_rvalid_chirho  <= 1'b0;
            ocl_rdata_chirho   <= 32'b0;
            ocl_rresp_chirho   <= 2'b00;
            rd_addr_chirho     <= 6'b0;
        end else begin
            case (rd_state_chirho)
                RD_IDLE_CHIRHO: begin
                    ocl_arready_chirho <= 1'b1;
                    ocl_rvalid_chirho  <= 1'b0;
                    if (ocl_cl_arvalid && ocl_arready_chirho) begin
                        ocl_arready_chirho <= 1'b0;
                        rd_addr_chirho     <= ocl_cl_araddr[7:2];  // Pipeline: capture address
                        rd_state_chirho    <= RD_DECODE_CHIRHO;
                    end
                end

                // Pipeline stage: decode address and prepare data (timing closure)
                RD_DECODE_CHIRHO: begin
                    case (rd_addr_chirho)
                        6'h00: ocl_rdata_chirho <= `MINIKANREN_VERSION_CHIRHO;
                        6'h01: ocl_rdata_chirho <= {29'b0, ctrl_hbm_mode_chirho, ctrl_reset_chirho, ctrl_enable_chirho};
                        6'h02: ocl_rdata_chirho <= {29'b0, hbm_ready_chirho, 1'b1, 1'b1}; // STATUS with HBM ready
                        6'h04: ocl_rdata_chirho <= cmd_reg_chirho[31:0];
                        6'h05: ocl_rdata_chirho <= cmd_reg_chirho[63:32];
                        6'h06: ocl_rdata_chirho <= {26'b0, cmd_reg_chirho[69:64]};
                        // Response registers
                        6'h08: ocl_rdata_chirho <= resp_wire_chirho[31:0];
                        6'h09: ocl_rdata_chirho <= resp_wire_chirho[63:32];
                        6'h0A: ocl_rdata_chirho <= resp_wire_chirho[95:64];
                        6'h0B: ocl_rdata_chirho <= resp_wire_chirho[127:96];
                        6'h0C: ocl_rdata_chirho <= resp_wire_chirho[159:128];
                        6'h0D: ocl_rdata_chirho <= resp_wire_chirho[191:160];
                        6'h0E: ocl_rdata_chirho <= resp_wire_chirho[223:192];
                        6'h0F: ocl_rdata_chirho <= resp_wire_chirho[255:224];
                        6'h10: ocl_rdata_chirho <= resp_wire_chirho[287:256];
                        6'h11: ocl_rdata_chirho <= resp_wire_chirho[319:288];
                        6'h12: ocl_rdata_chirho <= resp_wire_chirho[351:320];
                        6'h13: ocl_rdata_chirho <= resp_wire_chirho[383:352];
                        6'h14: ocl_rdata_chirho <= resp_wire_chirho[415:384];
                        6'h15: ocl_rdata_chirho <= resp_wire_chirho[447:416];
                        6'h16: ocl_rdata_chirho <= resp_wire_chirho[479:448];
                        6'h17: ocl_rdata_chirho <= resp_wire_chirho[511:480];
                        6'h18: ocl_rdata_chirho <= {30'b0, resp_wire_chirho[513:512]};
                        default: ocl_rdata_chirho <= 32'hDEADBEEF;
                    endcase
                    ocl_rvalid_chirho <= 1'b1;
                    ocl_rresp_chirho  <= 2'b00;
                    rd_state_chirho   <= RD_DATA_CHIRHO;
                end

                RD_DATA_CHIRHO: begin
                    if (ocl_cl_rready) begin
                        ocl_rvalid_chirho <= 1'b0;
                        rd_state_chirho   <= RD_IDLE_CHIRHO;
                    end
                end
            endcase
        end
    end

    // ========================================================================
    // OCL Output Assignments (F2 naming: cl_ocl_*)
    // ========================================================================

    assign cl_ocl_awready = ocl_awready_chirho;
    assign cl_ocl_wready  = ocl_wready_chirho;
    assign cl_ocl_bvalid  = ocl_bvalid_chirho;
    assign cl_ocl_bresp   = ocl_bresp_chirho;
    assign cl_ocl_arready = ocl_arready_chirho;
    assign cl_ocl_rvalid  = ocl_rvalid_chirho;
    assign cl_ocl_rdata   = ocl_rdata_chirho;
    assign cl_ocl_rresp   = ocl_rresp_chirho;

    // ========================================================================
    // Virtual LED / DIP Switch
    // ========================================================================

    assign cl_sh_status_vled[0]    = ctrl_enable_chirho;
    assign cl_sh_status_vled[1]    = engine_rst_chirho;
    assign cl_sh_status_vled[2]    = hbm_ready_chirho;
    assign cl_sh_status_vled[3]    = ctrl_hbm_mode_chirho;
    assign cl_sh_status_vled[7:4]  = cmd_reg_chirho[69:66];
    assign cl_sh_status_vled[15:8] = resp_wire_chirho[7:0];

    // ========================================================================
    // miniKanren Search Engine with HBM FSM
    // ========================================================================

if (EN_HBM) begin : HBM_ENGINE

    // HBM-backed search engine
    // Uses AXI4 to read/write variable domains in HBM

    // HBM FSM states
    typedef enum logic [2:0] {
        FSM_IDLE_CHIRHO,
        FSM_LOAD_VAR1_CHIRHO,
        FSM_LOAD_VAR2_CHIRHO,
        FSM_COMPUTE_CHIRHO,
        FSM_STORE_RESULT_CHIRHO,
        FSM_BATCH_NEXT_CHIRHO
    } hbm_fsm_state_t_chirho;

    hbm_fsm_state_t_chirho fsm_state_chirho;

    // Command parsing
    logic [15:0] var_id_1_chirho;
    logic [15:0] var_id_2_chirho;
    logic [3:0]  op_code_chirho;
    logic [15:0] batch_count_chirho;
    logic [15:0] batch_idx_chirho;

    // Domain storage (256-bit HBM words)
    logic [255:0] domain_1_chirho;
    logic [255:0] domain_2_chirho;
    logic [255:0] domain_result_chirho;

    // Result status
    logic op_done_chirho;
    logic op_valid_chirho;

    // AXI4 control signals
    logic axi_read_req_chirho;
    logic axi_write_req_chirho;
    logic [33:0] axi_addr_chirho;
    logic [255:0] axi_wdata_chirho;

    // Decode command register
    always_comb begin
        op_code_chirho     = cmd_reg_chirho[3:0];
        var_id_1_chirho    = cmd_reg_chirho[19:4];
        var_id_2_chirho    = cmd_reg_chirho[35:20];
        batch_count_chirho = cmd_reg_chirho[51:36];
    end

    // HBM address calculation: var_id * 64 bytes (for hierarchical domain)
    function automatic [33:0] var_to_hbm_addr_chirho(input [15:0] var_id_chirho);
        // Base address for variable domains (from cl_minikanren_chirho_defines.vh)
        return `HBM_VAR_DOMAINS_BASE_CHIRHO + ({18'b0, var_id_chirho} << 6);
    endfunction

    // HBM FSM
    always_ff @(posedge clk_main_a0) begin
        if (engine_rst_chirho) begin
            fsm_state_chirho      <= FSM_IDLE_CHIRHO;
            domain_1_chirho       <= 256'b0;
            domain_2_chirho       <= 256'b0;
            domain_result_chirho  <= 256'b0;
            op_done_chirho        <= 1'b0;
            op_valid_chirho       <= 1'b0;
            batch_idx_chirho      <= 16'b0;
            axi_read_req_chirho   <= 1'b0;
            axi_write_req_chirho  <= 1'b0;
            axi_addr_chirho       <= 34'b0;
            axi_wdata_chirho      <= 256'b0;
        end else begin
            case (fsm_state_chirho)
                FSM_IDLE_CHIRHO: begin
                    op_done_chirho  <= 1'b0;
                    op_valid_chirho <= 1'b0;
                    if (ctrl_enable_chirho && ctrl_hbm_mode_chirho && hbm_ready_chirho) begin
                        // Start operation
                        batch_idx_chirho    <= 16'b0;
                        axi_addr_chirho     <= var_to_hbm_addr_chirho(var_id_1_chirho);
                        axi_read_req_chirho <= 1'b1;
                        fsm_state_chirho    <= FSM_LOAD_VAR1_CHIRHO;
                    end
                end

                FSM_LOAD_VAR1_CHIRHO: begin
                    // Wait for AXI read response
                    if (hbm_axi4_bus_chirho.rvalid && hbm_axi4_bus_chirho.rready) begin
                        domain_1_chirho     <= hbm_axi4_bus_chirho.rdata;
                        axi_read_req_chirho <= 1'b0;
                        // Start loading var2
                        axi_addr_chirho     <= var_to_hbm_addr_chirho(var_id_2_chirho);
                        axi_read_req_chirho <= 1'b1;
                        fsm_state_chirho    <= FSM_LOAD_VAR2_CHIRHO;
                    end
                end

                FSM_LOAD_VAR2_CHIRHO: begin
                    if (hbm_axi4_bus_chirho.rvalid && hbm_axi4_bus_chirho.rready) begin
                        domain_2_chirho     <= hbm_axi4_bus_chirho.rdata;
                        axi_read_req_chirho <= 1'b0;
                        fsm_state_chirho    <= FSM_COMPUTE_CHIRHO;
                    end
                end

                FSM_COMPUTE_CHIRHO: begin
                    // Domain intersection (unification)
                    case (op_code_chirho)
                        4'h0: begin // INTERSECT (unify)
                            domain_result_chirho <= domain_1_chirho & domain_2_chirho;
                            op_valid_chirho <= (domain_1_chirho & domain_2_chirho) != 256'b0;
                        end
                        4'h1: begin // UNION (disjunction)
                            domain_result_chirho <= domain_1_chirho | domain_2_chirho;
                            op_valid_chirho <= 1'b1;
                        end
                        4'h2: begin // COMPLEMENT
                            domain_result_chirho <= ~domain_1_chirho;
                            op_valid_chirho <= 1'b1;
                        end
                        4'h3: begin // IS_GROUND (single value)
                            op_valid_chirho <= (domain_1_chirho != 256'b0) &&
                                              ((domain_1_chirho & (domain_1_chirho - 1)) == 256'b0);
                            domain_result_chirho <= domain_1_chirho;
                        end
                        default: begin
                            domain_result_chirho <= 256'b0;
                            op_valid_chirho <= 1'b0;
                        end
                    endcase
                    fsm_state_chirho <= FSM_STORE_RESULT_CHIRHO;
                end

                FSM_STORE_RESULT_CHIRHO: begin
                    // Write result back to var1's domain
                    axi_addr_chirho      <= var_to_hbm_addr_chirho(var_id_1_chirho);
                    axi_wdata_chirho     <= domain_result_chirho;
                    axi_write_req_chirho <= 1'b1;

                    if (hbm_axi4_bus_chirho.bvalid && hbm_axi4_bus_chirho.bready) begin
                        axi_write_req_chirho <= 1'b0;
                        fsm_state_chirho     <= FSM_BATCH_NEXT_CHIRHO;
                    end
                end

                FSM_BATCH_NEXT_CHIRHO: begin
                    batch_idx_chirho <= batch_idx_chirho + 1;
                    if (batch_idx_chirho >= batch_count_chirho - 1) begin
                        op_done_chirho   <= 1'b1;
                        fsm_state_chirho <= FSM_IDLE_CHIRHO;
                    end else begin
                        // Continue to next batch item (vars increment)
                        // TODO: Load batch table from HBM
                        fsm_state_chirho <= FSM_LOAD_VAR1_CHIRHO;
                    end
                end
            endcase
        end
    end

    // AXI4 read channel
    assign hbm_axi4_bus_chirho.arid    = 6'b0;
    assign hbm_axi4_bus_chirho.araddr  = axi_addr_chirho;
    assign hbm_axi4_bus_chirho.arlen   = 8'b0;  // Single beat
    assign hbm_axi4_bus_chirho.arsize  = 3'b101; // 32 bytes (256 bits)
    assign hbm_axi4_bus_chirho.arburst = 2'b01;  // INCR
    assign hbm_axi4_bus_chirho.arvalid = axi_read_req_chirho;
    assign hbm_axi4_bus_chirho.rready  = 1'b1;

    // AXI4 write channel
    assign hbm_axi4_bus_chirho.awid    = 6'b0;
    assign hbm_axi4_bus_chirho.awaddr  = axi_addr_chirho;
    assign hbm_axi4_bus_chirho.awlen   = 8'b0;  // Single beat
    assign hbm_axi4_bus_chirho.awsize  = 3'b101; // 32 bytes
    assign hbm_axi4_bus_chirho.awburst = 2'b01;  // INCR
    assign hbm_axi4_bus_chirho.awvalid = axi_write_req_chirho;
    assign hbm_axi4_bus_chirho.wdata   = axi_wdata_chirho;
    assign hbm_axi4_bus_chirho.wstrb   = 32'hFFFFFFFF;
    assign hbm_axi4_bus_chirho.wlast   = 1'b1;
    assign hbm_axi4_bus_chirho.wvalid  = axi_write_req_chirho;
    assign hbm_axi4_bus_chirho.bready  = 1'b1;

    // Response register mapping
    assign resp_wire_chirho = {
        446'b0,                      // Padding
        op_done_chirho,              // bit 67
        op_valid_chirho,             // bit 66
        2'b0,                        // bits 65:64
        domain_result_chirho[63:0]   // bits 63:0 (first 64 bits of result)
    };

end : HBM_ENGINE
else begin : LEGACY_ENGINE

    // Legacy register-based engine (no HBM)
    // NOTE: Port names .clk and .rst are Clash-generated standard clock/reset ports
    // and retain their original Clash names per AGENTS.md convention. Custom ports
    // (enChirho, cmdChirho, respChirho) use the Chirho suffix per project naming.
    searchEngineChirho u_engine_chirho (
        .clk        (clk_engine_chirho),
        .rst        (engine_rst_chirho),
        .enChirho   (ctrl_enable_chirho),
        .cmdChirho  (cmd_reg_chirho),
        .respChirho (resp_wire_chirho)
    );

end : LEGACY_ENGINE

endmodule

`default_nettype wire
