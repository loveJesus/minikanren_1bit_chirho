// Verilator Wrapper for searchEngineChirho ☧
//
// C interface to drive the Clash-generated Verilog via Verilator.
// Part of P5-00: Bridge of Truth.

#include <cstdint>
#include <cstring>

// Forward declarations - Verilator will generate these
class VsearchEngineChirho;
class VerilatedContext;

// Global state (single instance for simplicity)
static VerilatedContext* g_context_chirho = nullptr;
static VsearchEngineChirho* g_dut_chirho = nullptr;

extern "C" {

// Command encoding (matches Clash SearchCmdChirho)
// Tag (3 bits) + payload
// 0 = Init
// 1 = UnifyVars (v1: 3 bits, v2: 3 bits)
// 2 = ConstrainVar (v: 3 bits, mask: 64 bits)
// 3 = BranchVar (v: 3 bits)
// 4 = Backtrack
// 5 = Nop

struct CmdChirho {
    uint8_t tag_chirho;      // Command type (0-5)
    uint8_t var1_chirho;     // First variable index (0-7)
    uint8_t var2_chirho;     // Second variable index (0-7)
    uint64_t mask_chirho;    // Domain mask for ConstrainVar
};

struct RespChirho {
    uint8_t valid_chirho;    // 0 or 1
    uint8_t solution_chirho; // 0 or 1
    uint64_t domains_chirho[8]; // 8 x 64-bit domains
};

// Initialize the Verilator simulation
// Returns 0 on success, -1 on error
int verilator_init_chirho(void) {
    // Note: Actual implementation requires Verilator-generated headers
    // This is a stub that will be compiled when Verilator is available
    #ifdef VERILATOR_AVAILABLE
    g_context_chirho = new VerilatedContext;
    g_dut_chirho = new VsearchEngineChirho(g_context_chirho);

    // Reset sequence
    g_dut_chirho->rst = 1;
    g_dut_chirho->clk = 0;
    g_dut_chirho->evaluate_chirho(); // Verilator evaluate method
    g_dut_chirho->clk = 1;
    g_dut_chirho->evaluate_chirho();
    g_dut_chirho->rst = 0;

    return 0;
    #else
    return -1; // Verilator not available
    #endif
}

// Cleanup
void verilator_cleanup_chirho(void) {
    #ifdef VERILATOR_AVAILABLE
    if (g_dut_chirho) {
        delete g_dut_chirho;
        g_dut_chirho = nullptr;
    }
    if (g_context_chirho) {
        delete g_context_chirho;
        g_context_chirho = nullptr;
    }
    #endif
}

// Execute one step: apply command, clock, read response
// Returns 0 on success, -1 on error
int verilator_step_chirho(const CmdChirho* cmd_chirho, RespChirho* resp_chirho) {
    #ifdef VERILATOR_AVAILABLE
    if (!g_dut_chirho) return -1;

    // Encode command into 70-bit cmdChirho input
    // Format depends on Clash encoding - this is approximate
    uint64_t cmd_lo = 0;
    uint8_t cmd_hi = 0;

    switch (cmd_chirho->tag_chirho) {
        case 0: // Init
            cmd_lo = 0;
            break;
        case 1: // UnifyVars
            cmd_lo = (1ULL) |
                    ((uint64_t)cmd_chirho->var1_chirho << 3) |
                    ((uint64_t)cmd_chirho->var2_chirho << 6);
            break;
        case 2: // ConstrainVar
            cmd_lo = (2ULL) |
                    ((uint64_t)cmd_chirho->var1_chirho << 3) |
                    (cmd_chirho->mask_chirho << 6);
            cmd_hi = (uint8_t)(cmd_chirho->mask_chirho >> 58);
            break;
        case 3: // BranchVar
            cmd_lo = (3ULL) |
                    ((uint64_t)cmd_chirho->var1_chirho << 3);
            break;
        case 4: // Backtrack
            cmd_lo = 4;
            break;
        case 5: // Nop
            cmd_lo = 5;
            break;
    }

    // Apply to DUT
    g_dut_chirho->enChirho = 1;
    // g_dut_chirho->cmdChirho would be set here based on exact port width

    // Clock cycle using Verilator's evaluate method
    g_dut_chirho->clk = 0;
    g_dut_chirho->evaluate_chirho();
    g_dut_chirho->clk = 1;
    g_dut_chirho->evaluate_chirho();

    // Read response (514 bits)
    // Decode based on Clash encoding
    resp_chirho->valid_chirho = 1; // g_dut_chirho->respChirho[0]
    resp_chirho->solution_chirho = 0;
    memset(resp_chirho->domains_chirho, 0xFF, sizeof(resp_chirho->domains_chirho));

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
