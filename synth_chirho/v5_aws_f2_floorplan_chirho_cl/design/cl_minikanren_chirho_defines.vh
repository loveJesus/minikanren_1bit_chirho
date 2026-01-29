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
  // Hierarchical Domain Mode Selection
  // ========================================================================
  // Modes select domain size for different problem scales
  // Larger domains support more values per variable but require more HBM reads

  `define HIER_MODE_FLAT256_CHIRHO     3'd0  // 256 values (1 HBM beat)
  `define HIER_MODE_HIER_65K_CHIRHO    3'd1  // 256² = 65K values (256 beats + 1 summary)
  `define HIER_MODE_HIER_262K_CHIRHO   3'd2  // 512² = 262K values (512 beats + 1 summary)
  `define HIER_MODE_HIER_16M_CHIRHO    3'd3  // 256³ = 16.7M values (~8K beats) ← SWEET SPOT
  `define HIER_MODE_HIER_134M_CHIRHO   3'd4  // 512³ = 134M values (~65K beats)

  // Register addresses (OCL AXI-Lite, word-aligned)
  `define REG_VERSION_CHIRHO      8'h00  // Read-only version
  `define REG_CONTROL_CHIRHO      8'h04  // bit0=enable, bit1=reset, bit2=hbm_mode
  `define REG_STATUS_CHIRHO       8'h08  // bit0=done, bit1=valid, bit2=hbm_ready
  `define REG_CMD_LO_CHIRHO       8'h10  // cmdChirho[31:0]
  `define REG_CMD_MID_CHIRHO      8'h14  // cmdChirho[63:32]
  `define REG_CMD_HI_CHIRHO       8'h18  // cmdChirho[69:64]
  `define REG_HIER_MODE_CHIRHO    8'h40  // Hierarchical mode selection (3 bits)
  `define REG_HIER_LEVEL_CHIRHO   8'h44  // Current hierarchy level (debug)
  `define REG_BEAT_COUNT_CHIRHO   8'h48  // HBM beat counter (debug)
  `define REG_RESP_BASE_CHIRHO    8'h20  // Response registers start

  // Domain sizes per hierarchy mode (bytes per variable domain)
  `define DOMAIN_SIZE_FLAT256_CHIRHO    32      // 256 bits = 32 bytes
  `define DOMAIN_SIZE_HIER_65K_CHIRHO   8224    // 256² bits + summary = ~8KB
  `define DOMAIN_SIZE_HIER_262K_CHIRHO  33024   // 512² bits + summary = ~33KB
  `define DOMAIN_SIZE_HIER_16M_CHIRHO   2097408 // 256³ bits + summaries = ~2MB
  `define DOMAIN_SIZE_HIER_134M_CHIRHO  16810496 // 512³ bits + summaries = ~16MB

  // HBM beats required per domain (256-bit beats)
  `define BEATS_FLAT256_CHIRHO      1
  `define BEATS_HIER_65K_CHIRHO     258    // 256 data + 1 level1 + 1 level0
  `define BEATS_HIER_262K_CHIRHO    1026   // 512 × 512/256 + summaries
  `define BEATS_HIER_16M_CHIRHO     65794  // 256³/256 + summaries
  `define BEATS_HIER_134M_CHIRHO    525314 // 512³/256 + summaries

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

  // Probabilistic domain format (for intersect_prob_domain_64_chirho):
  //   [1087:1024] = 64-bit presence mask (ANDed like Boolean)
  //   [1023:0]    = 64 × 16-bit Q8.8 probabilities

`endif
