// ☧ For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life. - John 3:16
//
// AWS F1 Customer Logic Wrapper for miniKanren Search Engine
// Provides AXI-Lite interface for host communication
//
// Register Map (64-bit aligned):
//   0x00: Control (write: bit0=enable, bit1=reset)
//   0x08: Status  (read: bit0=done)
//   0x10-0x18: cmdChirho[69:0] (write: command input)
//   0x20-0x60: respChirho[513:0] (read: response output, 9 x 64-bit)

`default_nettype none
`timescale 1ns/1ps

module cl_minikanren_chirho #(
    parameter C_S_AXI_ADDR_WIDTH = 8,
    parameter C_S_AXI_DATA_WIDTH = 64
) (
    // AXI-Lite Slave Interface
    input  wire                             s_axi_aclk,
    input  wire                             s_axi_aresetn,
    // Write address
    input  wire [C_S_AXI_ADDR_WIDTH-1:0]    s_axi_awaddr,
    input  wire                             s_axi_awvalid,
    output wire                             s_axi_awready,
    // Write data
    input  wire [C_S_AXI_DATA_WIDTH-1:0]    s_axi_wdata,
    input  wire [C_S_AXI_DATA_WIDTH/8-1:0]  s_axi_wstrb,
    input  wire                             s_axi_wvalid,
    output wire                             s_axi_wready,
    // Write response
    output wire [1:0]                       s_axi_bresp,
    output wire                             s_axi_bvalid,
    input  wire                             s_axi_bready,
    // Read address
    input  wire [C_S_AXI_ADDR_WIDTH-1:0]    s_axi_araddr,
    input  wire                             s_axi_arvalid,
    output wire                             s_axi_arready,
    // Read data
    output wire [C_S_AXI_DATA_WIDTH-1:0]    s_axi_rdata,
    output wire [1:0]                       s_axi_rresp,
    output wire                             s_axi_rvalid,
    input  wire                             s_axi_rready
);

    // ========================================
    // Internal Registers
    // ========================================

    reg        ctrl_enable_chirho;
    reg        ctrl_reset_chirho;
    reg [69:0] cmd_reg_chirho;

    wire [513:0] resp_wire_chirho;

    // ========================================
    // AXI-Lite State Machine
    // ========================================

    localparam IDLE_CHIRHO    = 2'b00;
    localparam WRITE_CHIRHO   = 2'b01;
    localparam READ_CHIRHO    = 2'b10;
    localparam RESPOND_CHIRHO = 2'b11;

    reg [1:0] state_chirho;
    reg [C_S_AXI_ADDR_WIDTH-1:0] addr_latch_chirho;
    reg [C_S_AXI_DATA_WIDTH-1:0] rdata_chirho;
    reg awready_chirho, wready_chirho, bvalid_chirho;
    reg arready_chirho, rvalid_chirho;

    assign s_axi_awready = awready_chirho;
    assign s_axi_wready  = wready_chirho;
    assign s_axi_bresp   = 2'b00;  // OKAY
    assign s_axi_bvalid  = bvalid_chirho;
    assign s_axi_arready = arready_chirho;
    assign s_axi_rdata   = rdata_chirho;
    assign s_axi_rresp   = 2'b00;  // OKAY
    assign s_axi_rvalid  = rvalid_chirho;

    always @(posedge s_axi_aclk) begin
        if (!s_axi_aresetn) begin
            state_chirho <= IDLE_CHIRHO;
            ctrl_enable_chirho <= 1'b0;
            ctrl_reset_chirho  <= 1'b0;
            cmd_reg_chirho     <= 70'b0;
            awready_chirho <= 1'b0;
            wready_chirho  <= 1'b0;
            bvalid_chirho  <= 1'b0;
            arready_chirho <= 1'b0;
            rvalid_chirho  <= 1'b0;
            rdata_chirho   <= 64'b0;
        end else begin
            case (state_chirho)
                IDLE_CHIRHO: begin
                    awready_chirho <= 1'b1;
                    arready_chirho <= 1'b1;
                    bvalid_chirho  <= 1'b0;
                    rvalid_chirho  <= 1'b0;

                    if (s_axi_awvalid && awready_chirho) begin
                        addr_latch_chirho <= s_axi_awaddr;
                        awready_chirho <= 1'b0;
                        arready_chirho <= 1'b0;
                        wready_chirho  <= 1'b1;
                        state_chirho   <= WRITE_CHIRHO;
                    end else if (s_axi_arvalid && arready_chirho) begin
                        addr_latch_chirho <= s_axi_araddr;
                        awready_chirho <= 1'b0;
                        arready_chirho <= 1'b0;
                        state_chirho   <= READ_CHIRHO;
                    end
                end

                WRITE_CHIRHO: begin
                    if (s_axi_wvalid && wready_chirho) begin
                        wready_chirho <= 1'b0;
                        // Write to registers based on address
                        case (addr_latch_chirho[7:3])
                            5'h00: begin  // 0x00: Control
                                ctrl_enable_chirho <= s_axi_wdata[0];
                                ctrl_reset_chirho  <= s_axi_wdata[1];
                            end
                            5'h02: cmd_reg_chirho[63:0]  <= s_axi_wdata;     // 0x10
                            5'h03: cmd_reg_chirho[69:64] <= s_axi_wdata[5:0]; // 0x18
                        endcase
                        bvalid_chirho  <= 1'b1;
                        state_chirho   <= RESPOND_CHIRHO;
                    end
                end

                READ_CHIRHO: begin
                    // Read from registers based on address
                    case (addr_latch_chirho[7:3])
                        5'h00: rdata_chirho <= {62'b0, ctrl_reset_chirho, ctrl_enable_chirho}; // Control
                        5'h01: rdata_chirho <= {63'b0, 1'b1}; // Status (always done for now)
                        5'h02: rdata_chirho <= cmd_reg_chirho[63:0];   // cmdChirho[63:0]
                        5'h03: rdata_chirho <= {58'b0, cmd_reg_chirho[69:64]}; // cmdChirho[69:64]
                        // respChirho[513:0] = 9 x 64-bit registers at 0x20-0x60
                        5'h04: rdata_chirho <= resp_wire_chirho[63:0];
                        5'h05: rdata_chirho <= resp_wire_chirho[127:64];
                        5'h06: rdata_chirho <= resp_wire_chirho[191:128];
                        5'h07: rdata_chirho <= resp_wire_chirho[255:192];
                        5'h08: rdata_chirho <= resp_wire_chirho[319:256];
                        5'h09: rdata_chirho <= resp_wire_chirho[383:320];
                        5'h0A: rdata_chirho <= resp_wire_chirho[447:384];
                        5'h0B: rdata_chirho <= resp_wire_chirho[511:448];
                        5'h0C: rdata_chirho <= {62'b0, resp_wire_chirho[513:512]};
                        default: rdata_chirho <= 64'hDEADBEEF_DEADBEEF;
                    endcase
                    rvalid_chirho <= 1'b1;
                    state_chirho  <= RESPOND_CHIRHO;
                end

                RESPOND_CHIRHO: begin
                    if ((bvalid_chirho && s_axi_bready) || (rvalid_chirho && s_axi_rready)) begin
                        bvalid_chirho <= 1'b0;
                        rvalid_chirho <= 1'b0;
                        state_chirho  <= IDLE_CHIRHO;
                    end
                end
            endcase
        end
    end

    // ========================================
    // miniKanren Search Engine Instance
    // ========================================

    searchEngineChirho u_engine_chirho (
        .clk        (s_axi_aclk),
        .rst        (ctrl_reset_chirho || !s_axi_aresetn),
        .enChirho   (ctrl_enable_chirho),
        .cmdChirho  (cmd_reg_chirho),
        .respChirho (resp_wire_chirho)
    );

endmodule

`default_nettype wire
