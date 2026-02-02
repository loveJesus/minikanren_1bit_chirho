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
#define REG_PCIS_WRITE_CNT_CHIRHO   0xF0
#define REG_PCIS_READ_CNT_CHIRHO    0xF4

// HBM address space (from BAR4 perspective)
#define HBM_BASE_CHIRHO         0x10000000000ULL  // 0x10_0000_0000
#define HBM_TEST_OFFSET         0x00000000ULL     // Start of HBM

// Test parameters
#define TEST_SIZE               4096
#define EXPECTED_VERSION        0xF2560001  // V5.6

// ============================================================================
// Helper Functions
// ============================================================================

static double get_time_ns_chirho(void) {
    struct timespec ts_chirho;
    clock_gettime(CLOCK_MONOTONIC, &ts_chirho);
    return ts_chirho.tv_sec * 1e9 + ts_chirho.tv_nsec;
}

static void print_hex_dump_chirho(const char* name_chirho, uint8_t* data_chirho, size_t len_chirho) {
    printf("%s (%zu bytes):\n", name_chirho, len_chirho);
    for (size_t i_chirho = 0; i_chirho < len_chirho && i_chirho < 64; i_chirho++) {
        printf("%02x ", data_chirho[i_chirho]);
        if ((i_chirho + 1) % 16 == 0) printf("\n");
    }
    if (len_chirho > 64) printf("...\n");
    printf("\n");
}

// ============================================================================
// Main Test
// ============================================================================

