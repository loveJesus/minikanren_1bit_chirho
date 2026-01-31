/* ============================================================================
 * For God so loved the world, that He gave His only begotten Son,
 * that whosoever believeth in Him should not perish, but have everlasting life.
 * John 3:16
 *
 * hier_262k_benchmark_chirho.c - 512² Hierarchical Domain Benchmark ☧
 *
 * Tests the HBM-based 262K hierarchical domain path.
 *
 * REGISTER MAP CORRECTED from cl_minikanren_chirho.sv OCL read/write logic:
 * - OCL uses rd_addr[7:2] and wr_addr[7:2] as case selectors
 * - So case 6'h00 = address 0x00, 6'h01 = 0x04, 6'h02 = 0x08, etc.
 *
 * Uses mmap for direct FPGA register access (no AWS SDK required)
 *
 * Compile: gcc -O2 -o hier_262k_benchmark_chirho hier_262k_benchmark_chirho.c
 * Run: sudo ./hier_262k_benchmark_chirho
 *
 * Soli Deo Gloria ☧
 * ============================================================================ */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>

// =============================================================================
// CORRECT Register addresses from cl_minikanren_chirho.sv OCL case statements
// OCL uses addr[7:2] so byte_addr = case_value * 4
// =============================================================================
#define REG_VERSION_CHIRHO      0x00  // 6'h00: VERSION register (read-only)
#define REG_CONTROL_CHIRHO      0x04  // 6'h01: CONTROL {hbm_mode[2], reset[1], enable[0]}
#define REG_STATUS_CHIRHO       0x08  // 6'h02: STATUS {hbm_ready[2], valid[1], done[0]}
#define REG_CMD_LO_CHIRHO       0x10  // 6'h04: cmd[31:0]
#define REG_CMD_MID_CHIRHO      0x14  // 6'h05: cmd[63:32]
#define REG_CMD_HI_CHIRHO       0x18  // 6'h06: cmd[69:64]
#define REG_HIER_MODE_CHIRHO    0x80  // 6'h20: Hierarchical mode (0=flat, 1=65K, 2=262K)
#define REG_FSM_STATE_CHIRHO    0xC0  // 6'h30: Debug - FSM state
#define REG_AXI_STATUS_CHIRHO   0xC4  // 6'h31: Debug - AXI handshake signals
#define REG_BEAT_COUNT_CHIRHO   0xD0  // 6'h34: Debug - beat_counter

// Response registers at 0x20-0x5C (6'h08 - 6'h17)
#define REG_RESP_BASE_CHIRHO    0x20

// CONTROL register bits
#define CTRL_ENABLE_CHIRHO      0x01  // bit 0
#define CTRL_RESET_CHIRHO       0x02  // bit 1
#define CTRL_HBM_MODE_CHIRHO    0x04  // bit 2

// STATUS register bits
#define STATUS_DONE_CHIRHO      0x01  // bit 0
#define STATUS_VALID_CHIRHO     0x02  // bit 1
#define STATUS_HBM_READY_CHIRHO 0x04  // bit 2

// Hierarchical modes
#define HIER_MODE_FLAT256_CHIRHO     0  // 256 values (1 HBM beat)
#define HIER_MODE_HIER_65K_CHIRHO    1  // 256² = 65K values
#define HIER_MODE_HIER_262K_CHIRHO   2  // 512² = 262K values

// Operation codes (for CMD encoding)
#define OP_INTERSECT_CHIRHO     0
#define OP_UNION_CHIRHO         1
#define OP_COMPLEMENT_CHIRHO    2
#define OP_IS_GROUND_CHIRHO     3

// Expected version
#define EXPECTED_VERSION_V55_CHIRHO 0xF2550001

// BAR0 size for mmap
#define BAR0_SIZE_CHIRHO        (64 * 1024)  // 64KB should be enough

static volatile uint32_t *fpga_bar0_chirho = NULL;

// Initialize FPGA via mmap (no AWS SDK required)
static int init_fpga_chirho(void) {
    // Try write-combine path first (faster)
    const char *bar0_paths_chirho[] = {
        "/sys/bus/pci/devices/0000:34:00.0/resource0_wc",
        "/sys/bus/pci/devices/0000:34:00.0/resource0",
        NULL
    };

    for (int i = 0; bar0_paths_chirho[i] != NULL; i++) {
        int fd_chirho = open(bar0_paths_chirho[i], O_RDWR | O_SYNC);
        if (fd_chirho >= 0) {
            fpga_bar0_chirho = mmap(NULL, BAR0_SIZE_CHIRHO, PROT_READ | PROT_WRITE,
                                     MAP_SHARED, fd_chirho, 0);
            if (fpga_bar0_chirho != MAP_FAILED) {
                printf("Opened FPGA BAR0 via %s\n", bar0_paths_chirho[i]);
                return 0;
            }
            close(fd_chirho);
        }
    }

    perror("Failed to open FPGA BAR0");
    return -1;
}

