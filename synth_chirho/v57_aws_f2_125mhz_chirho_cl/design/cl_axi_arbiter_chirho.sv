// ============================================================================
// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// AXI Arbiter for HBM Access ☧
//
// Arbitrates between two AXI masters (PCIS and FSM) for HBM access.
// PCIS has higher priority (host DMA traffic).
//
// Design choice: Simple round-robin with PCIS priority, not a full crossbar.
// Rationale: V5.5 had -1.451ns WNS at 200MHz - can't afford crossbar latency.
//
// V5.6: Updated to use 512-bit data, 64-bit addresses, 16-bit IDs
//       to match axi_bus_t interface expected by cl_hbm_axi4.
//
// ============================================================================

module cl_axi_arbiter_chirho (
    input  logic         clk,
    input  logic         rst_n,

    // ========================================================================
    // Port 0: PCIS (higher priority - host DMA)
    // Uses 512-bit data, 64-bit address, 16-bit ID to match shell interface
    // ========================================================================

    // Write address channel
    input  logic [63:0]  pcis_awaddr_chirho,
    input  logic [15:0]  pcis_awid_chirho,
    input  logic [7:0]   pcis_awlen_chirho,
    input  logic [2:0]   pcis_awsize_chirho,
    input  logic [1:0]   pcis_awburst_chirho,
    input  logic         pcis_awvalid_chirho,
    output logic         pcis_awready_chirho,

    // Write data channel
    input  logic [511:0] pcis_wdata_chirho,
    input  logic [63:0]  pcis_wstrb_chirho,
    input  logic         pcis_wlast_chirho,
    input  logic         pcis_wvalid_chirho,
    output logic         pcis_wready_chirho,

    // Write response channel
    output logic [15:0]  pcis_bid_chirho,
    output logic [1:0]   pcis_bresp_chirho,
    output logic         pcis_bvalid_chirho,
    input  logic         pcis_bready_chirho,

    // Read address channel
    input  logic [63:0]  pcis_araddr_chirho,
    input  logic [15:0]  pcis_arid_chirho,
    input  logic [7:0]   pcis_arlen_chirho,
    input  logic [2:0]   pcis_arsize_chirho,
    input  logic [1:0]   pcis_arburst_chirho,
    input  logic         pcis_arvalid_chirho,
    output logic         pcis_arready_chirho,

    // Read data channel
    output logic [15:0]  pcis_rid_chirho,
    output logic [511:0] pcis_rdata_chirho,
    output logic [1:0]   pcis_rresp_chirho,
    output logic         pcis_rlast_chirho,
    output logic         pcis_rvalid_chirho,
    input  logic         pcis_rready_chirho,

    // ========================================================================
    // Port 1: FSM (lower priority - internal engine)
    // Uses 64-bit address, 16-bit ID, 512-bit data (matching axi_bus_t)
    // ========================================================================

    // Write address channel
    input  logic [63:0]  fsm_awaddr_chirho,
    input  logic [15:0]  fsm_awid_chirho,
    input  logic [7:0]   fsm_awlen_chirho,
    input  logic [2:0]   fsm_awsize_chirho,
    input  logic [1:0]   fsm_awburst_chirho,
    input  logic         fsm_awvalid_chirho,
    output logic         fsm_awready_chirho,

    // Write data channel
    input  logic [511:0] fsm_wdata_chirho,
    input  logic [63:0]  fsm_wstrb_chirho,
    input  logic         fsm_wlast_chirho,
    input  logic         fsm_wvalid_chirho,
    output logic         fsm_wready_chirho,

    // Write response channel
    output logic [15:0]  fsm_bid_chirho,
    output logic [1:0]   fsm_bresp_chirho,
    output logic         fsm_bvalid_chirho,
    input  logic         fsm_bready_chirho,

    // Read address channel
    input  logic [63:0]  fsm_araddr_chirho,
    input  logic [15:0]  fsm_arid_chirho,
    input  logic [7:0]   fsm_arlen_chirho,
    input  logic [2:0]   fsm_arsize_chirho,
    input  logic [1:0]   fsm_arburst_chirho,
    input  logic         fsm_arvalid_chirho,
    output logic         fsm_arready_chirho,

    // Read data channel
    output logic [15:0]  fsm_rid_chirho,
    output logic [511:0] fsm_rdata_chirho,
    output logic [1:0]   fsm_rresp_chirho,
    output logic         fsm_rlast_chirho,
    output logic         fsm_rvalid_chirho,
    input  logic         fsm_rready_chirho,

    // ========================================================================
    // Output to HBM (via cl_hbm_axi4)
    // Uses 512-bit data, 64-bit address, 16-bit ID to match axi_bus_t
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
    // Debug
    // ========================================================================
    output logic         grant_pcis_chirho,
    output logic         grant_fsm_chirho
);

    // ========================================================================
    // Arbiter FSM
    // ========================================================================
    // States: IDLE, GRANT_PCIS_WRITE, GRANT_PCIS_READ, GRANT_FSM_WRITE, GRANT_FSM_READ
    // Switch only at transaction boundaries (bvalid/bready or rlast/rvalid/rready)

    typedef enum logic [2:0] {
        ARB_IDLE_CHIRHO,
        ARB_PCIS_WRITE_CHIRHO,
        ARB_PCIS_READ_CHIRHO,
        ARB_FSM_WRITE_CHIRHO,
        ARB_FSM_READ_CHIRHO
    } arb_state_t_chirho;

    arb_state_t_chirho arb_state_chirho;

    // Transaction tracking
    logic write_in_progress_chirho;
    logic read_in_progress_chirho;
    logic [8:0] write_beats_remaining_chirho;
    logic [8:0] read_beats_remaining_chirho;

    // ========================================================================
    // Arbiter State Machine
    // ========================================================================

    always_ff @(posedge clk) begin
        if (!rst_n) begin
            arb_state_chirho <= ARB_IDLE_CHIRHO;
            write_in_progress_chirho <= 1'b0;
            read_in_progress_chirho <= 1'b0;
            write_beats_remaining_chirho <= 9'b0;
            read_beats_remaining_chirho <= 9'b0;
        end else begin
            case (arb_state_chirho)
                ARB_IDLE_CHIRHO: begin
                    // PCIS has higher priority
                    if (pcis_awvalid_chirho) begin
                        arb_state_chirho <= ARB_PCIS_WRITE_CHIRHO;
                        write_beats_remaining_chirho <= {1'b0, pcis_awlen_chirho} + 9'd1;
                    end else if (pcis_arvalid_chirho) begin
                        arb_state_chirho <= ARB_PCIS_READ_CHIRHO;
                        read_beats_remaining_chirho <= {1'b0, pcis_arlen_chirho} + 9'd1;
                    end else if (fsm_awvalid_chirho) begin
                        arb_state_chirho <= ARB_FSM_WRITE_CHIRHO;
                        write_beats_remaining_chirho <= {1'b0, fsm_awlen_chirho} + 9'd1;
                    end else if (fsm_arvalid_chirho) begin
                        arb_state_chirho <= ARB_FSM_READ_CHIRHO;
                        read_beats_remaining_chirho <= {1'b0, fsm_arlen_chirho} + 9'd1;
                    end
                end

                ARB_PCIS_WRITE_CHIRHO: begin
                    // Track write beats
                    if (hbm_wvalid_chirho && hbm_wready_chirho) begin
                        write_beats_remaining_chirho <= write_beats_remaining_chirho - 9'd1;
                    end

                    // Wait for write response
                    if (hbm_bvalid_chirho && pcis_bready_chirho) begin
                        arb_state_chirho <= ARB_IDLE_CHIRHO;
                    end
                end

                ARB_PCIS_READ_CHIRHO: begin
                    // Track read beats
                    if (hbm_rvalid_chirho && pcis_rready_chirho) begin
                        read_beats_remaining_chirho <= read_beats_remaining_chirho - 9'd1;
                        if (hbm_rlast_chirho) begin
                            arb_state_chirho <= ARB_IDLE_CHIRHO;
                        end
                    end
                end

                ARB_FSM_WRITE_CHIRHO: begin
                    // Track write beats
                    if (hbm_wvalid_chirho && hbm_wready_chirho) begin
                        write_beats_remaining_chirho <= write_beats_remaining_chirho - 9'd1;
                    end

                    // Wait for write response
                    if (hbm_bvalid_chirho && fsm_bready_chirho) begin
                        arb_state_chirho <= ARB_IDLE_CHIRHO;
                    end
                end

                ARB_FSM_READ_CHIRHO: begin
                    // Track read beats
                    if (hbm_rvalid_chirho && fsm_rready_chirho) begin
                        read_beats_remaining_chirho <= read_beats_remaining_chirho - 9'd1;
                        if (hbm_rlast_chirho) begin
                            arb_state_chirho <= ARB_IDLE_CHIRHO;
                        end
                    end
                end

                default: begin
                    arb_state_chirho <= ARB_IDLE_CHIRHO;
                end
            endcase
        end
    end

    // ========================================================================
    // Output Multiplexing
    // ========================================================================

    // Grant signals for debug
    assign grant_pcis_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO) ||
                               (arb_state_chirho == ARB_PCIS_READ_CHIRHO);
    assign grant_fsm_chirho = (arb_state_chirho == ARB_FSM_WRITE_CHIRHO) ||
                              (arb_state_chirho == ARB_FSM_READ_CHIRHO);

    // Write address mux
    assign hbm_awaddr_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_awvalid_chirho) ?
                               pcis_awaddr_chirho : fsm_awaddr_chirho;
    assign hbm_awid_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_awvalid_chirho) ?
                             pcis_awid_chirho : fsm_awid_chirho;
    assign hbm_awlen_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_awvalid_chirho) ?
                              pcis_awlen_chirho : fsm_awlen_chirho;
    assign hbm_awsize_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_awvalid_chirho) ?
                               pcis_awsize_chirho : fsm_awsize_chirho;
    assign hbm_awburst_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_awvalid_chirho) ?
                                pcis_awburst_chirho : fsm_awburst_chirho;
    assign hbm_awvalid_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO) ? pcis_awvalid_chirho :
                                (arb_state_chirho == ARB_FSM_WRITE_CHIRHO) ? fsm_awvalid_chirho :
                                (arb_state_chirho == ARB_IDLE_CHIRHO) ? (pcis_awvalid_chirho || fsm_awvalid_chirho) : 1'b0;

    // Write address ready
    assign pcis_awready_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO) ? hbm_awready_chirho : 1'b0;
    assign fsm_awready_chirho = (arb_state_chirho == ARB_FSM_WRITE_CHIRHO) ? hbm_awready_chirho :
                                (arb_state_chirho == ARB_IDLE_CHIRHO && !pcis_awvalid_chirho) ? hbm_awready_chirho : 1'b0;

    // Write data mux
    assign hbm_wdata_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO) ? pcis_wdata_chirho : fsm_wdata_chirho;
    assign hbm_wstrb_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO) ? pcis_wstrb_chirho : fsm_wstrb_chirho;
    assign hbm_wlast_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO) ? pcis_wlast_chirho : fsm_wlast_chirho;
    assign hbm_wvalid_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO) ? pcis_wvalid_chirho :
                               (arb_state_chirho == ARB_FSM_WRITE_CHIRHO) ? fsm_wvalid_chirho : 1'b0;

    // Write data ready
    assign pcis_wready_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO) ? hbm_wready_chirho : 1'b0;
    assign fsm_wready_chirho = (arb_state_chirho == ARB_FSM_WRITE_CHIRHO) ? hbm_wready_chirho : 1'b0;

    // Write response
    assign pcis_bid_chirho = hbm_bid_chirho;
    assign pcis_bresp_chirho = hbm_bresp_chirho;
    assign pcis_bvalid_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO) ? hbm_bvalid_chirho : 1'b0;

    assign fsm_bid_chirho = hbm_bid_chirho;
    assign fsm_bresp_chirho = hbm_bresp_chirho;
    assign fsm_bvalid_chirho = (arb_state_chirho == ARB_FSM_WRITE_CHIRHO) ? hbm_bvalid_chirho : 1'b0;

    assign hbm_bready_chirho = (arb_state_chirho == ARB_PCIS_WRITE_CHIRHO) ? pcis_bready_chirho :
                               (arb_state_chirho == ARB_FSM_WRITE_CHIRHO) ? fsm_bready_chirho : 1'b0;

    // Read address mux
    assign hbm_araddr_chirho = (arb_state_chirho == ARB_PCIS_READ_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_arvalid_chirho) ?
                               pcis_araddr_chirho : fsm_araddr_chirho;
    assign hbm_arid_chirho = (arb_state_chirho == ARB_PCIS_READ_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_arvalid_chirho) ?
                             pcis_arid_chirho : fsm_arid_chirho;
    assign hbm_arlen_chirho = (arb_state_chirho == ARB_PCIS_READ_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_arvalid_chirho) ?
                              pcis_arlen_chirho : fsm_arlen_chirho;
    assign hbm_arsize_chirho = (arb_state_chirho == ARB_PCIS_READ_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_arvalid_chirho) ?
                               pcis_arsize_chirho : fsm_arsize_chirho;
    assign hbm_arburst_chirho = (arb_state_chirho == ARB_PCIS_READ_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO && pcis_arvalid_chirho) ?
                                pcis_arburst_chirho : fsm_arburst_chirho;
    assign hbm_arvalid_chirho = (arb_state_chirho == ARB_PCIS_READ_CHIRHO) ? pcis_arvalid_chirho :
                                (arb_state_chirho == ARB_FSM_READ_CHIRHO) ? fsm_arvalid_chirho :
                                (arb_state_chirho == ARB_IDLE_CHIRHO && !pcis_awvalid_chirho) ? (pcis_arvalid_chirho || fsm_arvalid_chirho) : 1'b0;

    // Read address ready
    assign pcis_arready_chirho = (arb_state_chirho == ARB_PCIS_READ_CHIRHO || arb_state_chirho == ARB_IDLE_CHIRHO) ? hbm_arready_chirho : 1'b0;
    assign fsm_arready_chirho = (arb_state_chirho == ARB_FSM_READ_CHIRHO) ? hbm_arready_chirho :
                                (arb_state_chirho == ARB_IDLE_CHIRHO && !pcis_arvalid_chirho && !pcis_awvalid_chirho) ? hbm_arready_chirho : 1'b0;

    // Read data
    assign pcis_rid_chirho = hbm_rid_chirho;
    assign pcis_rdata_chirho = hbm_rdata_chirho;
    assign pcis_rresp_chirho = hbm_rresp_chirho;
    assign pcis_rlast_chirho = hbm_rlast_chirho;
    assign pcis_rvalid_chirho = (arb_state_chirho == ARB_PCIS_READ_CHIRHO) ? hbm_rvalid_chirho : 1'b0;

    assign fsm_rid_chirho = hbm_rid_chirho;
    assign fsm_rdata_chirho = hbm_rdata_chirho;
    assign fsm_rresp_chirho = hbm_rresp_chirho;
    assign fsm_rlast_chirho = hbm_rlast_chirho;
    assign fsm_rvalid_chirho = (arb_state_chirho == ARB_FSM_READ_CHIRHO) ? hbm_rvalid_chirho : 1'b0;

    assign hbm_rready_chirho = (arb_state_chirho == ARB_PCIS_READ_CHIRHO) ? pcis_rready_chirho :
                               (arb_state_chirho == ARB_FSM_READ_CHIRHO) ? fsm_rready_chirho : 1'b0;

endmodule
