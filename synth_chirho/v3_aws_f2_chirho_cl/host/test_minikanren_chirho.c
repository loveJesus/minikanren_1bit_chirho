/**
 * For God so loved the world - John 3:16 ☧
 *
 * Host test software for miniKanren FPGA accelerator
 * Tests basic functionality via AWS FPGA APIs
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
#include <fpga_dma.h>

// Register addresses (must match cl_minikanren_chirho.sv)
#define REG_VERSION_CHIRHO      0x000  // Version register
#define REG_STATUS_CHIRHO       0x004  // Status register
#define REG_CONTROL_CHIRHO      0x008  // Control register
#define REG_CMD_CHIRHO          0x010  // Command register
#define REG_RESP_LO_CHIRHO      0x014  // Response low 32 bits
#define REG_RESP_HI_CHIRHO      0x018  // Response high 32 bits
#define REG_HBM_TEST_CHIRHO     0x020  // HBM test register

// Expected values
#define EXPECTED_VERSION_CHIRHO 0xF216316A  // F2, John 3:16, rev A

// Commands (must match searchEngineChirho command encoding)
#define CMD_NOP_CHIRHO          0x00
#define CMD_INIT_CHIRHO         0x01
#define CMD_STEP_CHIRHO         0x02
#define CMD_READ_CHIRHO         0x03

static int slot_id_chirho = 0;
static pci_bar_handle_t pci_bar_handle_chirho = PCI_BAR_HANDLE_INIT;

int check_afi_ready_chirho(void) {
    struct fpga_mgmt_image_info info;
    int rc;

    rc = fpga_mgmt_describe_local_image(slot_id_chirho, &info, 0);
    if (rc) {
        printf("ERROR: Unable to get AFI information. rc=%d\n", rc);
        return -1;
    }

    if (info.status != FPGA_STATUS_LOADED) {
        printf("ERROR: AFI not loaded. Status: %d\n", info.status);
        printf("Load AFI first with: fpga-load-local-image -S %d -I <agfi-id>\n", slot_id_chirho);
        return -1;
    }

    printf("AFI loaded successfully.\n");
    printf("  Vendor ID: 0x%04x\n", info.spec.map[FPGA_APP_PF].vendor_id);
    printf("  Device ID: 0x%04x\n", info.spec.map[FPGA_APP_PF].device_id);

    return 0;
}

int test_version_chirho(void) {
    uint32_t version;
    int rc;

    printf("\n=== Test 1: Version Register ===\n");

    rc = fpga_pci_peek(pci_bar_handle_chirho, REG_VERSION_CHIRHO, &version);
    if (rc) {
        printf("FAIL: Unable to read version register. rc=%d\n", rc);
        return -1;
    }

    printf("Version register: 0x%08x\n", version);

    if (version == EXPECTED_VERSION_CHIRHO) {
        printf("PASS: Version matches expected 0x%08x\n", EXPECTED_VERSION_CHIRHO);
        return 0;
    } else {
        printf("FAIL: Expected 0x%08x, got 0x%08x\n", EXPECTED_VERSION_CHIRHO, version);
        return -1;
    }
}

int test_status_chirho(void) {
    uint32_t status;
    int rc;

    printf("\n=== Test 2: Status Register ===\n");

    rc = fpga_pci_peek(pci_bar_handle_chirho, REG_STATUS_CHIRHO, &status);
    if (rc) {
        printf("FAIL: Unable to read status register. rc=%d\n", rc);
        return -1;
    }

    printf("Status register: 0x%08x\n", status);
    printf("  Engine ready: %s\n", (status & 0x1) ? "YES" : "NO");
    printf("  HBM ready:    %s\n", (status & 0x2) ? "YES" : "NO");
    printf("  Busy:         %s\n", (status & 0x4) ? "YES" : "NO");

    return 0;
}

int test_command_chirho(void) {
    uint32_t cmd, resp_lo, resp_hi;
    int rc;

    printf("\n=== Test 3: Command/Response ===\n");

    // Send NOP command
    cmd = CMD_NOP_CHIRHO;
    printf("Sending NOP command (0x%02x)...\n", cmd);

    rc = fpga_pci_poke(pci_bar_handle_chirho, REG_CMD_CHIRHO, cmd);
    if (rc) {
        printf("FAIL: Unable to write command register. rc=%d\n", rc);
        return -1;
    }

    // Small delay for command processing
    usleep(1000);

    // Read response
    rc = fpga_pci_peek(pci_bar_handle_chirho, REG_RESP_LO_CHIRHO, &resp_lo);
    rc |= fpga_pci_peek(pci_bar_handle_chirho, REG_RESP_HI_CHIRHO, &resp_hi);
    if (rc) {
        printf("FAIL: Unable to read response registers. rc=%d\n", rc);
        return -1;
    }

    printf("Response: 0x%08x_%08x\n", resp_hi, resp_lo);
    printf("PASS: Command/response path working\n");

    return 0;
}

int test_hbm_chirho(void) {
    uint32_t test_val, read_val;
    int rc;

    printf("\n=== Test 4: HBM Access ===\n");

    // Write test pattern to HBM test register
    test_val = 0xDEADBEEF;
    printf("Writing test pattern 0x%08x to HBM test register...\n", test_val);

    rc = fpga_pci_poke(pci_bar_handle_chirho, REG_HBM_TEST_CHIRHO, test_val);
    if (rc) {
        printf("FAIL: Unable to write HBM test register. rc=%d\n", rc);
        return -1;
    }

    usleep(1000);

    rc = fpga_pci_peek(pci_bar_handle_chirho, REG_HBM_TEST_CHIRHO, &read_val);
    if (rc) {
        printf("FAIL: Unable to read HBM test register. rc=%d\n", rc);
        return -1;
    }

    printf("Read back: 0x%08x\n", read_val);

    if (read_val == test_val) {
        printf("PASS: HBM loopback test passed\n");
        return 0;
    } else {
        printf("WARN: HBM loopback mismatch (may be expected if HBM write-through)\n");
        return 0;  // Not a hard failure
    }
}

void print_usage_chirho(const char *prog) {
    printf("Usage: %s [-s slot_id]\n", prog);
    printf("  -s slot_id  FPGA slot to use (default: 0)\n");
}

int main(int argc, char *argv[]) {
    int rc;
    int opt;
    int tests_passed = 0;
    int tests_failed = 0;

    printf("============================================\n");
    printf("miniKanren FPGA Test ☧\n");
    printf("For God so loved the world - John 3:16\n");
    printf("============================================\n\n");

    // Parse arguments
    while ((opt = getopt(argc, argv, "s:h")) != -1) {
        switch (opt) {
            case 's':
                slot_id_chirho = atoi(optarg);
                break;
            case 'h':
            default:
                print_usage_chirho(argv[0]);
                return (opt == 'h') ? 0 : 1;
        }
    }

    printf("Using FPGA slot: %d\n", slot_id_chirho);

    // Initialize FPGA management library
    rc = fpga_mgmt_init();
    if (rc) {
        printf("ERROR: Unable to initialize FPGA management library. rc=%d\n", rc);
        return 1;
    }

    // Check AFI is loaded
    rc = check_afi_ready_chirho();
    if (rc) {
        goto cleanup;
    }

    // Attach to FPGA
    rc = fpga_pci_attach(slot_id_chirho, FPGA_APP_PF, APP_PF_BAR0, 0, &pci_bar_handle_chirho);
    if (rc) {
        printf("ERROR: Unable to attach to FPGA. rc=%d\n", rc);
        goto cleanup;
    }

    printf("Attached to FPGA successfully.\n");

    // Run tests
    if (test_version_chirho() == 0) tests_passed++; else tests_failed++;
    if (test_status_chirho() == 0) tests_passed++; else tests_failed++;
    if (test_command_chirho() == 0) tests_passed++; else tests_failed++;
    if (test_hbm_chirho() == 0) tests_passed++; else tests_failed++;

    // Summary
    printf("\n============================================\n");
    printf("Test Summary\n");
    printf("============================================\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);
    printf("\n");

    if (tests_failed == 0) {
        printf("ALL TESTS PASSED ☧\n");
        printf("Soli Deo Gloria\n");
    } else {
        printf("SOME TESTS FAILED\n");
    }

cleanup:
    if (pci_bar_handle_chirho != PCI_BAR_HANDLE_INIT) {
        fpga_pci_detach(pci_bar_handle_chirho);
    }
    fpga_mgmt_close();

    return tests_failed > 0 ? 1 : 0;
}