// Read register
static inline uint32_t read_reg_chirho(uint32_t offset_chirho) {
    return fpga_bar0_chirho[offset_chirho / 4];
}

// Write register
static inline void write_reg_chirho(uint32_t offset_chirho, uint32_t value_chirho) {
    fpga_bar0_chirho[offset_chirho / 4] = value_chirho;
    __sync_synchronize();  // Memory barrier
}

// Get current time in microseconds
static inline double get_time_us_chirho(void) {
    struct timespec ts_chirho;
    clock_gettime(CLOCK_MONOTONIC, &ts_chirho);
    return ts_chirho.tv_sec * 1e6 + ts_chirho.tv_nsec / 1e3;
}

// Dump all debug registers
static void dump_debug_regs_chirho(void) {
    printf("\n=== Debug Registers ===\n");

    uint32_t version_chirho = read_reg_chirho(REG_VERSION_CHIRHO);
    printf("VERSION (0x00): 0x%08X\n", version_chirho);

    uint32_t control_chirho = read_reg_chirho(REG_CONTROL_CHIRHO);
    printf("CONTROL (0x04): 0x%08X (enable=%d, reset=%d, hbm_mode=%d)\n",
           control_chirho,
           (control_chirho >> 0) & 1,
           (control_chirho >> 1) & 1,
           (control_chirho >> 2) & 1);

    uint32_t status_chirho = read_reg_chirho(REG_STATUS_CHIRHO);
    printf("STATUS  (0x08): 0x%08X (done=%d, valid=%d, hbm_ready=%d)\n",
           status_chirho,
           (status_chirho >> 0) & 1,
           (status_chirho >> 1) & 1,
           (status_chirho >> 2) & 1);

    uint32_t hier_mode_chirho = read_reg_chirho(REG_HIER_MODE_CHIRHO);
    printf("HIER_MODE (0x80): 0x%08X\n", hier_mode_chirho);

    uint32_t fsm_state_chirho = read_reg_chirho(REG_FSM_STATE_CHIRHO);
    printf("FSM_STATE (0xC0): 0x%08X (state=%d)\n",
           fsm_state_chirho, fsm_state_chirho & 0x1F);

    uint32_t axi_status_chirho = read_reg_chirho(REG_AXI_STATUS_CHIRHO);
    printf("AXI_STATUS (0xC4): 0x%08X\n", axi_status_chirho);
    printf("  arready=%d arvalid=%d rvalid=%d rready=%d\n",
           (axi_status_chirho >> 0) & 1,
           (axi_status_chirho >> 1) & 1,
           (axi_status_chirho >> 2) & 1,
           (axi_status_chirho >> 3) & 1);
    printf("  awready=%d awvalid=%d bvalid=%d bready=%d\n",
           (axi_status_chirho >> 4) & 1,
           (axi_status_chirho >> 5) & 1,
           (axi_status_chirho >> 6) & 1,
           (axi_status_chirho >> 7) & 1);

    uint32_t beat_count_chirho = read_reg_chirho(REG_BEAT_COUNT_CHIRHO);
    printf("BEAT_COUNT (0xD0): 0x%08X (%d)\n", beat_count_chirho, beat_count_chirho & 0xFFFFF);
}

// Encode a command for the FSM
// Command format from cl_minikanren_chirho.sv:
//   [3:0]   = op_code
//   [19:4]  = var_id_1 (16 bits)
//   [35:20] = var_id_2 (16 bits)
//   [51:36] = result_var_id (16 bits)
//   [67:52] = batch_count (16 bits)
//   [69:68] = hier_mode (2 bits)
static void encode_command_chirho(
    uint32_t op_code_chirho,
    uint32_t var_id_1_chirho,
    uint32_t var_id_2_chirho,
    uint32_t result_id_chirho,
    uint32_t batch_count_chirho,
    uint32_t hier_mode_chirho,
    uint32_t *cmd_lo_chirho,
    uint32_t *cmd_mid_chirho,
    uint32_t *cmd_hi_chirho
) {
    uint64_t cmd_chirho = 0;
    cmd_chirho |= (uint64_t)(op_code_chirho & 0xF);
    cmd_chirho |= (uint64_t)(var_id_1_chirho & 0xFFFF) << 4;
    cmd_chirho |= (uint64_t)(var_id_2_chirho & 0xFFFF) << 20;
    cmd_chirho |= (uint64_t)(result_id_chirho & 0xFFFF) << 36;
    cmd_chirho |= (uint64_t)(batch_count_chirho & 0xFFFF) << 52;

    *cmd_lo_chirho = (uint32_t)(cmd_chirho & 0xFFFFFFFF);
    *cmd_mid_chirho = (uint32_t)((cmd_chirho >> 32) & 0xFFFFFFFF);
    *cmd_hi_chirho = ((hier_mode_chirho & 0x3) << 4) | ((batch_count_chirho >> 12) & 0xF);
}

