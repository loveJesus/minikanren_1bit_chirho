// ============================================================================
// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// miniKanren F2 Custom Logic Defines with HBM Support
// ============================================================================

`ifndef CL_MINIKANREN_CHIRHO_DEFINES
`define CL_MINIKANREN_CHIRHO_DEFINES

  // CL Name for F2 shell instantiation
  `define CL_NAME cl_minikanren_chirho

  // Version register (F2 = 0xF2, HBM = 0x02, version = 0x0001)
  `define MINIKANREN_VERSION_CHIRHO 32'hF2_02_0001

  // Enable HBM support (recommended for async operations)
  `define FPGA_LESS_RST

  // Uncomment to use async shell-CL interface
  // `define SH_CL_ASYNC

  // Default AXI values for HBM interface
  `define DEF_AXSIZE    3'd5   // 32 Bytes per beat (256 bits)
  `define DEF_AXBURST   2'd1   // INCR burst
  `define DEF_AXCACHE   4'd3   // Bufferable, Modifiable
  `define DEF_AXLOCK    1'd0   // Normal access
  `define DEF_AXPROT    3'd2   // Unprivileged access, Non-Secure Access
  `define DEF_AXQOS     4'd0   // Regular Identifier
  `define DEF_AXREGION  4'd0   // Single region

  // HBM Memory Layout
  `define HBM_VAR_HEADERS_BASE  34'h0_0000_0000  // 0-512MB: Variable headers
  `define HBM_VAR_DOMAINS_BASE  34'h0_2000_0000  // 512MB-8.5GB: Variable domains
  `define HBM_TERM_STORE_BASE   34'h1_0000_0000  // 4GB-8GB: Term store
  `define HBM_HASH_TABLE_BASE   34'h2_0000_0000  // 8GB-8.25GB: Hash table
  `define HBM_TABLING_CACHE     34'h2_1000_0000  // 8.25GB-10.25GB: Tabling cache

  // Domain operation codes
  `define OP_INTERSECT_CHIRHO   4'h0  // Unification (AND)
  `define OP_UNION_CHIRHO       4'h1  // Disjunction (OR)
  `define OP_COMPLEMENT_CHIRHO  4'h2  // Negation (NOT)
  `define OP_IS_GROUND_CHIRHO   4'h3  // Check if single value

`endif
