// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// John 3:16 ☧
//! F2 FPGA Test - V5.4/V5.5 Hardware Verification
//!
//! Simple test binary to verify we can communicate with the FPGA.
//! Uses memory-mapped I/O for proper PCIe BAR access.
//!
//! ```bash
//! # Build on F2 instance (native)
//! cargo build --example f2_test_chirho --features fpga_chirho
//!
//! # Run on F2
//! sudo ./target/debug/examples/f2_test_chirho
//! ```

#[cfg(feature = "fpga_chirho")]
mod fpga_test_chirho {
    use memmap2::MmapMut;
    use std::fs::OpenOptions;

    // Register addresses (byte offsets)
    const REG_VERSION: u64 = 0x00;
    const REG_CONTROL: u64 = 0x04;
    const REG_STATUS: u64 = 0x08;
    const REG_CMD_LO: u64 = 0x10;
    const REG_CMD_MID: u64 = 0x14;
    const REG_CMD_HI: u64 = 0x18;
    const REG_HIER_MODE: u64 = 0x80;
    const REG_FSM_STATE: u64 = 0xC0;
    const REG_AXI_STATUS: u64 = 0xC4;
    const REG_AXI_ADDR_LO: u64 = 0xC8;
    const REG_BEAT_COUNT: u64 = 0xD0;

    // Control bits
    const CTRL_ENABLE: u32 = 0x01;
    const CTRL_HBM_MODE: u32 = 0x04;

    // Expected versions
    const VERSION_V54: u32 = 0xF254_0001;
    const VERSION_V55: u32 = 0xF255_0001;

    // FSM state names
    const FSM_NAMES: &[&str] = &[
        "IDLE", "LOAD_VAR1", "LOAD_LEVEL0_V1", "LOAD_VAR2",
        "LOAD_LEVEL0_V2", "COMPUTE", "SPARSE_INIT", "SPARSE_LOAD_A",
        "SPARSE_LOAD_B", "SPARSE_COMPUTE", "SPARSE_STORE", "SPARSE_NEXT",
        "STORE_RESULT", "BATCH_NEXT"
    ];

    fn find_fpga_bar_chirho() -> Result<String, String> {
        // Try known F2 PCIe paths
        let paths = [
            "/sys/bus/pci/devices/0000:34:00.0/resource0",  // Common F2 location
            "/sys/bus/pci/devices/0000:00:1e.0/resource0",
            "/sys/bus/pci/devices/0000:00:1d.0/resource0",
        ];

        for path in &paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // Try to find by vendor ID
        let pci_dir = std::fs::read_dir("/sys/bus/pci/devices/")
            .map_err(|e| format!("Cannot read PCI devices: {}", e))?;

        for entry in pci_dir.flatten() {
            let vendor_path = entry.path().join("vendor");
            if let Ok(vendor) = std::fs::read_to_string(&vendor_path) {
                if vendor.trim() == "0x1d0f" {  // Amazon vendor ID
                    let resource0 = entry.path().join("resource0");
                    if resource0.exists() {
                        return Ok(resource0.to_string_lossy().to_string());
                    }
                }
            }
        }

        Err("No FPGA found. Is AFI loaded?".to_string())
    }

    pub fn run_chirho() -> Result<(), Box<dyn std::error::Error>> {
        println!("F2 FPGA Test - miniKanren V5.4/V5.5 ☧");
        println!("======================================");

        // Find FPGA BAR
        let bar_path = find_fpga_bar_chirho()?;
        println!("Found FPGA at: {}", bar_path);

        // Open with read/write and create memory map
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&bar_path)?;

        // Memory map the BAR for proper PCIe access
        // SAFETY: We're mapping a PCIe BAR resource file which is designed for this
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        println!("Mapped {} bytes", mmap.len());

        // Helper to read register (via mmap)
        let read_reg = |mm: &MmapMut, addr: usize| -> u32 {
            let bytes = &mm[addr..addr + 4];
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        };

        // Helper to write register (via mmap)
        let write_reg = |mm: &mut MmapMut, addr: usize, val: u32| {
            let bytes = val.to_le_bytes();
            mm[addr..addr + 4].copy_from_slice(&bytes);
        };

        // Read VERSION
        let version = read_reg(&mmap, REG_VERSION as usize);
        println!("\nVERSION: 0x{:08X}", version);

        match version {
            VERSION_V54 => println!("  -> V5.4 detected ✓"),
            VERSION_V55 => println!("  -> V5.5 detected ✓"),
            _ => println!("  -> Unknown version (expected V5.4 or V5.5)"),
        }