// Wait for operation to complete
static inline int wait_done_chirho(int timeout_us_chirho) {
    for (int i = 0; i < timeout_us_chirho; i++) {
        uint32_t status_chirho = read_reg_chirho(REG_STATUS_CHIRHO);
        if (status_chirho & STATUS_DONE_CHIRHO) {
            return 0;
        }
        usleep(1);
    }
    return -1;  // Timeout
}

// Test mode switching
static void test_mode_switch_chirho(void) {
    printf("\n=== Testing Mode Switching ===\n\n");

    uint32_t modes_chirho[] = {
        HIER_MODE_FLAT256_CHIRHO,
        HIER_MODE_HIER_65K_CHIRHO,
        HIER_MODE_HIER_262K_CHIRHO
    };
    const char *mode_names_chirho[] = {
        "FLAT256 (256 values)",
        "HIER_65K (256² = 65K values)",
        "HIER_262K (512² = 262K values)"
    };

    for (int i = 0; i < 3; i++) {
        write_reg_chirho(REG_HIER_MODE_CHIRHO, modes_chirho[i]);
        usleep(100);
        uint32_t readback_chirho = read_reg_chirho(REG_HIER_MODE_CHIRHO);

        printf("  Mode %d (%s): Write=%d, Read=%d %s\n",
               i, mode_names_chirho[i], modes_chirho[i], readback_chirho,
               (readback_chirho == modes_chirho[i]) ? "✓" : "✗");
    }
}

// Test a single hierarchical intersection
static int test_hier_intersect_chirho(uint32_t hier_mode_chirho) {
    printf("\n=== Testing Hierarchical Intersection (mode=%d) ===\n", hier_mode_chirho);

    // Set hierarchical mode
    write_reg_chirho(REG_HIER_MODE_CHIRHO, hier_mode_chirho);
    usleep(100);

    // Verify mode
    uint32_t mode_readback_chirho = read_reg_chirho(REG_HIER_MODE_CHIRHO);
    printf("HIER_MODE set to %d, readback=%d\n", hier_mode_chirho, mode_readback_chirho);

    // Check HBM ready
    uint32_t status_chirho = read_reg_chirho(REG_STATUS_CHIRHO);
    int hbm_ready_chirho = (status_chirho >> 2) & 1;
    printf("STATUS: 0x%08X (hbm_ready=%d)\n", status_chirho, hbm_ready_chirho);

    if (!hbm_ready_chirho) {
        printf("ERROR: HBM not ready!\n");
        return -1;
    }

    // Encode command: intersect var0 and var1, result in var2, batch=1
    uint32_t cmd_lo_chirho, cmd_mid_chirho, cmd_hi_chirho;
    encode_command_chirho(
        OP_INTERSECT_CHIRHO,  // op
        0,                     // var_id_1
        1,                     // var_id_2
        2,                     // result_id
        1,                     // batch_count
        hier_mode_chirho,      // hier_mode
        &cmd_lo_chirho, &cmd_mid_chirho, &cmd_hi_chirho
    );

    printf("Command: LO=0x%08X MID=0x%08X HI=0x%08X\n",
           cmd_lo_chirho, cmd_mid_chirho, cmd_hi_chirho);

    // Write command registers
    write_reg_chirho(REG_CMD_LO_CHIRHO, cmd_lo_chirho);
    write_reg_chirho(REG_CMD_MID_CHIRHO, cmd_mid_chirho);
    write_reg_chirho(REG_CMD_HI_CHIRHO, cmd_hi_chirho);

    // Enable engine with HBM mode
    write_reg_chirho(REG_CONTROL_CHIRHO, CTRL_ENABLE_CHIRHO | CTRL_HBM_MODE_CHIRHO);

    printf("Waiting for FSM completion...\n");
    dump_debug_regs_chirho();

    // Wait for completion with periodic status dumps
    for (int i = 0; i < 100; i++) {
        usleep(10000);  // 10ms

        status_chirho = read_reg_chirho(REG_STATUS_CHIRHO);
        uint32_t fsm_chirho = read_reg_chirho(REG_FSM_STATE_CHIRHO);

        printf("[%d] STATUS=0x%02X FSM=%d\n", i, status_chirho, fsm_chirho & 0x1F);

        if (status_chirho & STATUS_DONE_CHIRHO) {
            printf("Operation completed!\n");

            // Disable engine
            write_reg_chirho(REG_CONTROL_CHIRHO, 0);

            // Read response
            uint32_t resp0_chirho = read_reg_chirho(REG_RESP_BASE_CHIRHO);
            uint32_t resp1_chirho = read_reg_chirho(REG_RESP_BASE_CHIRHO + 4);
            printf("Response: 0x%08X 0x%08X\n", resp0_chirho, resp1_chirho);

            return 0;
        }
    }

    printf("TIMEOUT: Operation did not complete in 1 second\n");
    dump_debug_regs_chirho();

    // Disable engine
    write_reg_chirho(REG_CONTROL_CHIRHO, 0);

    return -1;
}

