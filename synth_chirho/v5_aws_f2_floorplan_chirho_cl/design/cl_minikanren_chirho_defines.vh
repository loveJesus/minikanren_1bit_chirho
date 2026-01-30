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

  // Version register (F2 = 0xF2, 55 = V5.5, version = 0x0001)
  // V5.3: 0xF2020001, V5.4: 0xF2540001, V5.5: 0xF2550001
  `define MINIKANREN_VERSION_CHIRHO 32'hF2_55_0001

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

  // HBM Memory Layout (custom addresses for miniKanren)
  `define HBM_VAR_HEADERS_BASE_CHIRHO  34'h0_0000_0000  // 0-512MB: Variable headers
  `define HBM_VAR_DOMAINS_BASE_CHIRHO  34'h0_2000_0000  // 512MB-8.5GB: Variable domains
  `define HBM_TERM_STORE_BASE_CHIRHO   34'h1_0000_0000  // 4GB-8GB: Term store
  `define HBM_HASH_TABLE_BASE_CHIRHO   34'h2_0000_0000  // 8GB-8.25GB: Hash table
  `define HBM_TABLING_CACHE_CHIRHO     34'h2_1000_0000  // 8.25GB-10.25GB: Tabling cache

  // Domain operation codes
  `define OP_INTERSECT_CHIRHO   4'h0  // Unification (AND)
  `define OP_UNION_CHIRHO       4'h1  // Disjunction (OR)
  `define OP_COMPLEMENT_CHIRHO  4'h2  // Negation (NOT)
  `define OP_IS_GROUND_CHIRHO   4'h3  // Check if single value

  // ========================================================================
  // Hierarchical Domain Mode Selection (V5: flat + 65K only)
  // ========================================================================
  // V5 Scope: Only flat256 and 65K modes are fully implemented
  // - flat256: 256 values (1 HBM beat) - simplest mode
  // - 65K: 256² = 65,536 values (257 beats) - max complexity for V5
  //
  // REMOVED in V5 (timing closure / implementation incomplete):
  // - 262K (512²): requires 512-bit word assembly from 256-bit bus
  // - 1M (1024²): requires 1024-bit word assembly from 256-bit bus
  // - 16M (256³): streaming FSM caused -6.256ns WNS at 200MHz
  // - 134M (512³): streaming FSM not implemented

  `define HIER_MODE_FLAT256_CHIRHO     3'd0  // 256 values (1 HBM beat) ✓ V5
  `define HIER_MODE_HIER_65K_CHIRHO    3'd1  // 256² = 65K values (257 beats) ✓ V5
  `define HIER_MODE_HIER_262K_CHIRHO   3'd2  // 512² = 262K values (1026 beats) ✓ V5
  // Modes 3+ defined for future use but NOT implemented in V5 RTL
  `define HIER_MODE_HIER_1M_CHIRHO     3'd3  // TODO: V6
  // `define HIER_MODE_HIER_16M_CHIRHO  3'd3  // Replaced by 1M
  // `define HIER_MODE_HIER_134M_CHIRHO 3'd4  // Removed (streaming complexity)

  // Register addresses (OCL AXI-Lite, word-aligned)
  `define REG_VERSION_CHIRHO      8'h00  // Read-only version
  `define REG_CONTROL_CHIRHO      8'h04  // bit0=enable, bit1=reset, bit2=hbm_mode
  `define REG_STATUS_CHIRHO       8'h08  // bit0=done, bit1=valid, bit2=hbm_ready
  `define REG_CMD_LO_CHIRHO       8'h10  // cmdChirho[31:0]
  `define REG_CMD_MID_CHIRHO      8'h14  // cmdChirho[63:32]
  `define REG_CMD_HI_CHIRHO       8'h18  // cmdChirho[69:64]
  // V5.4: Moved HIER registers to 0x80+ to avoid conflict with RESP (0x20-0x5C)
  `define REG_HIER_MODE_CHIRHO    8'h80  // Hierarchical mode selection (3 bits)
  // REG_HIER_LEVEL and REG_BEAT_COUNT moved to debug section at 0xC0+
  `define REG_RESP_BASE_CHIRHO    8'h20  // Response registers start

  // Domain sizes per hierarchy mode (bytes per variable domain)
  `define DOMAIN_SIZE_FLAT256_CHIRHO    32      // 256 bits = 32 bytes ✓ V5
  `define DOMAIN_SIZE_HIER_65K_CHIRHO   8224    // 256² bits + summary = ~8KB ✓ V5
  `define DOMAIN_SIZE_HIER_262K_CHIRHO  33024   // 512² bits + summary = ~33KB ✓ V5
  // Larger sizes defined for future use but NOT implemented in V5
  `define DOMAIN_SIZE_HIER_1M_CHIRHO    131200  // TODO: V6

  // HBM beats required per domain (256-bit beats)
  `define BEATS_FLAT256_CHIRHO      1     // ✓ V5
  `define BEATS_HIER_65K_CHIRHO     257   // 1 level0 + 256 level1 ✓ V5
  `define BEATS_HIER_262K_CHIRHO    1026  // 2 beats for level0 + 512×2 for level1 ✓ V5
  // Larger hierarchies require 4× word assembly
  `define BEATS_HIER_1M_CHIRHO      4100  // TODO: V6

  // ========================================================================
  // Neurosymbolic Training Mode
  // ========================================================================
  // When training mode is enabled, the engine runs gradient descent on
  // Q16.16 fixed-point weights using Gumbel-softmax reparameterization

  `define REG_TRAIN_MODE_CHIRHO     8'h50  // bit0=train_enable, bit1=train_reset
  `define REG_TRAIN_CMD_LO_CHIRHO   8'h54  // train_cmd[31:0]
  `define REG_TRAIN_CMD_MID_CHIRHO  8'h58  // train_cmd[63:32]
  `define REG_TRAIN_CMD_HI_CHIRHO   8'h5C  // train_cmd[95:64]
  `define REG_TRAIN_CMD_TOP_CHIRHO  8'h60  // train_cmd[127:96]
  `define REG_TRAIN_RESP_LO_CHIRHO  8'h64  // train_resp[31:0]
  `define REG_TRAIN_RESP_HI_CHIRHO  8'h68  // train_resp[63:32]

  // Training FSM states (from DiffTrainChirho.hs)
  `define TRAIN_STATE_IDLE_CHIRHO          3'd0
  `define TRAIN_STATE_LOAD_WEIGHTS_CHIRHO  3'd1
  `define TRAIN_STATE_SAMPLE_CHIRHO        3'd2
  `define TRAIN_STATE_EVAL_CLAUSES_CHIRHO  3'd3
  `define TRAIN_STATE_ACCUM_GRADS_CHIRHO   3'd4
  `define TRAIN_STATE_UPDATE_CHIRHO        3'd5
  `define TRAIN_STATE_DONE_CHIRHO          3'd6
  `define TRAIN_STATE_RESET_CHIRHO         3'd7

  // ========================================================================
  // Inference Mode (Boolean vs Probabilistic)
  // ========================================================================
  // Boolean mode: Standard bitwise AND for domain intersection
  // Probabilistic mode: Q16.16 or Q8.8 soft AND for differentiable inference

  `define REG_INFER_MODE_CHIRHO     8'h70  // bit0=prob_mode (0=Boolean, 1=Probabilistic)
  `define INFER_MODE_BOOL_CHIRHO    1'b0   // Standard Boolean intersection
  `define INFER_MODE_PROB_CHIRHO    1'b1   // Probabilistic soft intersection (Q8.8/Q16.16)

  // ========================================================================
  // V5.4 Debug Registers (read-only, zero cost visibility)
  // ========================================================================
  `define REG_FSM_STATE_CHIRHO      8'hC0  // Current FSM state (5 bits)
  `define REG_AXI_STATUS_CHIRHO     8'hC4  // AXI signals: {bready,bvalid,awvalid,awready,rready,rvalid,arvalid,arready}
  `define REG_AXI_ADDR_LO_CHIRHO    8'hC8  // axi_addr_chirho[31:0]
  `define REG_AXI_ADDR_HI_CHIRHO    8'hCC  // axi_addr_chirho[33:32]
  `define REG_BEAT_COUNT_CHIRHO     8'hD0  // beat_counter_chirho[19:0]
  `define REG_SPARSE_IDX_CHIRHO     8'hD4  // sparse_idx_chirho[8:0]

  // Probabilistic domain format (for intersect_prob_domain_64_chirho):
  //   [1087:1024] = 64-bit presence mask (ANDed like Boolean)
  //   [1023:0]    = 64 × 16-bit Q8.8 probabilities

`endif
