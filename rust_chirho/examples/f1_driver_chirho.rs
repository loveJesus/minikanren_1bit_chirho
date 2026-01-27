// ============================================================================
// miniKanren F1 FPGA Driver ☧
// ============================================================================
// For God so loved the world, that he gave his only begotten Son,
// that whosoever believeth in him should not perish, but have everlasting life.
// John 3:16
// ============================================================================
//
// This driver communicates with the miniKanren search engine running on
// AWS F1 FPGA via the OCL (OpenCL-like) AXI-Lite interface.
//
// Usage:
//   cargo run --example f1_driver_chirho
//
// Prerequisites:
//   - Running on an F1 instance
//   - AFI loaded: sudo fpga-load-local-image -S 0 -I agfi-038ca2f7a81352cb4
// ============================================================================

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// OCL Register Map for miniKanren Engine
mod registers_chirho {
    // Command registers (write)
    pub const CMD_LOW_CHIRHO: u64 = 0x00;      // Command bits [31:0]
    pub const CMD_MID_CHIRHO: u64 = 0x04;      // Command bits [63:32]
    pub const CMD_HIGH_CHIRHO: u64 = 0x08;     // Command bits [69:64] + padding
    pub const CTRL_CHIRHO: u64 = 0x10;         // Control: bit 0 = enable

    // Response registers (read)
    pub const RESP_0_CHIRHO: u64 = 0x100;      // Response bits [31:0]
    pub const RESP_1_CHIRHO: u64 = 0x104;      // Response bits [63:32]
    pub const RESP_2_CHIRHO: u64 = 0x108;      // Response bits [95:64]
    pub const RESP_3_CHIRHO: u64 = 0x10C;      // Response bits [127:96]
    // ... up to 512 bits of response

    // Status registers (read)
    pub const HELLO_CHIRHO: u64 = 0x500;       // Should return magic value
    pub const STATUS_CHIRHO: u64 = 0x504;      // Engine status
    pub const RESP_COUNT_CHIRHO: u64 = 0x508;  // Number of responses ready
}

/// F1 FPGA Driver
pub struct F1DriverChirho {
    ocl_fd_chirho: std::fs::File,
    slot_chirho: u32,
}

impl F1DriverChirho {
    /// Open connection to F1 FPGA
    pub fn new_chirho(slot_chirho: u32) -> std::io::Result<Self> {
        // The OCL BAR is exposed via the FPGA resource file
        let path_chirho = format!("/sys/bus/pci/devices/0000:00:1d.0/resource0");

        // Try alternative paths
        let paths_chirho = [
            path_chirho.clone(),
            format!("/dev/xdma{}_user", slot_chirho),
            format!("/dev/fpga{}_ocl", slot_chirho),
        ];

        let mut ocl_fd_chirho = None;
        for path_chirho in &paths_chirho {
            if Path::new(path_chirho).exists() {
                match OpenOptions::new().read(true).write(true).open(path_chirho) {
                    Ok(fd_chirho) => {
                        ocl_fd_chirho = Some(fd_chirho);
                        println!("Opened FPGA at: {}", path_chirho);
                        break;
                    }
                    Err(e_chirho) => {
                        eprintln!("Failed to open {}: {}", path_chirho, e_chirho);
                    }
                }
            }
        }

        let ocl_fd_chirho = ocl_fd_chirho.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find FPGA OCL interface. Make sure AFI is loaded.",
            )
        })?;

        Ok(Self {
            ocl_fd_chirho,
            slot_chirho,
        })
    }

    /// Read a 32-bit register
    pub fn read_reg_chirho(&mut self, offset_chirho: u64) -> std::io::Result<u32> {
        self.ocl_fd_chirho.seek(SeekFrom::Start(offset_chirho))?;
        let mut buf_chirho = [0u8; 4];
        self.ocl_fd_chirho.read_exact(&mut buf_chirho)?;
        Ok(u32::from_le_bytes(buf_chirho))
    }

    /// Write a 32-bit register
    pub fn write_reg_chirho(&mut self, offset_chirho: u64, value_chirho: u32) -> std::io::Result<()> {
        self.ocl_fd_chirho.seek(SeekFrom::Start(offset_chirho))?;
        self.ocl_fd_chirho.write_all(&value_chirho.to_le_bytes())?;
        Ok(())
    }

    /// Check if the FPGA is responding
    pub fn check_hello_chirho(&mut self) -> std::io::Result<bool> {
        let hello_chirho = self.read_reg_chirho(registers_chirho::HELLO_CHIRHO)?;
        // Our design should return a recognizable pattern
        println!("Hello register: 0x{:08X}", hello_chirho);
        Ok(hello_chirho != 0 && hello_chirho != 0xFFFFFFFF)
    }

    /// Send a 70-bit command to the search engine
    pub fn send_command_chirho(&mut self, cmd_chirho: &[u8; 9]) -> std::io::Result<()> {
        // Pack 70 bits into three 32-bit registers
        let low_chirho = u32::from_le_bytes([cmd_chirho[0], cmd_chirho[1], cmd_chirho[2], cmd_chirho[3]]);
        let mid_chirho = u32::from_le_bytes([cmd_chirho[4], cmd_chirho[5], cmd_chirho[6], cmd_chirho[7]]);
        let high_chirho = cmd_chirho[8] as u32 & 0x3F; // Only 6 bits

        self.write_reg_chirho(registers_chirho::CMD_LOW_CHIRHO, low_chirho)?;
        self.write_reg_chirho(registers_chirho::CMD_MID_CHIRHO, mid_chirho)?;
        self.write_reg_chirho(registers_chirho::CMD_HIGH_CHIRHO, high_chirho)?;

        // Enable the engine
        self.write_reg_chirho(registers_chirho::CTRL_CHIRHO, 1)?;

        Ok(())
    }

    /// Read the 512-bit response
    pub fn read_response_chirho(&mut self) -> std::io::Result<[u8; 64]> {
        let mut response_chirho = [0u8; 64];

        for i_chirho in 0..16 {
            let offset_chirho = registers_chirho::RESP_0_CHIRHO + (i_chirho as u64 * 4);
            let word_chirho = self.read_reg_chirho(offset_chirho)?;
            let bytes_chirho = word_chirho.to_le_bytes();
            response_chirho[i_chirho * 4..i_chirho * 4 + 4].copy_from_slice(&bytes_chirho);
        }

        Ok(response_chirho)
    }

    /// Get engine status
    pub fn get_status_chirho(&mut self) -> std::io::Result<u32> {
        self.read_reg_chirho(registers_chirho::STATUS_CHIRHO)
    }

    /// Get number of responses ready
    pub fn get_response_count_chirho(&mut self) -> std::io::Result<u32> {
        self.read_reg_chirho(registers_chirho::RESP_COUNT_CHIRHO)
    }
}