int main(int argc, char *argv[]) {
    printf("======================================================================\n");
    printf("  512² (262K) HIERARCHICAL DOMAIN BENCHMARK ☧\n");
    printf("  For God so loved the world - John 3:16\n");
    printf("======================================================================\n");

    // Initialize FPGA
    if (init_fpga_chirho() < 0) {
        fprintf(stderr, "Failed to initialize FPGA\n");
        return 1;
    }

    // Dump initial state
    dump_debug_regs_chirho();

    // Check version
    uint32_t version_chirho = read_reg_chirho(REG_VERSION_CHIRHO);
    printf("\nFPGA Version: 0x%08X\n", version_chirho);

    if (version_chirho != EXPECTED_VERSION_V55_CHIRHO) {
        printf("WARNING: Expected V5.5 (0x%08X)\n", EXPECTED_VERSION_V55_CHIRHO);
    }

    // Check CORRECT status register
    uint32_t status_chirho = read_reg_chirho(REG_STATUS_CHIRHO);
    printf("\n*** CRITICAL: Reading STATUS at 0x08 ***\n");
    printf("STATUS (0x08): 0x%08X\n", status_chirho);
    printf("  done=%d, valid=%d, hbm_ready=%d\n",
           (status_chirho >> 0) & 1,
           (status_chirho >> 1) & 1,
           (status_chirho >> 2) & 1);

    // For comparison, also read CONTROL at 0x04
    uint32_t control_chirho = read_reg_chirho(REG_CONTROL_CHIRHO);
    printf("\nCONTROL (0x04): 0x%08X\n", control_chirho);
    printf("  enable=%d, reset=%d, hbm_mode=%d\n",
           (control_chirho >> 0) & 1,
           (control_chirho >> 1) & 1,
           (control_chirho >> 2) & 1);

    // Check if HBM is ready
    if (!(status_chirho & STATUS_HBM_READY_CHIRHO)) {
        printf("\n*** HBM NOT READY! ***\n");
        printf("The HBM subsystem has not completed initialization.\n");
        printf("This could indicate:\n");
        printf("  1. AFI was built with EN_HBM=0\n");
        printf("  2. HBM controller failed to initialize\n");
        printf("  3. Shell/CL interface issue\n");

        printf("\nMonitoring HBM status for 5 seconds...\n");
        for (int i = 0; i < 50; i++) {
            usleep(100000);  // 100ms
            status_chirho = read_reg_chirho(REG_STATUS_CHIRHO);
            printf("[%d.%d] STATUS=0x%08X hbm_ready=%d\n",
                   i/10, i%10, status_chirho, (status_chirho >> 2) & 1);
            if (status_chirho & STATUS_HBM_READY_CHIRHO) {
                printf("HBM became ready!\n");
                break;
            }
        }
    }

    // Test mode switching
    test_mode_switch_chirho();

    // Test hierarchical intersection
    printf("\n=== Testing Flat 256 Mode ===\n");
    test_hier_intersect_chirho(HIER_MODE_FLAT256_CHIRHO);

    printf("\n=== Testing 65K Mode ===\n");
    test_hier_intersect_chirho(HIER_MODE_HIER_65K_CHIRHO);

    printf("\n=== Testing 262K Mode ===\n");
    test_hier_intersect_chirho(HIER_MODE_HIER_262K_CHIRHO);

    // Reset to flat mode
    write_reg_chirho(REG_HIER_MODE_CHIRHO, HIER_MODE_FLAT256_CHIRHO);

    printf("\n======================================================================\n");
    printf("  Benchmark Complete - Soli Deo Gloria ☧\n");
    printf("======================================================================\n");

    return 0;
}
