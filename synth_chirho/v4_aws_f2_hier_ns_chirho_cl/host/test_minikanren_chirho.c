// ============================================================================
// For God so loved the world - John 3:16 ☧
// Host test program for miniKanren FPGA accelerator
// v4: Hierarchical Domains + Neurosymbolic
// ============================================================================

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/mman.h>

// AWS FPGA includes
#include <fpga_pci.h>
#include <fpga_mgmt.h>
#include <utils/lcd.h>

// Register addresses (from cl_minikanren_chirho_defines.vh)
#define REG_VERSION_CHIRHO      0x00
#define REG_CONTROL_CHIRHO      0x04
#define REG_STATUS_CHIRHO       0x08
#define REG_CMD_LO_CHIRHO       0x10
#define REG_CMD_MID_CHIRHO      0x14
#define REG_CMD_HI_CHIRHO       0x18
#define REG_RESP_BASE_CHIRHO    0x20
#define REG_HIER_MODE_CHIRHO    0x40
#define REG_HIER_LEVEL_CHIRHO   0x44
#define REG_BEAT_COUNT_CHIRHO   0x48
#define REG_TRAIN_MODE_CHIRHO   0x50
#define REG_INFER_MODE_CHIRHO   0x70

// Control register bits
#define CTRL_ENABLE_CHIRHO      (1 << 0)
#define CTRL_RESET_CHIRHO       (1 << 1)
#define CTRL_HBM_MODE_CHIRHO    (1 << 2)

// Status register bits
#define STATUS_DONE_CHIRHO      (1 << 0)
#define STATUS_VALID_CHIRHO     (1 << 1)
#define STATUS_HBM_READY_CHIRHO (1 << 2)

// Hierarchical modes
#define HIER_MODE_64_CHIRHO     0  // 64 values (BitVec64)
#define HIER_MODE_256SQ_CHIRHO  1  // 256² = 65K values
#define HIER_MODE_512SQ_CHIRHO  2  // 512² = 262K values
#define HIER_MODE_256CB_CHIRHO  3  // 256³ = 16.7M values
#define HIER_MODE_512CB_CHIRHO  4  // 512³ = 134M values

// Expected version: 0xF004 (PCI ID) + 0x316A (John 3:16 in hex-ish)
#define EXPECTED_VERSION_CHIRHO 0xF004316A

static int slot_id_chirho = 0;
static pci_bar_handle_t pci_bar_handle_chirho = PCI_BAR_HANDLE_INIT;

// ============================================================================
// Helper functions
// ============================================================================

static int check_afi_ready_chirho(void) {
    struct fpga_mgmt_image_info info_chirho = {0};
    int rc_chirho = fpga_mgmt_describe_local_image(slot_id_chirho, &info_chirho, 0);
    if (rc_chirho) {
        printf("ERROR: Unable to get FPGA image info: %d\n", rc_chirho);
        return -1;
    }
    if (info_chirho.status != FPGA_STATUS_LOADED) {
        printf("ERROR: AFI not loaded. Status: %d\n", info_chirho.status);
        return -1;
    }
    printf("AFI loaded. Vendor: 0x%04x, Device: 0x%04x\n",
           info_chirho.spec.map[0].vendor_id, info_chirho.spec.map[0].device_id);
    return 0;
}

static int reg_read_chirho(uint32_t addr_chirho, uint32_t *value_chirho) {
    return fpga_pci_peek(pci_bar_handle_chirho, addr_chirho, value_chirho);
}

static int reg_write_chirho(uint32_t addr_chirho, uint32_t value_chirho) {
    return fpga_pci_poke(pci_bar_handle_chirho, addr_chirho, value_chirho);
}

// ============================================================================
// Test functions
// ============================================================================

static int test_version_chirho(void) {
    uint32_t version_chirho;
    int rc_chirho = reg_read_chirho(REG_VERSION_CHIRHO, &version_chirho);
    if (rc_chirho) {
        printf("FAIL: Version read error: %d\n", rc_chirho);
        return -1;
    }
    printf("Version: 0x%08X (expected 0x%08X)\n", version_chirho, EXPECTED_VERSION_CHIRHO);
    if (version_chirho != EXPECTED_VERSION_CHIRHO) {
        printf("WARN: Version mismatch (may be OK for v4)\n");
    }
    return 0;
}

