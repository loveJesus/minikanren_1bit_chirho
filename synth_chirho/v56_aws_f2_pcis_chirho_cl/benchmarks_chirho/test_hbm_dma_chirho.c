// ============================================================================
// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// V5.6 HBM DMA Test - Using AWS FPGA SDK ☧
//
// Tests that HBM is actually accessible via PCIS (BAR4) using DMA.
// This is THE CRITICAL TEST to verify the V5.6 PCIS fix works.
//
// Build:
//   gcc -I$SDK_DIR/userspace/include -L$SDK_DIR/userspace/lib \
//       -o test_hbm_dma_chirho test_hbm_dma_chirho.c \
//       -lfpga_mgmt -lrt -lpthread
//
// Run:
//   sudo ./test_hbm_dma_chirho
//
// ============================================================================

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <time.h>

// AWS FPGA SDK headers
#include <fpga_pci.h>
#include <fpga_mgmt.h>
#include <fpga_dma.h>

// ============================================================================
// Register Addresses (must match cl_minikanren_chirho_defines.vh)
// ============================================================================

#define REG_VERSION_CHIRHO      0x00
#define REG_CONTROL_CHIRHO      0x04
#define REG_STATUS_CHIRHO       0x08
#define REG_HIER_MODE_CHIRHO    0x80
#define REG_FSM_STATE_CHIRHO    0xC0
#define REG_AXI_STATUS_CHIRHO   0xC4
#define REG_ARBITER_CHIRHO      0xE0
#define REG_PCIS_ACTIVE_CHIRHO  0xEC
#define REG_PCIS_WRITE_CNT      0xF0
#define REG_PCIS_READ_CNT       0xF4

// HBM address space (from BAR4 perspective)
#define HBM_BASE_CHIRHO         0x10000000000ULL  // 0x10_0000_0000
#define HBM_TEST_OFFSET         0x00000000ULL     // Start of HBM

// Test parameters
#define TEST_SIZE               4096
#define EXPECTED_VERSION        0xF2560001  // V5.6

// ============================================================================
// Helper Functions
// ============================================================================

static double get_time_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e9 + ts.tv_nsec;
}

static void print_hex_dump(const char* name, uint8_t* data, size_t len) {
    printf("%s (%zu bytes):\n", name, len);
    for (size_t i = 0; i < len && i < 64; i++) {
        printf("%02x ", data[i]);
        if ((i + 1) % 16 == 0) printf("\n");
    }
    if (len > 64) printf("...\n");
    printf("\n");
}

// ============================================================================
// Main Test
// ============================================================================

