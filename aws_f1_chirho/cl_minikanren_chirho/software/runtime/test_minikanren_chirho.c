/**
 * ============================================================================
 * ☧ For God so loved the world, that He gave His only begotten Son,
 * that whosoever believeth in Him should not perish, but have everlasting life.
 * - John 3:16
 * ============================================================================
 *
 * AWS F1 Runtime Test for miniKanren Search Engine
 *
 * This program tests the miniKanren FPGA accelerator by:
 *   1. Reading the version register
 *   2. Sending a simple unification command
 *   3. Reading the response
 *
 * Build: gcc -o test_minikanren_chirho test_minikanren_chirho.c -lfpga_mgmt
 * Run:   sudo ./test_minikanren_chirho
 * ============================================================================
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>

#include <fpga_pci.h>
#include <fpga_mgmt.h>

/* Register offsets */
#define REG_VERSION_CHIRHO   0x000
#define REG_CONTROL_CHIRHO   0x004
#define REG_STATUS_CHIRHO    0x008
#define REG_CMD_LO_CHIRHO    0x010
#define REG_CMD_MID_CHIRHO   0x014
#define REG_CMD_HI_CHIRHO    0x018
#define REG_RESP_BASE_CHIRHO 0x020

/* Control register bits */
#define CTRL_ENABLE_CHIRHO  (1 << 0)
#define CTRL_RESET_CHIRHO   (1 << 1)

/* Command opcodes (from MiniKanrenChirho.hs) */
#define CMD_RESET_CHIRHO    0  /* Reset state */
#define CMD_UNIFY_CHIRHO    1  /* Unify two variables */
#define CMD_APPLY_CHIRHO    2  /* Apply mask to variable */
#define CMD_PUSH_CHIRHO     3  /* Push branch point */
#define CMD_POP_CHIRHO      4  /* Pop and restore */

/* FPGA slot (usually 0 for single FPGA) */
static int slot_id_chirho = 0;

/* PCI BAR handle (external AWS SDK type - naming exception) */
static pci_bar_handle_t pci_bar_handle_chirho = PCI_BAR_HANDLE_INIT;

/**
 * Initialize FPGA access
 */
int init_fpga_chirho(void) {
    int rc_chirho;

    /* Initialize FPGA management library */
    rc_chirho = fpga_mgmt_init();
    if (rc_chirho) {
        printf("ERROR: fpga_mgmt_init failed: %d\n", rc_chirho);
        return rc_chirho;
    }

    /* Attach to the FPGA's PCIe BAR */
    rc_chirho = fpga_pci_attach(slot_id_chirho, FPGA_APP_PF, APP_PF_BAR0, 0, &pci_bar_handle_chirho);
    if (rc_chirho) {
        printf("ERROR: fpga_pci_attach failed: %d\n", rc_chirho);
        return rc_chirho;
    }

    printf("FPGA attached successfully (slot %d)\n", slot_id_chirho);
    return 0;
}

/**
 * Cleanup FPGA access
 */
void cleanup_fpga_chirho(void) {
    fpga_pci_detach(pci_bar_handle_chirho);
    fpga_mgmt_close();
}

/**
 * Read a 32-bit register
 */
uint32_t read_reg_chirho(uint32_t offset_chirho) {
    uint32_t value_chirho = 0;
    int rc_chirho = fpga_pci_peek(pci_bar_handle_chirho, offset_chirho, &value_chirho);
    if (rc_chirho) {
        printf("ERROR: fpga_pci_peek(0x%03x) failed: %d\n", offset_chirho, rc_chirho);
    }
    return value_chirho;
}

/**
 * Write a 32-bit register
 */
void write_reg_chirho(uint32_t offset_chirho, uint32_t value_chirho) {
    int rc_chirho = fpga_pci_poke(pci_bar_handle_chirho, offset_chirho, value_chirho);
    if (rc_chirho) {
        printf("ERROR: fpga_pci_poke(0x%03x, 0x%08x) failed: %d\n", offset_chirho, value_chirho, rc_chirho);
    }
}

/**
 * Send a command to the miniKanren engine
 *
 * Command format (70 bits):
 *   [69:67] = opcode (3 bits)
 *   [66:64] = var1 (3 bits)
 *   [63:61] = var2 (3 bits) - for unify
 *   [63:0]  = mask (64 bits) - for apply
 */
void send_cmd_chirho(uint8_t opcode_chirho, uint8_t var1_chirho, uint8_t var2_chirho, uint64_t mask_chirho) {
    uint32_t cmd_lo_chirho  = mask_chirho & 0xFFFFFFFF;
    uint32_t cmd_mid_chirho = (mask_chirho >> 32) & 0xFFFFFFFF;
    uint32_t cmd_hi_chirho  = ((opcode_chirho & 0x7) << 3) | (var1_chirho & 0x7);

    /* For unify, var2 goes in the upper bits of the mask position */
    if (opcode_chirho == CMD_UNIFY_CHIRHO) {
        cmd_mid_chirho = (cmd_mid_chirho & 0x1FFFFFFF) | ((var2_chirho & 0x7) << 29);
    }

    write_reg_chirho(REG_CMD_LO_CHIRHO, cmd_lo_chirho);
    write_reg_chirho(REG_CMD_MID_CHIRHO, cmd_mid_chirho);
    write_reg_chirho(REG_CMD_HI_CHIRHO, cmd_hi_chirho);
}