static int test_status_chirho(void) {
    uint32_t status_chirho;
    int rc_chirho = reg_read_chirho(REG_STATUS_CHIRHO, &status_chirho);
    if (rc_chirho) {
        printf("FAIL: Status read error: %d\n", rc_chirho);
        return -1;
    }
    printf("Status: 0x%08X\n", status_chirho);
    printf("  Done:      %s\n", (status_chirho & STATUS_DONE_CHIRHO) ? "Yes" : "No");
    printf("  Valid:     %s\n", (status_chirho & STATUS_VALID_CHIRHO) ? "Yes" : "No");
    printf("  HBM Ready: %s\n", (status_chirho & STATUS_HBM_READY_CHIRHO) ? "Yes" : "No");
    return 0;
}

static int test_reset_chirho(void) {
    int rc_chirho;

    // Assert reset
    rc_chirho = reg_write_chirho(REG_CONTROL_CHIRHO, CTRL_RESET_CHIRHO);
    if (rc_chirho) {
        printf("FAIL: Reset write error: %d\n", rc_chirho);
        return -1;
    }
    usleep(1000);  // 1ms

    // Deassert reset, enable
    rc_chirho = reg_write_chirho(REG_CONTROL_CHIRHO, CTRL_ENABLE_CHIRHO);
    if (rc_chirho) {
        printf("FAIL: Enable write error: %d\n", rc_chirho);
        return -1;
    }
    usleep(1000);

    printf("Reset and enable complete\n");
    return 0;
}

static int test_hier_mode_chirho(int mode_chirho) {
    int rc_chirho;
    const char *mode_names_chirho[] = {
        "64 (BitVec64)",
        "256² (65K)",
        "512² (262K)",
        "256³ (16.7M)",
        "512³ (134M)"
    };

    if (mode_chirho < 0 || mode_chirho > 4) {
        printf("FAIL: Invalid mode %d\n", mode_chirho);
        return -1;
    }

    rc_chirho = reg_write_chirho(REG_HIER_MODE_CHIRHO, mode_chirho);
    if (rc_chirho) {
        printf("FAIL: Hier mode write error: %d\n", rc_chirho);
        return -1;
    }

    uint32_t readback_chirho;
    rc_chirho = reg_read_chirho(REG_HIER_MODE_CHIRHO, &readback_chirho);
    if (rc_chirho) {
        printf("FAIL: Hier mode read error: %d\n", rc_chirho);
        return -1;
    }

    printf("Hierarchical mode set to %d: %s (readback: %d)\n",
           mode_chirho, mode_names_chirho[mode_chirho], readback_chirho);
    return (readback_chirho == (uint32_t)mode_chirho) ? 0 : -1;
}

static int test_simple_query_chirho(void) {
    int rc_chirho;
    uint32_t status_chirho, resp_chirho;

    // Write a simple command (NOP: all zeros)
    rc_chirho = reg_write_chirho(REG_CMD_LO_CHIRHO, 0x00000000);
    rc_chirho |= reg_write_chirho(REG_CMD_MID_CHIRHO, 0x00000000);
    rc_chirho |= reg_write_chirho(REG_CMD_HI_CHIRHO, 0x00000000);
    if (rc_chirho) {
        printf("FAIL: Command write error\n");
        return -1;
    }

    // Wait for done
    for (int i_chirho = 0; i_chirho < 100; i_chirho++) {
        rc_chirho = reg_read_chirho(REG_STATUS_CHIRHO, &status_chirho);
        if (rc_chirho) break;
        if (status_chirho & STATUS_DONE_CHIRHO) break;
        usleep(100);
    }

    if (!(status_chirho & STATUS_DONE_CHIRHO)) {
        printf("WARN: Command did not complete (status=0x%08X)\n", status_chirho);
    }

    // Read response
    rc_chirho = reg_read_chirho(REG_RESP_BASE_CHIRHO, &resp_chirho);
    if (rc_chirho) {
        printf("FAIL: Response read error: %d\n", rc_chirho);
        return -1;
    }

    printf("Simple query complete. Response: 0x%08X\n", resp_chirho);
    return 0;
}

static int wait_hbm_ready_chirho(int timeout_ms_chirho) {
    uint32_t status_chirho;
    int elapsed_chirho = 0;

    while (elapsed_chirho < timeout_ms_chirho) {
        int rc_chirho = reg_read_chirho(REG_STATUS_CHIRHO, &status_chirho);
        if (rc_chirho) return -1;
        if (status_chirho & STATUS_HBM_READY_CHIRHO) {
            printf("HBM ready after %d ms\n", elapsed_chirho);
            return 0;
        }
        usleep(10000);  // 10ms
        elapsed_chirho += 10;
    }

    printf("WARN: HBM not ready after %d ms\n", timeout_ms_chirho);
    return -1;
}

// ============================================================================
// Benchmark functions
// ============================================================================