int main(int argc, char **argv) {
    int rc;
    int slot_id = 0;
    pci_bar_handle_t pci_bar_handle = PCI_BAR_HANDLE_INIT;
    uint32_t version, status, arbiter, pcis_active;
    uint32_t pcis_write_cnt, pcis_read_cnt;

    printf("====================================================\n");
    printf("V5.6 HBM DMA Test ☧\n");
    printf("====================================================\n\n");

    // Initialize FPGA management library
    rc = fpga_mgmt_init();
    if (rc) {
        printf("ERROR: fpga_mgmt_init failed: %d\n", rc);
        return 1;
    }

    // Attach to FPGA slot
    rc = fpga_pci_attach(slot_id, FPGA_APP_PF, APP_PF_BAR0, 0, &pci_bar_handle);
    if (rc) {
        printf("ERROR: fpga_pci_attach failed: %d\n", rc);
        return 1;
    }
    printf("Attached to FPGA slot %d\n", slot_id);

    // ========================================================================
    // Step 1: Verify version register
    // ========================================================================

    rc = fpga_pci_peek(pci_bar_handle, REG_VERSION_CHIRHO, &version);
    if (rc) {
        printf("ERROR: Failed to read VERSION register: %d\n", rc);
        goto cleanup;
    }
    printf("VERSION: 0x%08X ", version);
    if (version == EXPECTED_VERSION) {
        printf("(V5.6 CORRECT)\n");
    } else {
        printf("(UNEXPECTED - expected 0x%08X)\n", EXPECTED_VERSION);
        // Continue anyway to see what we have
    }

    // ========================================================================
    // Step 2: Reset and wait for HBM ready
    // ========================================================================

    printf("\nResetting CL and waiting for HBM ready...\n");

    // Assert reset
    rc = fpga_pci_poke(pci_bar_handle, REG_CONTROL_CHIRHO, 0x2);  // bit1 = reset
    if (rc) {
        printf("ERROR: Failed to write CONTROL: %d\n", rc);
        goto cleanup;
    }
    usleep(1000);  // 1ms

    // Deassert reset
    rc = fpga_pci_poke(pci_bar_handle, REG_CONTROL_CHIRHO, 0x0);
    if (rc) {
        printf("ERROR: Failed to deassert reset: %d\n", rc);
        goto cleanup;
    }
    usleep(10000);  // 10ms for HBM init

    // Wait for HBM ready (bit 2 of STATUS)
    int timeout = 100;
    do {
        rc = fpga_pci_peek(pci_bar_handle, REG_STATUS_CHIRHO, &status);
        if (rc) {
            printf("ERROR: Failed to read STATUS: %d\n", rc);
            goto cleanup;
        }
        if (status & 0x4) break;  // bit 2 = hbm_ready
        usleep(10000);
        timeout--;
    } while (timeout > 0);

    if (!(status & 0x4)) {
        printf("ERROR: HBM never became ready (STATUS=0x%08X)\n", status);
        goto cleanup;
    }
    printf("HBM ready! STATUS: 0x%08X\n", status);

    // ========================================================================
    // Step 3: Check arbiter status
    // ========================================================================

    rc = fpga_pci_peek(pci_bar_handle, REG_ARBITER_CHIRHO, &arbiter);
    printf("ARBITER: 0x%08X (grant_pcis=%d, grant_fsm=%d)\n",
           arbiter, arbiter & 1, (arbiter >> 1) & 1);

    rc = fpga_pci_peek(pci_bar_handle, REG_PCIS_ACTIVE_CHIRHO, &pcis_active);
    printf("PCIS_ACTIVE: %d\n", pcis_active & 1);

    // ========================================================================
    // Step 4: Open DMA queues
    // ========================================================================

    printf("\nOpening DMA queues...\n");

    int write_fd = fpga_dma_open_queue(FPGA_DMA_XDMA, slot_id, 0, false);  // H2C
    if (write_fd < 0) {
        printf("ERROR: Failed to open DMA write queue: %d\n", write_fd);
        goto cleanup;
    }
    printf("DMA write queue opened (fd=%d)\n", write_fd);

    int read_fd = fpga_dma_open_queue(FPGA_DMA_XDMA, slot_id, 0, true);   // C2H
    if (read_fd < 0) {
        printf("ERROR: Failed to open DMA read queue: %d\n", read_fd);
        close(write_fd);
        goto cleanup;
    }
    printf("DMA read queue opened (fd=%d)\n", read_fd);

    // ========================================================================
    // Step 5: Allocate aligned buffers
    // ========================================================================

    uint8_t *write_buf = aligned_alloc(4096, TEST_SIZE);
    uint8_t *read_buf = aligned_alloc(4096, TEST_SIZE);
    if (!write_buf || !read_buf) {
        printf("ERROR: Failed to allocate buffers\n");
        goto cleanup_dma;
    }

    // Fill write buffer with test pattern
    for (int i = 0; i < TEST_SIZE; i++) {
        write_buf[i] = i & 0xFF;
    }
    memset(read_buf, 0, TEST_SIZE);

    print_hex_dump("Write buffer", write_buf, 64);

    // ========================================================================
    // Step 6: DMA write to HBM
    // ========================================================================

    printf("Performing DMA write to HBM (address 0x%llx, size %d)...\n",
           (unsigned long long)HBM_BASE_CHIRHO, TEST_SIZE);

    double t0 = get_time_ns();
    rc = fpga_dma_burst_write(write_fd, write_buf, TEST_SIZE, HBM_BASE_CHIRHO + HBM_TEST_OFFSET);
    double t1 = get_time_ns();

    if (rc) {
        printf("ERROR: DMA write failed: %d\n", rc);
        goto cleanup_buffers;
    }
    printf("DMA write completed in %.2f us\n", (t1 - t0) / 1000.0);

    // Check PCIS counters
    rc = fpga_pci_peek(pci_bar_handle, REG_PCIS_WRITE_CNT, &pcis_write_cnt);
    printf("PCIS write count: %u\n", pcis_write_cnt);

    // ========================================================================
    // Step 7: DMA read from HBM
    // ========================================================================

    printf("Performing DMA read from HBM...\n");

    t0 = get_time_ns();
    rc = fpga_dma_burst_read(read_fd, read_buf, TEST_SIZE, HBM_BASE_CHIRHO + HBM_TEST_OFFSET);
    t1 = get_time_ns();

    if (rc) {
        printf("ERROR: DMA read failed: %d\n", rc);
        goto cleanup_buffers;
    }
    printf("DMA read completed in %.2f us\n", (t1 - t0) / 1000.0);

    // Check PCIS counters
    rc = fpga_pci_peek(pci_bar_handle, REG_PCIS_READ_CNT, &pcis_read_cnt);
    printf("PCIS read count: %u\n", pcis_read_cnt);

    print_hex_dump("Read buffer", read_buf, 64);

    // ========================================================================
    // Step 8: Verify data
    // ========================================================================

    printf("\nVerifying data...\n");
    int errors = 0;
    int first_error_idx = -1;
    for (int i = 0; i < TEST_SIZE; i++) {
        if (write_buf[i] != read_buf[i]) {
            if (first_error_idx < 0) first_error_idx = i;
            errors++;
            if (errors <= 10) {
                printf("Mismatch at offset %d: wrote 0x%02X, read 0x%02X\n",
                       i, write_buf[i], read_buf[i]);
            }
        }
    }

    printf("\n====================================================\n");
    if (errors == 0) {
        printf("SUCCESS: HBM DMA VERIFIED! ☧\n");
        printf("All %d bytes match.\n", TEST_SIZE);
        printf("====================================================\n");
        rc = 0;
    } else {
        printf("FAILED: %d/%d byte mismatches (first at offset %d)\n",
               errors, TEST_SIZE, first_error_idx);
        printf("====================================================\n");
        rc = 1;
    }

    // ========================================================================
    // Cleanup
    // ========================================================================

cleanup_buffers:
    free(write_buf);
    free(read_buf);

cleanup_dma:
    close(write_fd);
    close(read_fd);

cleanup:
    fpga_pci_detach(pci_bar_handle);
    fpga_mgmt_close();

    return rc;
}