int main(int argc, char **argv) {
    int rc_chirho;
    int slot_id_chirho = 0;
    pci_bar_handle_t pci_bar_handle_chirho = PCI_BAR_HANDLE_INIT;
    uint32_t version_chirho, status_chirho, arbiter_chirho, pcis_active_chirho;
    uint32_t pcis_write_cnt_chirho, pcis_read_cnt_chirho;

    printf("====================================================\n");
    printf("V5.6 HBM DMA Test ☧\n");
    printf("====================================================\n\n");

    // Initialize FPGA management library
    rc_chirho = fpga_mgmt_init();
    if (rc_chirho) {
        printf("ERROR: fpga_mgmt_init failed: %d\n", rc_chirho);
        return 1;
    }

    // Attach to FPGA slot
    rc_chirho = fpga_pci_attach(slot_id_chirho, FPGA_APP_PF, APP_PF_BAR0, 0, &pci_bar_handle_chirho);
    if (rc_chirho) {
        printf("ERROR: fpga_pci_attach failed: %d\n", rc_chirho);
        return 1;
    }
    printf("Attached to FPGA slot %d\n", slot_id_chirho);

    // ========================================================================
    // Step 1: Verify version register
    // ========================================================================

    rc_chirho = fpga_pci_peek(pci_bar_handle_chirho, REG_VERSION_CHIRHO, &version_chirho);
    if (rc_chirho) {
        printf("ERROR: Failed to read VERSION register: %d\n", rc_chirho);
        goto cleanup;
    }
    printf("VERSION: 0x%08X ", version_chirho);
    if (version_chirho == EXPECTED_VERSION) {
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
    rc_chirho = fpga_pci_poke(pci_bar_handle_chirho, REG_CONTROL_CHIRHO, 0x2);  // bit1 = reset
    if (rc_chirho) {
        printf("ERROR: Failed to write CONTROL: %d\n", rc_chirho);
        goto cleanup;
    }
    usleep(1000);  // 1ms

    // Deassert reset
    rc_chirho = fpga_pci_poke(pci_bar_handle_chirho, REG_CONTROL_CHIRHO, 0x0);
    if (rc_chirho) {
        printf("ERROR: Failed to deassert reset: %d\n", rc_chirho);
        goto cleanup;
    }
    usleep(10000);  // 10ms for HBM init

    // Wait for HBM ready (bit 2 of STATUS)
    int timeout_chirho = 100;
    do {
        rc_chirho = fpga_pci_peek(pci_bar_handle_chirho, REG_STATUS_CHIRHO, &status_chirho);
        if (rc_chirho) {
            printf("ERROR: Failed to read STATUS: %d\n", rc_chirho);
            goto cleanup;
        }
        if (status_chirho & 0x4) break;  // bit 2 = hbm_ready
        usleep(10000);
        timeout_chirho--;
    } while (timeout_chirho > 0);

    if (!(status_chirho & 0x4)) {
        printf("ERROR: HBM never became ready (STATUS=0x%08X)\n", status_chirho);
        goto cleanup;
    }
    printf("HBM ready! STATUS: 0x%08X\n", status_chirho);

    // ========================================================================
    // Step 3: Check arbiter status
    // ========================================================================

    rc_chirho = fpga_pci_peek(pci_bar_handle_chirho, REG_ARBITER_CHIRHO, &arbiter_chirho);
    printf("ARBITER: 0x%08X (grant_pcis=%d, grant_fsm=%d)\n",
           arbiter_chirho, arbiter_chirho & 1, (arbiter_chirho >> 1) & 1);

    rc_chirho = fpga_pci_peek(pci_bar_handle_chirho, REG_PCIS_ACTIVE_CHIRHO, &pcis_active_chirho);
    printf("PCIS_ACTIVE: %d\n", pcis_active_chirho & 1);

    // ========================================================================
    // Step 4: Open DMA queues
    // ========================================================================

    printf("\nOpening DMA queues...\n");

    int write_fd_chirho = fpga_dma_open_queue(FPGA_DMA_XDMA, slot_id_chirho, 0, false);  // H2C
    if (write_fd_chirho < 0) {
        printf("ERROR: Failed to open DMA write queue: %d\n", write_fd_chirho);
        goto cleanup;
    }
    printf("DMA write queue opened (fd=%d)\n", write_fd_chirho);

    int read_fd_chirho = fpga_dma_open_queue(FPGA_DMA_XDMA, slot_id_chirho, 0, true);   // C2H
    if (read_fd_chirho < 0) {
        printf("ERROR: Failed to open DMA read queue: %d\n", read_fd_chirho);
        close(write_fd_chirho);
        goto cleanup;
    }
    printf("DMA read queue opened (fd=%d)\n", read_fd_chirho);

    // ========================================================================
    // Step 5: Allocate aligned buffers
    // ========================================================================

    uint8_t *write_buf_chirho = aligned_alloc(4096, TEST_SIZE);
    uint8_t *read_buf_chirho = aligned_alloc(4096, TEST_SIZE);
    if (!write_buf_chirho || !read_buf_chirho) {
        printf("ERROR: Failed to allocate buffers\n");
        goto cleanup_dma;
    }

    // Fill write buffer with test pattern
    for (int i_chirho = 0; i_chirho < TEST_SIZE; i_chirho++) {
        write_buf_chirho[i_chirho] = i_chirho & 0xFF;
    }
    memset(read_buf_chirho, 0, TEST_SIZE);

    print_hex_dump_chirho("Write buffer", write_buf_chirho, 64);

    // ========================================================================
    // Step 6: DMA write to HBM
    // ========================================================================

    printf("Performing DMA write to HBM (address 0x%llx, size %d)...\n",
           (unsigned long long)HBM_BASE_CHIRHO, TEST_SIZE);

    double t0_chirho = get_time_ns_chirho();
    rc_chirho = fpga_dma_burst_write(write_fd_chirho, write_buf_chirho, TEST_SIZE, HBM_BASE_CHIRHO + HBM_TEST_OFFSET);
    double t1_chirho = get_time_ns_chirho();

    if (rc_chirho) {
        printf("ERROR: DMA write failed: %d\n", rc_chirho);
        goto cleanup_buffers;
    }
    printf("DMA write completed in %.2f us\n", (t1_chirho - t0_chirho) / 1000.0);

    // Check PCIS counters
    rc_chirho = fpga_pci_peek(pci_bar_handle_chirho, REG_PCIS_WRITE_CNT_CHIRHO, &pcis_write_cnt_chirho);
    printf("PCIS write count: %u\n", pcis_write_cnt_chirho);

    // ========================================================================
    // Step 7: DMA read from HBM
    // ========================================================================

    printf("Performing DMA read from HBM...\n");

    t0_chirho = get_time_ns_chirho();
    rc_chirho = fpga_dma_burst_read(read_fd_chirho, read_buf_chirho, TEST_SIZE, HBM_BASE_CHIRHO + HBM_TEST_OFFSET);
    t1_chirho = get_time_ns_chirho();

    if (rc_chirho) {
        printf("ERROR: DMA read failed: %d\n", rc_chirho);
        goto cleanup_buffers;
    }
    printf("DMA read completed in %.2f us\n", (t1_chirho - t0_chirho) / 1000.0);

    // Check PCIS counters
    rc_chirho = fpga_pci_peek(pci_bar_handle_chirho, REG_PCIS_READ_CNT_CHIRHO, &pcis_read_cnt_chirho);
    printf("PCIS read count: %u\n", pcis_read_cnt_chirho);

    print_hex_dump_chirho("Read buffer", read_buf_chirho, 64);

    // ========================================================================
    // Step 8: Verify data
    // ========================================================================

    printf("\nVerifying data...\n");
    int errors_chirho = 0;
    int first_error_idx_chirho = -1;
    for (int i_chirho = 0; i_chirho < TEST_SIZE; i_chirho++) {
        if (write_buf_chirho[i_chirho] != read_buf_chirho[i_chirho]) {
            if (first_error_idx_chirho < 0) first_error_idx_chirho = i_chirho;
            errors_chirho++;
            if (errors_chirho <= 10) {
                printf("Mismatch at offset %d: wrote 0x%02X, read 0x%02X\n",
                       i_chirho, write_buf_chirho[i_chirho], read_buf_chirho[i_chirho]);
            }
        }
    }

    printf("\n====================================================\n");
    if (errors_chirho == 0) {
        printf("SUCCESS: HBM DMA VERIFIED! ☧\n");
        printf("All %d bytes match.\n", TEST_SIZE);
        printf("====================================================\n");
        rc_chirho = 0;
    } else {
        printf("FAILED: %d/%d byte mismatches (first at offset %d)\n",
               errors_chirho, TEST_SIZE, first_error_idx_chirho);
        printf("====================================================\n");
        rc_chirho = 1;
    }

    // ========================================================================
    // Cleanup
    // ========================================================================

cleanup_buffers:
    free(write_buf_chirho);
    free(read_buf_chirho);

cleanup_dma:
    close(write_fd_chirho);
    close(read_fd_chirho);

cleanup:
    fpga_pci_detach(pci_bar_handle_chirho);
    fpga_mgmt_close();

    return rc_chirho;
}
