// Verilator Wrapper for searchEngineChirho ☧
//
// C interface to drive the Clash-generated Verilog via Verilator.
// Part of P5-00: Bridge of Truth.
//
// "Sanctify them through thy truth: thy word is truth." — John 17:17

#include <cstdint>
#include <cstring>

#ifdef VERILATOR_AVAILABLE
#include "VsearchEngineChirho.h"
#include "verilated.h"
#endif

// Global state (single instance for simplicity)
#ifdef VERILATOR_AVAILABLE
static VerilatedContext* g_context_chirho = nullptr;
static VsearchEngineChirho* g_dut_chirho = nullptr;
#endif

extern "C" {

// Command encoding (matches Rust SearchCmdChirho and Clash cmdChirho)
// cmdChirho[69:67] = tag (3 bits)
// cmdChirho[66:64] = var1 (3 bits)
// cmdChirho[63:61] = var2 (3 bits, for UnifyVars)
// cmdChirho[63:0]  = mask (64 bits, for ConstrainVar)
//
// Tags:
//   0 = Init
//   1 = UnifyVars (v1, v2)
//   2 = ConstrainVar (v, mask)
//   3 = BranchVar (v)
//   4 = Backtrack
//   5 = Nop

struct CmdChirho {
    uint8_t tag_chirho;      // Command type (0-5)
    uint8_t var1_chirho;     // First variable index (0-7)
    uint8_t var2_chirho;     // Second variable index (0-7)
    uint64_t mask_chirho;    // Domain mask for ConstrainVar
};

// Response encoding (matches Rust SearchRespChirho and Clash respChirho)
// respChirho[513]    = valid (1 bit)
// respChirho[512]    = solution (1 bit) - valid AND all domains singleton
// respChirho[511:0]  = domains (8 × 64 bits)
struct RespChirho {
    uint8_t valid_chirho;       // 0 or 1
    uint8_t solution_chirho;    // 0 or 1
    uint64_t domains_chirho[8]; // 8 × 64-bit domains
};

// Initialize the Verilator simulation
// Returns 0 on success, -1 on error
int verilator_init_chirho(void) {
#ifdef VERILATOR_AVAILABLE
    if (g_context_chirho != nullptr) {
        // Already initialized
        return 0;
    }

    g_context_chirho = new VerilatedContext;
    g_context_chirho->commandArgs(0, nullptr);

    g_dut_chirho = new VsearchEngineChirho(g_context_chirho);

    // Reset sequence: hold rst high for a few cycles
    g_dut_chirho->rst = 1;
    g_dut_chirho->enChirho = 0;
    g_dut_chirho->cmdChirho[0] = 0;
    g_dut_chirho->cmdChirho[1] = 0;
    g_dut_chirho->cmdChirho[2] = 0;

    // Clock a few times while in reset
    for (int i = 0; i < 5; i++) {
        g_dut_chirho->clk = 0;
        g_dut_chirho->eval();
        g_dut_chirho->clk = 1;
        g_dut_chirho->eval();
    }

    // Release reset
    g_dut_chirho->rst = 0;
    g_dut_chirho->clk = 0;
    g_dut_chirho->eval();
    g_dut_chirho->clk = 1;
    g_dut_chirho->eval();

    return 0;
#else
    return -1; // Verilator not available
#endif
}

// Cleanup
void verilator_cleanup_chirho(void) {
#ifdef VERILATOR_AVAILABLE
    if (g_dut_chirho) {
        g_dut_chirho->final();
        delete g_dut_chirho;
        g_dut_chirho = nullptr;
    }
    if (g_context_chirho) {
        delete g_context_chirho;
        g_context_chirho = nullptr;
    }
#endif
}

// Encode command into 70-bit cmdChirho (3 × 32-bit words = 96 bits, use lower 70)
static void encode_cmd_chirho(const CmdChirho* cmd_chirho, uint32_t out_chirho[3]) {
    // cmdChirho[69:67] = tag (3 bits)
    // cmdChirho[66:64] = var1 (3 bits)
    // cmdChirho[63:61] = var2 (3 bits) [for UnifyVars]
    // cmdChirho[63:0]  = mask (64 bits) [for ConstrainVar]
    //
    // Bit layout (little-endian words):
    //   out[0] = bits 31:0   = mask[31:0]
    //   out[1] = bits 63:32  = mask[63:32]
    //   out[2] = bits 69:64  = tag[2:0] << 3 | var1[2:0] | (var2 for UnifyVars)

    uint64_t cmd70_chirho = 0;

    switch (cmd_chirho->tag_chirho) {
        case 0: // Init
            cmd70_chirho = ((uint64_t)0 << 67);
            break;
        case 1: // UnifyVars(v1, v2)
            cmd70_chirho = ((uint64_t)1 << 67) |
                           ((uint64_t)(cmd_chirho->var1_chirho & 0x7) << 64) |
                           ((uint64_t)(cmd_chirho->var2_chirho & 0x7) << 61);
            break;
        case 2: // ConstrainVar(v, mask)
            cmd70_chirho = ((uint64_t)2 << 67) |
                           ((uint64_t)(cmd_chirho->var1_chirho & 0x7) << 64) |
                           (cmd_chirho->mask_chirho & 0xFFFFFFFFFFFFFFFFULL);
            break;
        case 3: // BranchVar(v)
            cmd70_chirho = ((uint64_t)3 << 67) |
                           ((uint64_t)(cmd_chirho->var1_chirho & 0x7) << 64);
            break;
        case 4: // Backtrack
            cmd70_chirho = ((uint64_t)4 << 67);
            break;
        case 5: // Nop
        default:
            cmd70_chirho = ((uint64_t)5 << 67);
            break;
    }

    // Split into 32-bit words for Verilator
    out_chirho[0] = (uint32_t)(cmd70_chirho & 0xFFFFFFFF);
    out_chirho[1] = (uint32_t)((cmd70_chirho >> 32) & 0xFFFFFFFF);
    out_chirho[2] = (uint32_t)((cmd70_chirho >> 64) & 0x3F); // Only 6 bits in top word
}

// Decode 514-bit respChirho into RespChirho struct
static void decode_resp_chirho(const uint32_t in_chirho[17], RespChirho* resp_chirho) {
    // respChirho[513]   = valid
    // respChirho[512]   = solution
    // respChirho[511:0] = domains[8][64]
    //
    // 514 bits = 16 × 32 + 2 = 17 words (last word has 2 bits)
    //
    // Bit layout:
    //   in[0]  = domains[0][31:0]
    //   in[1]  = domains[0][63:32]
    //   in[2]  = domains[1][31:0]
    //   ...
    //   in[15] = domains[7][63:32]
    //   in[16] = {30'b0, solution, valid} - bits 513:512

    // Extract domains (little-endian)
    for (int i = 0; i < 8; i++) {
        uint64_t lo_chirho = in_chirho[i * 2];
        uint64_t hi_chirho = in_chirho[i * 2 + 1];
        resp_chirho->domains_chirho[i] = lo_chirho | (hi_chirho << 32);
    }

    // Extract valid and solution from top word
    uint32_t top_chirho = in_chirho[16];
    resp_chirho->valid_chirho = (top_chirho >> 1) & 1;    // Bit 513
    resp_chirho->solution_chirho = top_chirho & 1;         // Bit 512
}

// Execute one step: apply command, clock, read response
// Returns 0 on success, -1 on error
int verilator_step_chirho(const CmdChirho* cmd_chirho, RespChirho* resp_chirho) {
#ifdef VERILATOR_AVAILABLE
    if (!g_dut_chirho) return -1;

    // Encode command
    uint32_t cmd_encoded_chirho[3];
    encode_cmd_chirho(cmd_chirho, cmd_encoded_chirho);

    // Apply command
    g_dut_chirho->enChirho = 1;
    g_dut_chirho->cmdChirho[0] = cmd_encoded_chirho[0];
    g_dut_chirho->cmdChirho[1] = cmd_encoded_chirho[1];
    g_dut_chirho->cmdChirho[2] = cmd_encoded_chirho[2];

    // Rising edge
    g_dut_chirho->clk = 0;
    g_dut_chirho->eval();
    g_dut_chirho->clk = 1;
    g_dut_chirho->eval();

    // Read response (514 bits = 17 words)
    // Verilator stores wide signals as arrays of uint32_t
    uint32_t resp_raw_chirho[17];
    for (int i = 0; i < 16; i++) {
        resp_raw_chirho[i] = g_dut_chirho->respChirho[i];
    }
    resp_raw_chirho[16] = g_dut_chirho->respChirho[16];

    decode_resp_chirho(resp_raw_chirho, resp_chirho);

    return 0;
#else
    // Stub: return error when Verilator not available
    (void)cmd_chirho;
    (void)resp_chirho;
    return -1;
#endif
}

// Check if Verilator is available
int verilator_available_chirho(void) {
#ifdef VERILATOR_AVAILABLE
    return 1;
#else
    return 0;
#endif
}

} // extern "C"
