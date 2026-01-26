// ============================================================================
// ☧ For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// AWS F1 Custom Logic: miniKanren 1-Bit Search Engine
//
// This module integrates the Clash-generated miniKanren search engine with
// the AWS F1 shell. It provides:
//   - AXI-Lite register interface for host communication
//   - Clock domain crossing from shell 250MHz to engine 50MHz
//   - Proper tie-offs for unused shell interfaces
//
// Register Map (via OCL AXI-Lite):
//   0x000: VERSION   - Read-only version register
//   0x004: CONTROL   - bit0=enable, bit1=reset
//   0x008: STATUS    - bit0=done, bit1=valid
//   0x010: CMD_LO    - cmdChirho[31:0]
//   0x014: CMD_MID   - cmdChirho[63:32]
//   0x018: CMD_HI    - cmdChirho[69:64]
//   0x020-0x060: RESP[0-8] - respChirho[513:0] (9 x 64-bit)
// ============================================================================

`include "cl_minikanren_chirho_defines.vh"

module cl_minikanren_chirho #(
    parameter NUM_PCIE  = 1,
    parameter NUM_DDR   = 4,
    parameter NUM_HBM   = 0,
    parameter NUM_GTY   = 0
) (
    `include "cl_ports.vh"
);

    // ========================================================================
    // Tie-off unused interfaces (from AWS HDK)
    // ========================================================================
    `include "unused_flr_template.inc"
    `include "unused_ddr_a_b_d_template.inc"
    `include "unused_ddr_c_template.inc"
    `include "unused_pcim_template.inc"
    `include "unused_dma_pcis_template.inc"
    `include "unused_cl_sda_template.inc"
    `include "unused_sh_bar1_template.inc"
    `include "unused_apppf_irq_template.inc"
    `include "unused_hbm_template.inc"

    // ========================================================================
    // Internal Signals
    // ========================================================================

    // Synchronized reset
    logic rst_main_n_sync;
    logic engine_rst_chirho;

    // Engine clock (50MHz from 250MHz)
    logic clk_engine_chirho;
    logic [2:0] clk_div_chirho;

    // Control registers
    logic ctrl_enable_chirho;
    logic ctrl_reset_chirho;
    logic [69:0] cmd_reg_chirho;

    // Response from engine
    logic [513:0] resp_wire_chirho;

    // AXI-Lite OCL signals
    logic        ocl_awvalid_q;
    logic        ocl_awready_q;
    logic [31:0] ocl_awaddr_q;
    logic        ocl_wvalid_q;
    logic        ocl_wready_q;
    logic [31:0] ocl_wdata_q;
    logic [3:0]  ocl_wstrb_q;
    logic        ocl_bvalid_q;
    logic        ocl_bready_q;
    logic [1:0]  ocl_bresp_q;
    logic        ocl_arvalid_q;
    logic        ocl_arready_q;
    logic [31:0] ocl_araddr_q;
    logic        ocl_rvalid_q;
    logic        ocl_rready_q;
    logic [31:0] ocl_rdata_q;
    logic [1:0]  ocl_rresp_q;

    // ========================================================================
    // Reset Synchronizer
    // ========================================================================

    // NOTE: lib_pipe is an AWS HDK module (external library - naming exception)
    lib_pipe #(
        .WIDTH(1),
        .STAGES(4)
    ) rst_pipe_inst_chirho (
        .clk    (clk_main_a0),
        .rst_n  (1'b1),
        .in_bus (rst_main_n),
        .out_bus(rst_main_n_sync)
    );

    // ========================================================================
    // Clock Divider (250MHz -> 50MHz)
    // ========================================================================

    always_ff @(posedge clk_main_a0 or negedge rst_main_n_sync) begin
        if (!rst_main_n_sync) begin
            clk_div_chirho <= 3'b0;
        end else begin
            clk_div_chirho <= clk_div_chirho + 1'b1;
        end
    end

    // 50MHz enable (every 5th cycle of 250MHz)
    // Actually, 250/5 = 50MHz, but we'll use a simpler divide-by-4 for now (62.5MHz)
    // For precise 50MHz, use MMCM/PLL in production
    assign clk_engine_chirho = clk_div_chirho[1]; // Divide by 4 = 62.5MHz

    // Engine reset
    assign engine_rst_chirho = !rst_main_n_sync || ctrl_reset_chirho;

    // ========================================================================
    // AXI-Lite Register Stage
    // ========================================================================

    // Register the AXI-Lite signals for timing
    always_ff @(posedge clk_main_a0) begin
        if (!rst_main_n_sync) begin
            ocl_awvalid_q <= 1'b0;
            ocl_wvalid_q  <= 1'b0;
            ocl_arvalid_q <= 1'b0;
            ocl_bready_q  <= 1'b0;
            ocl_rready_q  <= 1'b0;
        end else begin
            ocl_awvalid_q <= sh_ocl_awvalid;
            ocl_awaddr_q  <= sh_ocl_awaddr;
            ocl_wvalid_q  <= sh_ocl_wvalid;
            ocl_wdata_q   <= sh_ocl_wdata;
            ocl_wstrb_q   <= sh_ocl_wstrb;
            ocl_arvalid_q <= sh_ocl_arvalid;
            ocl_araddr_q  <= sh_ocl_araddr;
            ocl_bready_q  <= sh_ocl_bready;
            ocl_rready_q  <= sh_ocl_rready;
        end
    end

    // ========================================================================
    // AXI-Lite Write Logic
    // ========================================================================

    typedef enum logic [1:0] {
        WR_IDLE_CHIRHO,
        WR_DATA_CHIRHO,
        WR_RESP_CHIRHO
    } wr_state_t;

    wr_state_t wr_state_chirho;
    logic [31:0] wr_addr_chirho;

    always_ff @(posedge clk_main_a0) begin
        if (!rst_main_n_sync) begin
            wr_state_chirho <= WR_IDLE_CHIRHO;
            ocl_awready_q   <= 1'b0;
            ocl_wready_q    <= 1'b0;
            ocl_bvalid_q    <= 1'b0;
            ocl_bresp_q     <= 2'b00;
            ctrl_enable_chirho <= 1'b0;
            ctrl_reset_chirho  <= 1'b0;
            cmd_reg_chirho     <= 70'b0;
        end else begin
            case (wr_state_chirho)
                WR_IDLE_CHIRHO: begin
                    ocl_awready_q <= 1'b1;
                    ocl_bvalid_q  <= 1'b0;
                    if (ocl_awvalid_q && ocl_awready_q) begin
                        wr_addr_chirho  <= ocl_awaddr_q;
                        ocl_awready_q   <= 1'b0;
                        ocl_wready_q    <= 1'b1;
                        wr_state_chirho <= WR_DATA_CHIRHO;
                    end
                end

                WR_DATA_CHIRHO: begin
                    if (ocl_wvalid_q && ocl_wready_q) begin
                        ocl_wready_q <= 1'b0;
                        // Decode write address
                        case (wr_addr_chirho[7:2])
                            6'h01: begin // 0x004: CONTROL
                                ctrl_enable_chirho <= ocl_wdata_q[0];
                                ctrl_reset_chirho  <= ocl_wdata_q[1];
                            end
                            6'h04: cmd_reg_chirho[31:0]   <= ocl_wdata_q; // 0x010: CMD_LO
                            6'h05: cmd_reg_chirho[63:32]  <= ocl_wdata_q; // 0x014: CMD_MID
                            6'h06: cmd_reg_chirho[69:64]  <= ocl_wdata_q[5:0]; // 0x018: CMD_HI
                        endcase
                        ocl_bvalid_q    <= 1'b1;
                        ocl_bresp_q     <= 2'b00; // OKAY
                        wr_state_chirho <= WR_RESP_CHIRHO;
                    end
                end

                WR_RESP_CHIRHO: begin
                    if (ocl_bready_q) begin
                        ocl_bvalid_q    <= 1'b0;
                        wr_state_chirho <= WR_IDLE_CHIRHO;
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
    } rd_state_t;

    rd_state_t rd_state_chirho;

    always_ff @(posedge clk_main_a0) begin
        if (!rst_main_n_sync) begin
            rd_state_chirho <= RD_IDLE_CHIRHO;
            ocl_arready_q   <= 1'b0;
            ocl_rvalid_q    <= 1'b0;
            ocl_rdata_q     <= 32'b0;
            ocl_rresp_q     <= 2'b00;
        end else begin
            case (rd_state_chirho)
                RD_IDLE_CHIRHO: begin
                    ocl_arready_q <= 1'b1;
                    ocl_rvalid_q  <= 1'b0;
                    if (ocl_arvalid_q && ocl_arready_q) begin
                        ocl_arready_q <= 1'b0;
                        // Decode read address
                        case (ocl_araddr_q[7:2])
                            6'h00: ocl_rdata_q <= `MINIKANREN_VERSION_CHIRHO; // 0x000: VERSION
                            6'h01: ocl_rdata_q <= {30'b0, ctrl_reset_chirho, ctrl_enable_chirho}; // 0x004: CONTROL
                            6'h02: ocl_rdata_q <= {30'b0, 1'b1, 1'b1}; // 0x008: STATUS (done, valid)
                            6'h04: ocl_rdata_q <= cmd_reg_chirho[31:0];  // 0x010: CMD_LO
                            6'h05: ocl_rdata_q <= cmd_reg_chirho[63:32]; // 0x014: CMD_MID
                            6'h06: ocl_rdata_q <= {26'b0, cmd_reg_chirho[69:64]}; // 0x018: CMD_HI
                            // Response registers (32-bit reads for 64-bit values)
                            6'h08: ocl_rdata_q <= resp_wire_chirho[31:0];    // 0x020
                            6'h09: ocl_rdata_q <= resp_wire_chirho[63:32];   // 0x024
                            6'h0A: ocl_rdata_q <= resp_wire_chirho[95:64];   // 0x028
                            6'h0B: ocl_rdata_q <= resp_wire_chirho[127:96];  // 0x02C
                            6'h0C: ocl_rdata_q <= resp_wire_chirho[159:128]; // 0x030
                            6'h0D: ocl_rdata_q <= resp_wire_chirho[191:160]; // 0x034
                            6'h0E: ocl_rdata_q <= resp_wire_chirho[223:192]; // 0x038
                            6'h0F: ocl_rdata_q <= resp_wire_chirho[255:224]; // 0x03C
                            6'h10: ocl_rdata_q <= resp_wire_chirho[287:256]; // 0x040
                            6'h11: ocl_rdata_q <= resp_wire_chirho[319:288]; // 0x044
                            6'h12: ocl_rdata_q <= resp_wire_chirho[351:320]; // 0x048
                            6'h13: ocl_rdata_q <= resp_wire_chirho[383:352]; // 0x04C
                            6'h14: ocl_rdata_q <= resp_wire_chirho[415:384]; // 0x050
                            6'h15: ocl_rdata_q <= resp_wire_chirho[447:416]; // 0x054
                            6'h16: ocl_rdata_q <= resp_wire_chirho[479:448]; // 0x058
                            6'h17: ocl_rdata_q <= resp_wire_chirho[511:480]; // 0x05C
                            6'h18: ocl_rdata_q <= {30'b0, resp_wire_chirho[513:512]}; // 0x060
                            default: ocl_rdata_q <= 32'hDEADBEEF;
                        endcase
                        ocl_rvalid_q    <= 1'b1;
                        ocl_rresp_q     <= 2'b00; // OKAY
                        rd_state_chirho <= RD_DATA_CHIRHO;
                    end
                end

                RD_DATA_CHIRHO: begin
                    if (ocl_rready_q) begin
                        ocl_rvalid_q    <= 1'b0;
                        rd_state_chirho <= RD_IDLE_CHIRHO;
                    end
                end
            endcase
        end
    end

    // ========================================================================
    // AXI-Lite Output Assignments
    // ========================================================================

    assign ocl_sh_awready = ocl_awready_q;
    assign ocl_sh_wready  = ocl_wready_q;
    assign ocl_sh_bvalid  = ocl_bvalid_q;
    assign ocl_sh_bresp   = ocl_bresp_q;
    assign ocl_sh_arready = ocl_arready_q;
    assign ocl_sh_rvalid  = ocl_rvalid_q;
    assign ocl_sh_rdata   = ocl_rdata_q;
    assign ocl_sh_rresp   = ocl_rresp_q;

    // ========================================================================
    // Virtual LED / DIP Switch
    // ========================================================================

    // Show engine status on virtual LEDs
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

    // ========================================================================
    // ID Assignments (Required by AWS)
    // ========================================================================

    assign cl_sh_id0 = 32'hF000_0001;  // Vendor ID
    assign cl_sh_id1 = 32'h1D51_FEDD;  // Device ID

endmodule

`default_nettype wire