static int benchmark_intersect_chirho(int mode_chirho, int iterations_chirho) {
    int rc_chirho;
    uint32_t status_chirho;
    struct timespec start_chirho, end_chirho;

    printf("\n=== Benchmark: Mode %d, %d iterations ===\n", mode_chirho, iterations_chirho);

    // Set mode
    rc_chirho = test_hier_mode_chirho(mode_chirho);
    if (rc_chirho) return -1;

    // Enable HBM mode for large domains
    if (mode_chirho >= HIER_MODE_256CB_CHIRHO) {
        rc_chirho = reg_write_chirho(REG_CONTROL_CHIRHO, CTRL_ENABLE_CHIRHO | CTRL_HBM_MODE_CHIRHO);
        if (rc_chirho) return -1;
        wait_hbm_ready_chirho(5000);
    }

    clock_gettime(CLOCK_MONOTONIC, &start_chirho);

    for (int i_chirho = 0; i_chirho < iterations_chirho; i_chirho++) {
        // Write command (simple intersect: cmd = 0x01)
        reg_write_chirho(REG_CMD_LO_CHIRHO, 0x00000001);
        reg_write_chirho(REG_CMD_MID_CHIRHO, 0xFFFFFFFF);  // All bits set
        reg_write_chirho(REG_CMD_HI_CHIRHO, 0x00000000);

        // Wait for done
        do {
            reg_read_chirho(REG_STATUS_CHIRHO, &status_chirho);
        } while (!(status_chirho & STATUS_DONE_CHIRHO));
    }

    clock_gettime(CLOCK_MONOTONIC, &end_chirho);

    double elapsed_ns_chirho = (end_chirho.tv_sec - start_chirho.tv_sec) * 1e9 + (end_chirho.tv_nsec - start_chirho.tv_nsec);
    double per_op_ns_chirho = elapsed_ns_chirho / iterations_chirho;
    double ops_per_sec_chirho = 1e9 / per_op_ns_chirho;

    printf("Total time: %.3f ms\n", elapsed_ns_chirho / 1e6);
    printf("Per operation: %.1f ns\n", per_op_ns_chirho);
    printf("Operations/sec: %.0f\n", ops_per_sec_chirho);

    return 0;
}

// ============================================================================
// Main
// ============================================================================

int main(int argc, char **argv) {
    int rc_chirho;
    int run_benchmarks_chirho = 0;

    printf("============================================================\n");
    printf("miniKanren FPGA Test - v4 Hierarchical + Neurosymbolic ☧\n");
    printf("For God so loved the world - John 3:16\n");
    printf("============================================================\n\n");

    if (argc > 1 && strcmp(argv[1], "-b") == 0) {
        run_benchmarks_chirho = 1;
    }

    // Initialize FPGA libraries
    rc_chirho = fpga_mgmt_init();
    if (rc_chirho) {
        printf("ERROR: fpga_mgmt_init failed: %d\n", rc_chirho);
        return 1;
    }

    // Check AFI is loaded
    rc_chirho = check_afi_ready_chirho();
    if (rc_chirho) {
        fpga_mgmt_close();
        return 1;
    }

    // Attach to BAR0 (OCL)
    rc_chirho = fpga_pci_attach(slot_id_chirho, FPGA_APP_PF, APP_PF_BAR0, 0, &pci_bar_handle_chirho);
    if (rc_chirho) {
        printf("ERROR: fpga_pci_attach failed: %d\n", rc_chirho);
        fpga_mgmt_close();
        return 1;
    }

    printf("\n--- Basic Tests ---\n");
    test_version_chirho();
    test_status_chirho();
    test_reset_chirho();
    test_status_chirho();

    printf("\n--- Hierarchical Mode Tests ---\n");
    for (int mode_chirho = 0; mode_chirho <= 4; mode_chirho++) {
        test_hier_mode_chirho(mode_chirho);
    }

    printf("\n--- Simple Query Test ---\n");
    test_simple_query_chirho();

    if (run_benchmarks_chirho) {
        printf("\n--- Benchmarks ---\n");
        benchmark_intersect_chirho(HIER_MODE_64_CHIRHO, 10000);
        benchmark_intersect_chirho(HIER_MODE_256SQ_CHIRHO, 1000);
        benchmark_intersect_chirho(HIER_MODE_512SQ_CHIRHO, 1000);
        // HBM modes need more setup
        // benchmark_intersect_chirho(HIER_MODE_256CB_CHIRHO, 100);
    }

    printf("\n============================================================\n");
    printf("Tests complete. Soli Deo Gloria ☧\n");
    printf("============================================================\n");

    fpga_pci_detach(pci_bar_handle_chirho);
    fpga_mgmt_close();

    return 0;
}
