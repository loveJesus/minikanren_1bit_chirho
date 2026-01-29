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
    logic [2:0] ctrl_hier_mode_chirho;  // Hierarchical domain mode (see defines.vh)
    logic [69:0] cmd_reg_chirho;
    logic [513:0] resp_wire_chirho;

    // HBM interface signals
    logic hbm_ready_chirho;
    logic [7:0] hbm_stat_int_chirho;

    // Neurosymbolic training signals
    logic train_enable_chirho;
    logic train_reset_chirho;
    logic [127:0] train_cmd_chirho;
    logic [63:0] train_resp_chirho;

    // Probabilistic inference signals
    logic infer_prob_mode_chirho;  // 0=Boolean, 1=Probabilistic
    logic [31:0] soft_and_a_chirho;
    logic [31:0] soft_and_b_chirho;
    logic [31:0] soft_and_result_chirho;
    logic [1087:0] prob_domain_a_chirho;
    logic [1087:0] prob_domain_b_chirho;
    logic [1087:0] prob_domain_result_chirho;

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
            ctrl_hier_mode_chirho <= `HIER_MODE_FLAT256_CHIRHO;
            cmd_reg_chirho      <= 70'b0;
            train_enable_chirho <= 1'b0;
            train_reset_chirho  <= 1'b0;
            train_cmd_chirho    <= 128'b0;
            infer_prob_mode_chirho <= `INFER_MODE_BOOL_CHIRHO;
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
                            6'h10: ctrl_hier_mode_chirho <= ocl_cl_wdata[2:0]; // REG_HIER_MODE_CHIRHO (0x40)
                            // Neurosymbolic training registers (0x50-0x60)
                            6'h14: begin // REG_TRAIN_MODE_CHIRHO (0x50)
                                train_enable_chirho <= ocl_cl_wdata[0];
                                train_reset_chirho  <= ocl_cl_wdata[1];
                            end
                            6'h15: train_cmd_chirho[31:0]   <= ocl_cl_wdata; // REG_TRAIN_CMD_LO (0x54)
                            6'h16: train_cmd_chirho[63:32]  <= ocl_cl_wdata; // REG_TRAIN_CMD_MID (0x58)
                            6'h17: train_cmd_chirho[95:64]  <= ocl_cl_wdata; // REG_TRAIN_CMD_HI (0x5C)
                            6'h18: train_cmd_chirho[127:96] <= ocl_cl_wdata; // REG_TRAIN_CMD_TOP (0x60)
                            6'h1C: infer_prob_mode_chirho <= ocl_cl_wdata[0]; // REG_INFER_MODE (0x70)
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
                        6'h10: ocl_rdata_chirho <= {29'b0, ctrl_hier_mode_chirho}; // REG_HIER_MODE_CHIRHO (0x40)
                        // Neurosymbolic training registers (0x50-0x68)
                        6'h14: ocl_rdata_chirho <= {30'b0, train_reset_chirho, train_enable_chirho}; // REG_TRAIN_MODE
                        6'h15: ocl_rdata_chirho <= train_cmd_chirho[31:0];   // REG_TRAIN_CMD_LO
                        6'h16: ocl_rdata_chirho <= train_cmd_chirho[63:32];  // REG_TRAIN_CMD_MID
                        6'h17: ocl_rdata_chirho <= train_cmd_chirho[95:64];  // REG_TRAIN_CMD_HI
                        6'h18: ocl_rdata_chirho <= train_cmd_chirho[127:96]; // REG_TRAIN_CMD_TOP
                        6'h19: ocl_rdata_chirho <= train_resp_chirho[31:0];  // REG_TRAIN_RESP_LO (0x64)
                        6'h1A: ocl_rdata_chirho <= train_resp_chirho[63:32]; // REG_TRAIN_RESP_HI (0x68)
                        6'h1C: ocl_rdata_chirho <= {31'b0, infer_prob_mode_chirho}; // REG_INFER_MODE (0x70)
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

    // ========================================================================
    // HBM-backed search engine with Hierarchical Domain Support
    // Supports: 256-bit flat, 256² (65K), 512² (262K), 256³ (16.7M), 512³ (134M)
    // ========================================================================

    // HBM FSM states (extended for multi-beat hierarchical and streaming reads)
    typedef enum logic [4:0] {
        FSM_IDLE_CHIRHO,
        FSM_LOAD_VAR1_CHIRHO,           // Single beat (flat 256-bit)
        FSM_LOAD_VAR1_BURST_CHIRHO,     // Multi-beat for 2-level hierarchy
        FSM_LOAD_VAR2_CHIRHO,
        FSM_LOAD_VAR2_BURST_CHIRHO,
        FSM_COMPUTE_CHIRHO,             // Flat intersection
        FSM_COMPUTE_HIER_CHIRHO,        // 2-level hierarchical intersection
        FSM_STORE_RESULT_CHIRHO,
        FSM_STORE_BURST_CHIRHO,
        // Streaming states for 3-level hierarchies (256³, 512³)
        FSM_LOAD_SUMMARIES_CHIRHO,      // Load level0 + level1 summaries
        FSM_STREAM_LOAD_L2_V1_CHIRHO,   // Stream level2 blocks for var1
        FSM_STREAM_LOAD_L2_V2_CHIRHO,   // Stream level2 blocks for var2
        FSM_STREAM_COMPUTE_CHIRHO,      // Compute current level2 block
        FSM_STREAM_STORE_CHIRHO,        // Store current level2 result
        FSM_STREAM_NEXT_BLOCK_CHIRHO,   // Advance to next level2 block
        FSM_BATCH_NEXT_CHIRHO
    } hbm_fsm_state_t_chirho;

    hbm_fsm_state_t_chirho fsm_state_chirho;

    // Command parsing
    logic [15:0] var_id_1_chirho;
    logic [15:0] var_id_2_chirho;
    logic [3:0]  op_code_chirho;
    logic [15:0] batch_count_chirho;
    logic [15:0] batch_idx_chirho;

    // Flat domain storage (256-bit for flat mode)
    logic [255:0] domain_1_flat_chirho;
    logic [255:0] domain_2_flat_chirho;
    logic [255:0] domain_result_flat_chirho;

    // ========================================================================
    // Hierarchical Domain Buffers (multiple sizes for benchmarking)
    // ========================================================================

    // 256² = 65K values (8KB per domain) - FULLY BUFFERED
    // Level 0 (summary): 256 bits indicating which level-1 blocks are non-zero
    // Level 1 (data): 256 × 256 bits = 8KB
    logic [255:0] hier_65k_1_level0_chirho;
    logic [255:0] hier_65k_1_level1_chirho [0:255];  // BRAM: 256 × 256-bit words
    logic [255:0] hier_65k_2_level0_chirho;
    logic [255:0] hier_65k_2_level1_chirho [0:255];
    logic [255:0] hier_65k_result_level0_chirho;
    logic [255:0] hier_65k_result_level1_chirho [0:255];

    // 512² = 262K values (33KB per domain) - FULLY BUFFERED
    // Level 0 (summary): 512 bits indicating which level-1 blocks are non-zero
    // Level 1 (data): 512 × 512 bits = 33KB
    logic [511:0] hier_262k_1_level0_chirho;
    logic [511:0] hier_262k_1_level1_chirho [0:511];  // BRAM: 512 × 512-bit words
    logic [511:0] hier_262k_2_level0_chirho;
    logic [511:0] hier_262k_2_level1_chirho [0:511];
    logic [511:0] hier_262k_result_level0_chirho;
    logic [511:0] hier_262k_result_level1_chirho [0:511];

    // 256³ = 16.7M values (2MB per domain) - STREAMING MODE
    // Too large to fully buffer; process block-by-block
    // Level 0: 256 bits (which level1 blocks exist)
    // Level 1: 256 × 256 bits = 8KB (which level2 blocks exist)
    // Level 2: 256 × 256 × 256 bits = 2MB (actual data, streamed)
    logic [255:0] hier_16m_1_level0_chirho;
    logic [255:0] hier_16m_1_level1_chirho [0:255];
    logic [255:0] hier_16m_2_level0_chirho;
    logic [255:0] hier_16m_2_level1_chirho [0:255];
    logic [255:0] hier_16m_result_level0_chirho;
    logic [255:0] hier_16m_result_level1_chirho [0:255];
    // Level 2 streaming buffers (one block at a time)
    logic [255:0] hier_16m_1_level2_block_chirho [0:255];
    logic [255:0] hier_16m_2_level2_block_chirho [0:255];
    logic [255:0] hier_16m_result_level2_block_chirho [0:255];
    logic [7:0] hier_16m_level1_idx_chirho;  // Current level1 block being processed
    logic [7:0] hier_16m_level2_idx_chirho;  // Current level2 block within level1

    // 512³ = 134M values (16MB per domain) - STREAMING MODE
    // Level 0: 512 bits, Level 1: 512 × 512 bits = 33KB, Level 2: streamed
    logic [511:0] hier_134m_1_level0_chirho;
    logic [511:0] hier_134m_1_level1_chirho [0:511];
    logic [511:0] hier_134m_2_level0_chirho;
    logic [511:0] hier_134m_2_level1_chirho [0:511];
    logic [511:0] hier_134m_result_level0_chirho;
    logic [511:0] hier_134m_result_level1_chirho [0:511];
    // Level 2 streaming buffers
    logic [511:0] hier_134m_1_level2_block_chirho [0:511];
    logic [511:0] hier_134m_2_level2_block_chirho [0:511];
    logic [511:0] hier_134m_result_level2_block_chirho [0:511];
    logic [8:0] hier_134m_level1_idx_chirho;
    logic [8:0] hier_134m_level2_idx_chirho;

    // Beat counter for burst transfers
    logic [19:0] beat_counter_chirho;  // Extended to handle 512³ (up to 525K beats)
    logic [19:0] beats_required_chirho;

    // Result status
    logic op_done_chirho;
    logic op_valid_chirho;
    logic hier_domain_empty_chirho;

    // AXI4 control signals
    logic axi_read_req_chirho;
    logic axi_write_req_chirho;
    logic [33:0] axi_addr_chirho;
    logic [255:0] axi_wdata_chirho;
    logic [7:0] axi_burst_len_chirho;  // For burst transfers

    // Decode command register
    always_comb begin
        op_code_chirho     = cmd_reg_chirho[3:0];
        var_id_1_chirho    = cmd_reg_chirho[19:4];
        var_id_2_chirho    = cmd_reg_chirho[35:20];
        batch_count_chirho = cmd_reg_chirho[51:36];
    end

    // HBM address calculation based on hierarchy mode
    function automatic [33:0] var_to_hbm_addr_chirho(input [15:0] var_id_chirho);
        case (ctrl_hier_mode_chirho)
            `HIER_MODE_FLAT256_CHIRHO:    return `HBM_VAR_DOMAINS_BASE_CHIRHO + ({18'b0, var_id_chirho} << 5);      // 32B per var
            `HIER_MODE_HIER_65K_CHIRHO:   return `HBM_VAR_DOMAINS_BASE_CHIRHO + ({18'b0, var_id_chirho} * 20'd8224);   // ~8KB per var
            `HIER_MODE_HIER_262K_CHIRHO:  return `HBM_VAR_DOMAINS_BASE_CHIRHO + ({18'b0, var_id_chirho} * 20'd33024);  // ~33KB per var
            `HIER_MODE_HIER_16M_CHIRHO:   return `HBM_VAR_DOMAINS_BASE_CHIRHO + ({18'b0, var_id_chirho} * 24'd2097408); // ~2MB per var
            `HIER_MODE_HIER_134M_CHIRHO:  return `HBM_VAR_DOMAINS_BASE_CHIRHO + ({18'b0, var_id_chirho} * 28'd16810496); // ~16MB per var
            default:                      return `HBM_VAR_DOMAINS_BASE_CHIRHO + ({18'b0, var_id_chirho} << 5);
        endcase
    endfunction

    // Beats required for initial load (summaries only for 3-level hierarchies)
    function automatic [19:0] get_beats_required_chirho();
        case (ctrl_hier_mode_chirho)
            `HIER_MODE_FLAT256_CHIRHO:    return 20'd1;
            `HIER_MODE_HIER_65K_CHIRHO:   return 20'd257;    // 1 level0 + 256 level1
            `HIER_MODE_HIER_262K_CHIRHO:  return 20'd513;    // 2 level0 + 512 level1 (512-bit words)
            `HIER_MODE_HIER_16M_CHIRHO:   return 20'd257;    // 1 level0 + 256 level1 (summaries only, level2 streamed)
            `HIER_MODE_HIER_134M_CHIRHO:  return 20'd513;    // 2 level0 + 512 level1 (summaries only, level2 streamed)
            default:                      return 20'd1;
        endcase
    endfunction

    // Check if this hierarchy mode uses streaming (3-level)
    function automatic logic is_streaming_mode_chirho();
        return (ctrl_hier_mode_chirho == `HIER_MODE_HIER_16M_CHIRHO) ||
               (ctrl_hier_mode_chirho == `HIER_MODE_HIER_134M_CHIRHO);
    endfunction

    // Get number of level1 blocks for current mode
    function automatic [9:0] get_level1_count_chirho();
        case (ctrl_hier_mode_chirho)
            `HIER_MODE_HIER_65K_CHIRHO:   return 10'd256;
            `HIER_MODE_HIER_262K_CHIRHO:  return 10'd512;
            `HIER_MODE_HIER_16M_CHIRHO:   return 10'd256;
            `HIER_MODE_HIER_134M_CHIRHO:  return 10'd512;
            default:                      return 10'd1;
        endcase
    endfunction

    // HBM FSM with hierarchical support
    always_ff @(posedge clk_main_a0) begin
        if (engine_rst_chirho) begin
            fsm_state_chirho          <= FSM_IDLE_CHIRHO;
            domain_1_flat_chirho      <= 256'b0;
            domain_2_flat_chirho      <= 256'b0;
            domain_result_flat_chirho <= 256'b0;
            hier_65k_1_level0_chirho  <= 256'b0;
            hier_65k_2_level0_chirho  <= 256'b0;
            hier_65k_result_level0_chirho <= 256'b0;
            hier_262k_result_level0_chirho <= 512'b0;
            hier_16m_result_level0_chirho <= 256'b0;
            hier_134m_result_level0_chirho <= 512'b0;
            hier_16m_level1_idx_chirho <= 8'b0;
            hier_16m_level2_idx_chirho <= 8'b0;
            hier_134m_level1_idx_chirho <= 9'b0;
            hier_134m_level2_idx_chirho <= 9'b0;
            op_done_chirho            <= 1'b0;
            op_valid_chirho           <= 1'b0;
            batch_idx_chirho          <= 16'b0;
            beat_counter_chirho       <= 20'b0;
            beats_required_chirho     <= 20'b1;
            axi_read_req_chirho       <= 1'b0;
            axi_write_req_chirho      <= 1'b0;
            axi_addr_chirho           <= 34'b0;
            axi_wdata_chirho          <= 256'b0;
            axi_burst_len_chirho      <= 8'b0;
            hier_domain_empty_chirho  <= 1'b0;
        end else begin
            case (fsm_state_chirho)
                FSM_IDLE_CHIRHO: begin
                    op_done_chirho  <= 1'b0;
                    op_valid_chirho <= 1'b0;
                    if (ctrl_enable_chirho && ctrl_hbm_mode_chirho && hbm_ready_chirho) begin
                        batch_idx_chirho      <= 16'b0;
                        beat_counter_chirho   <= 20'b0;
                        beats_required_chirho <= get_beats_required_chirho();
                        axi_addr_chirho       <= var_to_hbm_addr_chirho(var_id_1_chirho);
                        axi_read_req_chirho   <= 1'b1;
                        axi_burst_len_chirho  <= (ctrl_hier_mode_chirho == `HIER_MODE_FLAT256_CHIRHO) ? 8'b0 : 8'hFF;
                        // Reset streaming indices
                        hier_16m_level1_idx_chirho <= 8'b0;
                        hier_16m_level2_idx_chirho <= 8'b0;
                        hier_134m_level1_idx_chirho <= 9'b0;
                        hier_134m_level2_idx_chirho <= 9'b0;
                        // Branch based on hierarchy mode
                        case (ctrl_hier_mode_chirho)
                            `HIER_MODE_FLAT256_CHIRHO:
                                fsm_state_chirho <= FSM_LOAD_VAR1_CHIRHO;
                            `HIER_MODE_HIER_65K_CHIRHO,
                            `HIER_MODE_HIER_262K_CHIRHO:
                                fsm_state_chirho <= FSM_LOAD_VAR1_BURST_CHIRHO;
                            `HIER_MODE_HIER_16M_CHIRHO,
                            `HIER_MODE_HIER_134M_CHIRHO:
                                fsm_state_chirho <= FSM_LOAD_SUMMARIES_CHIRHO;  // 3-level streaming
                            default:
                                fsm_state_chirho <= FSM_LOAD_VAR1_CHIRHO;
                        endcase
                    end
                end

                // Flat 256-bit mode (single beat)
                FSM_LOAD_VAR1_CHIRHO: begin
                    if (hbm_axi4_bus_chirho.rvalid && hbm_axi4_bus_chirho.rready) begin
                        domain_1_flat_chirho <= hbm_axi4_bus_chirho.rdata;
                        axi_read_req_chirho  <= 1'b0;
                        axi_addr_chirho      <= var_to_hbm_addr_chirho(var_id_2_chirho);
                        axi_read_req_chirho  <= 1'b1;
                        fsm_state_chirho     <= FSM_LOAD_VAR2_CHIRHO;
                    end
                end

                FSM_LOAD_VAR2_CHIRHO: begin
                    if (hbm_axi4_bus_chirho.rvalid && hbm_axi4_bus_chirho.rready) begin
                        domain_2_flat_chirho <= hbm_axi4_bus_chirho.rdata;
                        axi_read_req_chirho  <= 1'b0;
                        fsm_state_chirho     <= FSM_COMPUTE_CHIRHO;
                    end
                end

                // Hierarchical 256² mode (multi-beat burst)
                FSM_LOAD_VAR1_BURST_CHIRHO: begin
                    if (hbm_axi4_bus_chirho.rvalid && hbm_axi4_bus_chirho.rready) begin
                        if (beat_counter_chirho == 16'b0) begin
                            // First beat is level0 summary
                            hier_65k_1_level0_chirho <= hbm_axi4_bus_chirho.rdata;
                        end else if (beat_counter_chirho <= 16'd256) begin
                            // Beats 1-256 are level1 data
                            hier_65k_1_level1_chirho[beat_counter_chirho - 1] <= hbm_axi4_bus_chirho.rdata;
                        end

                        beat_counter_chirho <= beat_counter_chirho + 1;

                        if (beat_counter_chirho >= beats_required_chirho - 1) begin
                            // Done loading var1, start var2
                            axi_read_req_chirho <= 1'b0;
                            beat_counter_chirho <= 16'b0;
                            axi_addr_chirho     <= var_to_hbm_addr_chirho(var_id_2_chirho);
                            axi_read_req_chirho <= 1'b1;
                            fsm_state_chirho    <= FSM_LOAD_VAR2_BURST_CHIRHO;
                        end
                    end
                end

                FSM_LOAD_VAR2_BURST_CHIRHO: begin
                    if (hbm_axi4_bus_chirho.rvalid && hbm_axi4_bus_chirho.rready) begin
                        if (beat_counter_chirho == 16'b0) begin
                            hier_65k_2_level0_chirho <= hbm_axi4_bus_chirho.rdata;
                        end else if (beat_counter_chirho <= 16'd256) begin
                            hier_65k_2_level1_chirho[beat_counter_chirho - 1] <= hbm_axi4_bus_chirho.rdata;
                        end

                        beat_counter_chirho <= beat_counter_chirho + 1;

                        if (beat_counter_chirho >= beats_required_chirho - 1) begin
                            axi_read_req_chirho <= 1'b0;
                            beat_counter_chirho <= 16'b0;
                            fsm_state_chirho    <= FSM_COMPUTE_HIER_CHIRHO;
                        end
                    end
                end

                // Flat compute (single cycle)
                FSM_COMPUTE_CHIRHO: begin
                    case (op_code_chirho)
                        `OP_INTERSECT_CHIRHO: begin
                            domain_result_flat_chirho <= domain_1_flat_chirho & domain_2_flat_chirho;
                            op_valid_chirho <= (domain_1_flat_chirho & domain_2_flat_chirho) != 256'b0;
                        end
                        `OP_UNION_CHIRHO: begin
                            domain_result_flat_chirho <= domain_1_flat_chirho | domain_2_flat_chirho;
                            op_valid_chirho <= 1'b1;
                        end
                        `OP_COMPLEMENT_CHIRHO: begin
                            domain_result_flat_chirho <= ~domain_1_flat_chirho;
                            op_valid_chirho <= 1'b1;
                        end
                        `OP_IS_GROUND_CHIRHO: begin
                            op_valid_chirho <= (domain_1_flat_chirho != 256'b0) &&
                                              ((domain_1_flat_chirho & (domain_1_flat_chirho - 1)) == 256'b0);
                            domain_result_flat_chirho <= domain_1_flat_chirho;
                        end
                        default: begin
                            domain_result_flat_chirho <= 256'b0;
                            op_valid_chirho <= 1'b0;
                        end
                    endcase
                    fsm_state_chirho <= FSM_STORE_RESULT_CHIRHO;
                end

                // Hierarchical compute (level0 intersection determines which level1 to process)
                FSM_COMPUTE_HIER_CHIRHO: begin
                    // Intersect level0 summaries first
                    hier_65k_result_level0_chirho <= hier_65k_1_level0_chirho & hier_65k_2_level0_chirho;

                    // Check if result is non-empty
                    hier_domain_empty_chirho <= (hier_65k_1_level0_chirho & hier_65k_2_level0_chirho) == 256'b0;
                    op_valid_chirho <= (hier_65k_1_level0_chirho & hier_65k_2_level0_chirho) != 256'b0;

                    // Level1 intersections happen in parallel (combinatorially wired below)
                    beat_counter_chirho <= 16'b0;
                    fsm_state_chirho    <= FSM_STORE_BURST_CHIRHO;
                end

                FSM_STORE_RESULT_CHIRHO: begin
                    // Flat mode: single beat write
                    axi_addr_chirho      <= var_to_hbm_addr_chirho(var_id_1_chirho);
                    axi_wdata_chirho     <= domain_result_flat_chirho;
                    axi_write_req_chirho <= 1'b1;

                    if (hbm_axi4_bus_chirho.bvalid && hbm_axi4_bus_chirho.bready) begin
                        axi_write_req_chirho <= 1'b0;
                        fsm_state_chirho     <= FSM_BATCH_NEXT_CHIRHO;
                    end
                end

                FSM_STORE_BURST_CHIRHO: begin
                    // Hierarchical mode: multi-beat write
                    axi_addr_chirho <= var_to_hbm_addr_chirho(var_id_1_chirho) + ({18'b0, beat_counter_chirho} << 5);

                    if (beat_counter_chirho == 16'b0) begin
                        axi_wdata_chirho <= hier_65k_result_level0_chirho;
                    end else if (beat_counter_chirho <= 16'd256) begin
                        axi_wdata_chirho <= hier_65k_result_level1_chirho[beat_counter_chirho - 1];
                    end

                    axi_write_req_chirho <= 1'b1;

                    if (hbm_axi4_bus_chirho.bvalid && hbm_axi4_bus_chirho.bready) begin
                        axi_write_req_chirho <= 1'b0;
                        beat_counter_chirho  <= beat_counter_chirho + 1;

                        if (beat_counter_chirho >= beats_required_chirho - 1) begin
                            fsm_state_chirho <= FSM_BATCH_NEXT_CHIRHO;
                        end
                    end
                end

                // ================================================================
                // 3-Level Streaming States (256³ and 512³)
                // ================================================================
                // For 16M and 134M hierarchies, we stream level2 blocks one at a time
                // to avoid requiring 2MB+ of on-chip buffers per domain.

                FSM_LOAD_SUMMARIES_CHIRHO: begin
                    // Load level0 and level1 summaries for both variables
                    // These fit in BRAM (8KB for 256³, 33KB for 512³)
                    if (hbm_axi4_bus_chirho.rvalid && hbm_axi4_bus_chirho.rready) begin
                        // Store based on which variable we're loading (use high bit of beat_counter)
                        if (beat_counter_chirho < beats_required_chirho) begin
                            // Loading var1 summaries
                            if (beat_counter_chirho == 20'd0) begin
                                hier_16m_1_level0_chirho <= hbm_axi4_bus_chirho.rdata;
                            end else begin
                                hier_16m_1_level1_chirho[beat_counter_chirho - 1] <= hbm_axi4_bus_chirho.rdata;
                            end
                        end else begin
                            // Loading var2 summaries
                            if (beat_counter_chirho == beats_required_chirho) begin
                                hier_16m_2_level0_chirho <= hbm_axi4_bus_chirho.rdata;
                            end else begin
                                hier_16m_2_level1_chirho[beat_counter_chirho - beats_required_chirho - 1] <= hbm_axi4_bus_chirho.rdata;
                            end
                        end

                        beat_counter_chirho <= beat_counter_chirho + 1;

                        // After loading var1 summaries, switch to var2
                        if (beat_counter_chirho == beats_required_chirho - 1) begin
                            axi_addr_chirho <= var_to_hbm_addr_chirho(var_id_2_chirho);
                        end

                        // After loading both, compute summary intersections and start streaming
                        if (beat_counter_chirho >= (beats_required_chirho * 2) - 1) begin
                            axi_read_req_chirho <= 1'b0;
                            // Intersect level0 and level1 summaries
                            hier_16m_result_level0_chirho <= hier_16m_1_level0_chirho & hier_16m_2_level0_chirho;
                            hier_16m_level1_idx_chirho <= 8'b0;
                            hier_16m_level2_idx_chirho <= 8'b0;
                            beat_counter_chirho <= 20'b0;
                            fsm_state_chirho <= FSM_STREAM_LOAD_L2_V1_CHIRHO;
                        end
                    end
                end

                FSM_STREAM_LOAD_L2_V1_CHIRHO: begin
                    // Stream level2 block for var1 (256 beats for 256³, 512 for 512³)
                    // Only load if both level1 summary bits are set (sparse optimization)
                    if (hier_16m_1_level1_chirho[hier_16m_level1_idx_chirho][hier_16m_level2_idx_chirho] &&
                        hier_16m_2_level1_chirho[hier_16m_level1_idx_chirho][hier_16m_level2_idx_chirho]) begin
                        // Calculate address: base + level1_offset + level2_offset
                        axi_addr_chirho <= var_to_hbm_addr_chirho(var_id_1_chirho) +
                                          ({12'b0, hier_16m_level1_idx_chirho} * 20'd8224) + // level1 block offset
                                          ({12'b0, hier_16m_level2_idx_chirho} << 5) +       // level2 beat offset
                                          20'd8224;  // Skip level0+level1 headers
                        axi_read_req_chirho <= 1'b1;

                        if (hbm_axi4_bus_chirho.rvalid && hbm_axi4_bus_chirho.rready) begin
                            hier_16m_1_level2_block_chirho[beat_counter_chirho[7:0]] <= hbm_axi4_bus_chirho.rdata;
                            beat_counter_chirho <= beat_counter_chirho + 1;

                            if (beat_counter_chirho >= 20'd255) begin
                                axi_read_req_chirho <= 1'b0;
                                beat_counter_chirho <= 20'b0;
                                fsm_state_chirho <= FSM_STREAM_LOAD_L2_V2_CHIRHO;
                            end
                        end
                    end else begin
                        // Skip this block (sparse: level1 bit not set)
                        fsm_state_chirho <= FSM_STREAM_NEXT_BLOCK_CHIRHO;
                    end
                end

                FSM_STREAM_LOAD_L2_V2_CHIRHO: begin
                    // Stream level2 block for var2
                    axi_addr_chirho <= var_to_hbm_addr_chirho(var_id_2_chirho) +
                                      ({12'b0, hier_16m_level1_idx_chirho} * 20'd8224) +
                                      ({12'b0, hier_16m_level2_idx_chirho} << 5) +
                                      20'd8224;
                    axi_read_req_chirho <= 1'b1;

                    if (hbm_axi4_bus_chirho.rvalid && hbm_axi4_bus_chirho.rready) begin
                        hier_16m_2_level2_block_chirho[beat_counter_chirho[7:0]] <= hbm_axi4_bus_chirho.rdata;
                        beat_counter_chirho <= beat_counter_chirho + 1;

                        if (beat_counter_chirho >= 20'd255) begin
                            axi_read_req_chirho <= 1'b0;
                            beat_counter_chirho <= 20'b0;
                            fsm_state_chirho <= FSM_STREAM_COMPUTE_CHIRHO;
                        end
                    end
                end

                FSM_STREAM_COMPUTE_CHIRHO: begin
                    // Intersect the current level2 blocks (256 parallel ANDs)
                    // Result computed combinatorially in generate block below
                    fsm_state_chirho <= FSM_STREAM_STORE_CHIRHO;
                end

                FSM_STREAM_STORE_CHIRHO: begin
                    // Write intersected level2 block back to result
                    axi_addr_chirho <= var_to_hbm_addr_chirho(var_id_1_chirho) +
                                      ({12'b0, hier_16m_level1_idx_chirho} * 20'd8224) +
                                      ({12'b0, hier_16m_level2_idx_chirho} << 5) +
                                      20'd8224;
                    axi_wdata_chirho <= hier_16m_result_level2_block_chirho[beat_counter_chirho[7:0]];
                    axi_write_req_chirho <= 1'b1;

                    if (hbm_axi4_bus_chirho.bvalid && hbm_axi4_bus_chirho.bready) begin
                        axi_write_req_chirho <= 1'b0;
                        beat_counter_chirho <= beat_counter_chirho + 1;

                        if (beat_counter_chirho >= 20'd255) begin
                            beat_counter_chirho <= 20'b0;
                            fsm_state_chirho <= FSM_STREAM_NEXT_BLOCK_CHIRHO;
                        end
                    end
                end

                FSM_STREAM_NEXT_BLOCK_CHIRHO: begin
                    // Advance to next level2 block, or next level1 block, or done
                    if (hier_16m_level2_idx_chirho >= 8'd255) begin
                        // Done with this level1 block, advance to next
                        hier_16m_level2_idx_chirho <= 8'b0;
                        if (hier_16m_level1_idx_chirho >= 8'd255) begin
                            // All done! Store summary and finish
                            op_done_chirho <= 1'b1;
                            op_valid_chirho <= (hier_16m_result_level0_chirho != 256'b0);
                            fsm_state_chirho <= FSM_BATCH_NEXT_CHIRHO;
                        end else begin
                            hier_16m_level1_idx_chirho <= hier_16m_level1_idx_chirho + 1;
                            fsm_state_chirho <= FSM_STREAM_LOAD_L2_V1_CHIRHO;
                        end
                    end else begin
                        hier_16m_level2_idx_chirho <= hier_16m_level2_idx_chirho + 1;
                        fsm_state_chirho <= FSM_STREAM_LOAD_L2_V1_CHIRHO;
                    end
                end

                FSM_BATCH_NEXT_CHIRHO: begin
                    batch_idx_chirho <= batch_idx_chirho + 1;
                    if (batch_idx_chirho >= batch_count_chirho - 1) begin
                        op_done_chirho   <= 1'b1;
                        fsm_state_chirho <= FSM_IDLE_CHIRHO;
                    end else begin
                        fsm_state_chirho <= FSM_IDLE_CHIRHO; // Re-enter for next batch
                    end
                end
            endcase
        end
    end

    // ========================================================================
    // Hierarchical Level1 Intersection (256 parallel 256-bit ANDs)
    // This runs combinatorially while FSM handles sequencing
    // ========================================================================
    genvar gi_chirho;
    for (gi_chirho = 0; gi_chirho < 256; gi_chirho = gi_chirho + 1) begin : hier_level1_intersect_chirho
        // Only intersect if both level0 bits indicate non-empty blocks
        wire level0_active_chirho = hier_65k_1_level0_chirho[gi_chirho] &
                                    hier_65k_2_level0_chirho[gi_chirho];

        // Gated intersection: if either level0 bit is 0, result is 0
        always_ff @(posedge clk_main_a0) begin
            if (fsm_state_chirho == FSM_COMPUTE_HIER_CHIRHO) begin
                if (level0_active_chirho) begin
                    hier_65k_result_level1_chirho[gi_chirho] <=
                        hier_65k_1_level1_chirho[gi_chirho] & hier_65k_2_level1_chirho[gi_chirho];
                end else begin
                    hier_65k_result_level1_chirho[gi_chirho] <= 256'b0;
                end
            end
        end
    end  // hier_level1_intersect_chirho

    // ========================================================================
    // Streaming Level2 Intersection (256 parallel 256-bit ANDs for current block)
    // Used by 3-level hierarchies (256³, 512³) in streaming mode
    // ========================================================================
    genvar gj_chirho;
    for (gj_chirho = 0; gj_chirho < 256; gj_chirho = gj_chirho + 1) begin : hier_level2_stream_intersect_chirho
        // Compute intersection of current level2 block
        always_ff @(posedge clk_main_a0) begin
            if (fsm_state_chirho == FSM_STREAM_COMPUTE_CHIRHO) begin
                hier_16m_result_level2_block_chirho[gj_chirho] <=
                    hier_16m_1_level2_block_chirho[gj_chirho] & hier_16m_2_level2_block_chirho[gj_chirho];
            end
        end
    end  // hier_level2_stream_intersect_chirho

    // Also compute level1 summary updates during streaming
    for (gj_chirho = 0; gj_chirho < 256; gj_chirho = gj_chirho + 1) begin : hier_level1_summary_update_chirho
        always_ff @(posedge clk_main_a0) begin
            if (fsm_state_chirho == FSM_LOAD_SUMMARIES_CHIRHO &&
                beat_counter_chirho == (beats_required_chirho * 2) - 1) begin
                // Intersect level1 summaries
                hier_16m_result_level1_chirho[gj_chirho] <=
                    hier_16m_1_level1_chirho[gj_chirho] & hier_16m_2_level1_chirho[gj_chirho];
            end
        end
    end  // hier_level1_summary_update_chirho

    // AXI4 read channel
    assign hbm_axi4_bus_chirho.arid    = 6'b0;
    assign hbm_axi4_bus_chirho.araddr  = axi_addr_chirho;
    assign hbm_axi4_bus_chirho.arlen   = axi_burst_len_chirho;
    assign hbm_axi4_bus_chirho.arsize  = 3'b101; // 32 bytes (256 bits)
    assign hbm_axi4_bus_chirho.arburst = 2'b01;  // INCR
    assign hbm_axi4_bus_chirho.arvalid = axi_read_req_chirho;
    assign hbm_axi4_bus_chirho.rready  = 1'b1;

    // AXI4 write channel
    assign hbm_axi4_bus_chirho.awid    = 6'b0;
    assign hbm_axi4_bus_chirho.awaddr  = axi_addr_chirho;
    assign hbm_axi4_bus_chirho.awlen   = 8'b0;  // Single beat writes (simplifies FSM)
    assign hbm_axi4_bus_chirho.awsize  = 3'b101; // 32 bytes
    assign hbm_axi4_bus_chirho.awburst = 2'b01;  // INCR
    assign hbm_axi4_bus_chirho.awvalid = axi_write_req_chirho;
    assign hbm_axi4_bus_chirho.wdata   = axi_wdata_chirho;
    assign hbm_axi4_bus_chirho.wstrb   = 32'hFFFFFFFF;
    assign hbm_axi4_bus_chirho.wlast   = 1'b1;
    assign hbm_axi4_bus_chirho.wvalid  = axi_write_req_chirho;
    assign hbm_axi4_bus_chirho.bready  = 1'b1;

    // Response register mapping
    // Includes hierarchy mode, beat counter for debug, and result summary
    assign resp_wire_chirho = {
        429'b0,                           // Padding
        ctrl_hier_mode_chirho,            // bits 84:82
        beat_counter_chirho,              // bits 81:66 (debug)
        op_done_chirho,                   // bit 65
        op_valid_chirho,                  // bit 64
        (ctrl_hier_mode_chirho == `HIER_MODE_FLAT256_CHIRHO) ?
            domain_result_flat_chirho[63:0] :
            hier_65k_result_level0_chirho[63:0]  // bits 63:0
    };

end : HBM_ENGINE
else begin : LEGACY_ENGINE

    // Legacy 64-bit register-based engine (no HBM)
    // NOTE: Port names .clk and .rst are Clash-generated standard clock/reset ports
    // and retain their original Clash names per AGENTS.md convention. Custom ports
    // (enChirho, cmdChirho, respChirho) use the Chirho suffix per project naming.
    // For larger domains, use hierarchical modules: intersect_hier_262k_chirho (512²)
    searchEngine64BitChirho u_engine_64bit_chirho (
        .clk        (clk_engine_chirho),
        .rst        (engine_rst_chirho),
        .enChirho   (ctrl_enable_chirho),
        .cmdChirho  (cmd_reg_chirho),
        .respChirho (resp_wire_chirho)
    );

end : LEGACY_ENGINE

    // ========================================================================
    // Neurosymbolic Training Engine (Q16.16 Gumbel-Softmax)
    // ========================================================================
    // Runs in parallel with search engine. Enable with train_enable_chirho.
    // Uses Gumbel-softmax reparameterization for differentiable discrete sampling.
    // Training FSM: Idle → LoadWeights → Sample → EvalClauses → AccumGrads → UpdateWeights → Done
    //
    // Command format (128 bits):
    //   [31:0]   - learning_rate (Q16.16)
    //   [63:32]  - temperature (Q16.16)
    //   [79:64]  - num_epochs
    //   [95:80]  - num_samples
    //   [127:96] - clause_count
    //
    // Response format (64 bits):
    //   [31:0]   - final_loss (Q16.16)
    //   [47:32]  - current_epoch
    //   [48]     - valid
    //   [49]     - done
    //
    // NOTE: Port naming convention per AGENTS.md:
    //   - 'clk', 'rst' = Clash standard ports (exempt from _chirho suffix)
    //   - 'en_chirho', 'cmd_chirho', 'resp_chirho' = Project custom ports (_chirho suffix)

    diffTrainChirho u_train_engine_chirho (
        .clk        (clk_main_a0),
        .rst        (engine_rst_chirho || train_reset_chirho),
        .en_chirho  (train_enable_chirho),
        .cmd_chirho (train_cmd_chirho),
        .resp_chirho(train_resp_chirho)
    );

    // ========================================================================
    // Probabilistic Inference Engines (Q16.16 and Q8.8 Soft AND)
    // ========================================================================
    // These modules perform differentiable "soft" intersection for probabilistic
    // reasoning. Used when infer_prob_mode_chirho = 1.
    //
    // NOTE: Port naming convention per AGENTS.md:
    //   - All ports use _chirho suffix (Clash custom ports)

    // Single Q16.16 soft AND (for scalar probability multiplication)
    soft_and_32_chirho u_soft_and_32_chirho (
        .a_chirho      (soft_and_a_chirho),
        .b_chirho      (soft_and_b_chirho),
        .result_chirho (soft_and_result_chirho)
    );

    // 64-element Q8.8 probabilistic domain intersection
    // Input: 64 × Q8.8 probabilities + 64-bit presence mask = 1088 bits
    // NOTE: This module internally instantiates soft_and_16_chirho
    intersect_prob_domain_64_chirho u_prob_domain_64_chirho (
        .domain_a_chirho   (prob_domain_a_chirho),
        .domain_b_chirho   (prob_domain_b_chirho),
        .domain_out_chirho (prob_domain_result_chirho)
    );

endmodule

`default_nettype wire
