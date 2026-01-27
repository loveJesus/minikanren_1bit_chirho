// ============================================================================
// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// DMA/DDR Defines for miniKanren HBM Integration
// (Minimal version with only what's needed for HBM support)
// ============================================================================

`ifndef CL_DRAM_DMA_DEFINES
`define CL_DRAM_DMA_DEFINES

  // Put module name of the CL design here
  `define CL_NAME cl_minikanren_chirho

  // Recommended for reduced reset routing
  `define FPGA_LESS_RST

  // DDR controllers - all disabled for HBM-only design
  `define DDR_A_PRESENT 0
  `define DDR_B_PRESENT 0
  `define DDR_D_PRESENT 0

  // Default AXI values (used by HBM wrappers)
  `ifndef DEF_AXSIZE
    `define DEF_AXSIZE    3'd5   // 32 Bytes per beat
  `endif
  `ifndef DEF_AXBURST
    `define DEF_AXBURST   2'd1   // INCR burst
  `endif
  `ifndef DEF_AXCACHE
    `define DEF_AXCACHE   4'd3   // Bufferable, Modifiable
  `endif
  `ifndef DEF_AXLOCK
    `define DEF_AXLOCK    1'd0   // Normal access
  `endif
  `ifndef DEF_AXPROT
    `define DEF_AXPROT    3'd2   // Unprivileged access, Non-Secure
  `endif
  `ifndef DEF_AXQOS
    `define DEF_AXQOS     4'd0   // Regular Identifier
  `endif
  `ifndef DEF_AXREGION
    `define DEF_AXREGION  4'd0   // Single region
  `endif

`endif