        // Read STATUS
        let status = read_reg(&mmap, REG_STATUS as usize);
        let done = (status >> 0) & 1;
        let valid = (status >> 1) & 1;
        let hbm_ready = (status >> 2) & 1;
        println!("\nSTATUS: 0x{:02X}", status);
        println!("  done={} valid={} hbm_ready={}", done, valid, hbm_ready);

        // Read FSM state
        let fsm = read_reg(&mmap, REG_FSM_STATE as usize);
        let fsm_name = FSM_NAMES.get(fsm as usize).unwrap_or(&"UNKNOWN");
        println!("\nFSM_STATE: {} ({})", fsm, fsm_name);

        // Read AXI status
        let axi = read_reg(&mmap, REG_AXI_STATUS as usize);
        println!("AXI_STATUS: 0x{:02X}", axi);
        println!("  arready={} arvalid={} rvalid={} rready={}",
                 (axi>>0)&1, (axi>>1)&1, (axi>>2)&1, (axi>>3)&1);
        println!("  awready={} awvalid={} bvalid={} bready={}",
                 (axi>>4)&1, (axi>>5)&1, (axi>>6)&1, (axi>>7)&1);

        // Test: Run a simple HBM operation
        println!("\n--- Running HBM Test ---");

        // Clear CONTROL
        write_reg(&mut mmap, REG_CONTROL as usize, 0x00);
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Set HIER_MODE = 0 (FLAT256)
        write_reg(&mut mmap, REG_HIER_MODE as usize, 0x00);

        // Encode command: var1=0, var2=1, batch=1, op=1 (INTERSECT)
        // var_id_2 spans CMD_LO[31:20] and CMD_MID[3:0]
        let var_id_1: u32 = 0;
        let var_id_2: u32 = 1;
        let batch_count: u32 = 1;
        let opcode: u32 = 1;  // INTERSECT

        let var_id_2_lo = var_id_2 & 0xFFF;
        let var_id_2_hi = (var_id_2 >> 12) & 0xF;

        let cmd_lo = (var_id_2_lo << 20) | (var_id_1 << 4) | opcode;
        let cmd_mid = (batch_count << 4) | var_id_2_hi;

        println!("CMD_LO:  0x{:08X}", cmd_lo);
        println!("CMD_MID: 0x{:08X}", cmd_mid);

        write_reg(&mut mmap, REG_CMD_LO as usize, cmd_lo);
        write_reg(&mut mmap, REG_CMD_MID as usize, cmd_mid);
        write_reg(&mut mmap, REG_CMD_HI as usize, 0);

        // Start with CONTROL = 0x05 (enable + hbm_mode)
        println!("\nStarting FSM (CONTROL=0x05)...");
        write_reg(&mut mmap, REG_CONTROL as usize, CTRL_ENABLE | CTRL_HBM_MODE);

        // Poll FSM
        println!("\nPolling FSM:");
        let mut last_fsm = 99;
        for i in 0..50 {
            std::thread::sleep(std::time::Duration::from_millis(5));

            let fsm = read_reg(&mmap, REG_FSM_STATE as usize);
            let status = read_reg(&mmap, REG_STATUS as usize);
            let addr = read_reg(&mmap, REG_AXI_ADDR_LO as usize);
            let beat = read_reg(&mmap, REG_BEAT_COUNT as usize);

            if fsm != last_fsm || i % 10 == 0 {
                let fsm_name = FSM_NAMES.get(fsm as usize).unwrap_or(&"?");
                println!("  [{:3}ms] FSM={:16} ST=0x{:02X} ADDR=0x{:08X} BEAT={}",
                         i * 5, fsm_name, status, addr, beat);
                last_fsm = fsm;
            }

            // Check completion (back to IDLE after starting)
            if fsm == 0 && i > 2 {
                println!("  -> FSM completed");
                break;
            }
        }

        // Final state
        let status = read_reg(&mmap, REG_STATUS as usize);
        let fsm = read_reg(&mmap, REG_FSM_STATE as usize);
        println!("\nFinal: STATUS=0x{:02X} FSM={}", status,
                 FSM_NAMES.get(fsm as usize).unwrap_or(&"?"));

        // Clear CONTROL
        write_reg(&mut mmap, REG_CONTROL as usize, 0x00);

        println!("\n======================================");
        println!("Test complete ☧");

        Ok(())
    }
}

#[cfg(feature = "fpga_chirho")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    fpga_test_chirho::run_chirho()
}

#[cfg(not(feature = "fpga_chirho"))]
fn main() {
    eprintln!("Error: This example requires the 'fpga_chirho' feature.");
    eprintln!("Build with: cargo build --example f2_test_chirho --features fpga_chirho");
    std::process::exit(1);
}