/// Example: Run a simple miniKanren query on FPGA
fn main() {
    println!("=== miniKanren F1 FPGA Driver ☧ ===\n");

    // Try to open the FPGA
    let mut driver_chirho = match F1DriverChirho::new_chirho(0) {
        Ok(d_chirho) => d_chirho,
        Err(e_chirho) => {
            eprintln!("Failed to open FPGA: {}", e_chirho);
            eprintln!("\nMake sure:");
            eprintln!("  1. You're running on an F1 instance");
            eprintln!("  2. AFI is loaded: sudo fpga-load-local-image -S 0 -I agfi-038ca2f7a81352cb4");
            eprintln!("  3. You have permissions (try with sudo)");
            return;
        }
    };

    // Check hello
    println!("Checking FPGA connection...");
    match driver_chirho.check_hello_chirho() {
        Ok(true) => println!("✓ FPGA responding\n"),
        Ok(false) => {
            eprintln!("✗ FPGA not responding correctly");
            return;
        }
        Err(e_chirho) => {
            eprintln!("✗ Error reading FPGA: {}", e_chirho);
            return;
        }
    }

    // Get status
    match driver_chirho.get_status_chirho() {
        Ok(status_chirho) => println!("Status: 0x{:08X}", status_chirho),
        Err(e_chirho) => eprintln!("Error reading status: {}", e_chirho),
    }

    // Send a test command
    // This is a placeholder - real commands would encode miniKanren goals
    println!("\nSending test command...");
    let test_cmd_chirho = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x3F];
    if let Err(e_chirho) = driver_chirho.send_command_chirho(&test_cmd_chirho) {
        eprintln!("Error sending command: {}", e_chirho);
        return;
    }
    println!("✓ Command sent");

    // Wait a bit for processing
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Read response
    println!("\nReading response...");
    match driver_chirho.read_response_chirho() {
        Ok(response_chirho) => {
            println!("Response (first 32 bytes):");
            for (i_chirho, chunk_chirho) in response_chirho[..32].chunks(8).enumerate() {
                print!("  [{:02}]: ", i_chirho * 8);
                for byte_chirho in chunk_chirho {
                    print!("{:02X} ", byte_chirho);
                }
                println!();
            }
        }
        Err(e_chirho) => eprintln!("Error reading response: {}", e_chirho),
    }

    // Get response count
    match driver_chirho.get_response_count_chirho() {
        Ok(count_chirho) => println!("\nResponses ready: {}", count_chirho),
        Err(e_chirho) => eprintln!("Error reading response count: {}", e_chirho),
    }

    println!("\n=== Test Complete ☧ ===");
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_register_offsets_chirho() {
        // Verify register offsets are 4-byte aligned
        assert_eq!(registers_chirho::CMD_LOW_CHIRHO % 4, 0);
        assert_eq!(registers_chirho::CMD_MID_CHIRHO % 4, 0);
        assert_eq!(registers_chirho::CTRL_CHIRHO % 4, 0);
        assert_eq!(registers_chirho::RESP_0_CHIRHO % 4, 0);
        assert_eq!(registers_chirho::HELLO_CHIRHO % 4, 0);
    }
}