/**
 * Read response from the miniKanren engine
 *
 * Response format (514 bits):
 *   [513:512] = flags (2 bits)
 *   [511:0]   = 8 x 64-bit domain vectors
 */
void read_resp_chirho(uint64_t domains_chirho[8], uint8_t *flags_chirho) {
    uint32_t words_chirho[17];

    /* Read all 17 words of response */
    for (int i_chirho = 0; i_chirho < 17; i_chirho++) {
        words_chirho[i_chirho] = read_reg_chirho(REG_RESP_BASE_CHIRHO + i_chirho * 4);
    }

    /* Reconstruct 64-bit domains */
    for (int i_chirho = 0; i_chirho < 8; i_chirho++) {
        domains_chirho[i_chirho] = ((uint64_t)words_chirho[i_chirho*2 + 1] << 32) | words_chirho[i_chirho*2];
    }

    *flags_chirho = words_chirho[16] & 0x3;
}

/**
 * Print domain as binary (showing set bits)
 */
void print_domain_chirho(const char *name_chirho, uint64_t domain_chirho) {
    printf("  %s: 0x%016lx (popcount=%d)\n", name_chirho, domain_chirho, __builtin_popcountll(domain_chirho));

    /* Show first few set values */
    if (domain_chirho != 0 && domain_chirho != 0xFFFFFFFFFFFFFFFFULL) {
        printf("       Values: ");
        int count_chirho = 0;
        for (int i_chirho = 0; i_chirho < 64 && count_chirho < 8; i_chirho++) {
            if (domain_chirho & (1ULL << i_chirho)) {
                printf("%d ", i_chirho);
                count_chirho++;
            }
        }
        if (__builtin_popcountll(domain_chirho) > 8) {
            printf("...");
        }
        printf("\n");
    }
}

/**
 * Main test program
 */
int main(int argc_chirho, char *argv_chirho[]) {
    int rc_chirho;
    (void)argc_chirho;
    (void)argv_chirho;

    printf("\n");
    printf("=================================================\n");
    printf("  miniKanren FPGA Accelerator Test ☧\n");
    printf("=================================================\n\n");

    /* Initialize FPGA */
    rc_chirho = init_fpga_chirho();
    if (rc_chirho) {
        return 1;
    }

    /* Read version register */
    uint32_t version_chirho = read_reg_chirho(REG_VERSION_CHIRHO);
    printf("Version: 0x%08x (v%d.%d.%d)\n\n",
           version_chirho,
           (version_chirho >> 16) & 0xFF,
           (version_chirho >> 8) & 0xFF,
           version_chirho & 0xFF);

    /* Reset the engine */
    printf("Resetting engine...\n");
    write_reg_chirho(REG_CONTROL_CHIRHO, CTRL_RESET_CHIRHO);
    usleep(1000);
    write_reg_chirho(REG_CONTROL_CHIRHO, 0);

    /* Enable the engine */
    printf("Enabling engine...\n");
    write_reg_chirho(REG_CONTROL_CHIRHO, CTRL_ENABLE_CHIRHO);

    /* Test 1: Reset to full domains */
    printf("\n--- Test 1: Reset (all domains = full) ---\n");
    send_cmd_chirho(CMD_RESET_CHIRHO, 0, 0, 0);
    usleep(100);

    uint64_t domains_chirho[8];
    uint8_t flags_chirho;
    read_resp_chirho(domains_chirho, &flags_chirho);

    printf("Flags: 0x%x (valid=%d, done=%d)\n", flags_chirho, flags_chirho & 1, (flags_chirho >> 1) & 1);
    for (int i_chirho = 0; i_chirho < 8; i_chirho++) {
        char name_chirho[8];
        snprintf(name_chirho, sizeof(name_chirho), "v%d", i_chirho);
        print_domain_chirho(name_chirho, domains_chirho[i_chirho]);
    }

    /* Test 2: Apply mask to v0 */
    printf("\n--- Test 2: Apply mask 0x0F to v0 ---\n");
    send_cmd_chirho(CMD_APPLY_CHIRHO, 0, 0, 0x0FULL);
    usleep(100);

    read_resp_chirho(domains_chirho, &flags_chirho);
    print_domain_chirho("v0", domains_chirho[0]);

    /* Test 3: Apply mask to v1 */
    printf("\n--- Test 3: Apply mask 0x1E to v1 ---\n");
    send_cmd_chirho(CMD_APPLY_CHIRHO, 1, 0, 0x1EULL);
    usleep(100);

    read_resp_chirho(domains_chirho, &flags_chirho);
    print_domain_chirho("v1", domains_chirho[1]);

    /* Test 4: Unify v0 and v1 */
    printf("\n--- Test 4: Unify v0 and v1 ---\n");
    send_cmd_chirho(CMD_UNIFY_CHIRHO, 0, 1, 0);
    usleep(100);

    read_resp_chirho(domains_chirho, &flags_chirho);
    printf("After unification:\n");
    print_domain_chirho("v0", domains_chirho[0]);
    print_domain_chirho("v1", domains_chirho[1]);
    printf("Expected: v0 = v1 = 0x0F & 0x1E = 0x0E\n");

    /* Disable engine */
    write_reg_chirho(REG_CONTROL_CHIRHO, 0);

    printf("\n=================================================\n");
    printf("  Test Complete ☧\n");
    printf("=================================================\n\n");

    cleanup_fpga_chirho();
    return 0;
}
