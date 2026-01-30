// For God so loved the world that He gave His only begotten Son ☧
//! FPGA Backend Implementation
//!
//! Connects to AWS F2 instance via PCIe memory-mapped I/O.
//! Feature-gated: only compiled with `fpga_chirho` feature.
//!
//! # Hardware Communication
//!
//! On Linux with FPGA hardware:
//! - Uses memmap2 to map PCIe BAR0 into process memory
//! - Reads/writes to FPGA registers via volatile memory access
//!
//! For testing without hardware:
//! - Set `FPGA_MOCK_CHIRHO=1` environment variable
//! - Operations return expected values without hardware

use super::trait_chirho::{
    BackendInfoChirho, ConstraintChirho, DomainVecChirho, SolutionChirho, SolverBackendChirho,
};

#[cfg(target_os = "linux")]
use memmap2::MmapMut;
#[cfg(target_os = "linux")]
use std::fs::OpenOptions;

// Register addresses - generated from cl_minikanren_chirho_defines.vh at build time
// Source of truth: synth_chirho/.../cl_minikanren_chirho_defines.vh
include!(concat!(env!("OUT_DIR"), "/fpga_regs_chirho.rs"));

// Expected version (matches MINIKANREN_VERSION_CHIRHO in defines.vh)
// Format: 0xF2_VV_0001 where VV = version (54=V5.4, 55=V5.5)
// V5.4: 0xF2540001, V5.5: 0xF2550001
const EXPECTED_VERSION_V54_CHIRHO: u32 = 0xF254_0001;
const EXPECTED_VERSION_V55_CHIRHO: u32 = 0xF255_0001;

// Control register bits
const CTRL_ENABLE_CHIRHO: u32 = 0x01;
const CTRL_RESET_CHIRHO: u32 = 0x02;
const CTRL_HBM_MODE_CHIRHO: u32 = 0x04;

// Status register bits
const STATUS_DONE_CHIRHO: u32 = 0x01;
const STATUS_VALID_CHIRHO: u32 = 0x02;
const STATUS_HBM_READY_CHIRHO: u32 = 0x04;

/// FPGA backend using AWS F2
///
/// Communicates with the FPGA via:
/// - Register reads/writes for simple operations (PCIe BAR0)
/// - DMA for batch operations (HBM) - future
#[derive(Debug)]
pub struct FpgaBackendChirho {
    /// FPGA slot (usually 0)
    slot_chirho: u32,
    /// Whether HBM batch mode is available
    hbm_available_chirho: bool,
    /// Cached version for health checks
    version_chirho: u32,
    /// Memory-mapped PCIe BAR0 (Linux only)
    #[cfg(target_os = "linux")]
    bar0_mmap_chirho: Option<MmapMut>,
    /// Mock mode (for testing without hardware)
    mock_mode_chirho: bool,
}

impl FpgaBackendChirho {
    /// Connect to FPGA
    ///
    /// # Errors
    /// Returns error if:
    /// - No FPGA found
    /// - Version mismatch (wrong AFI loaded)
    /// - Permission denied
    pub fn connect_chirho() -> Result<Self, String> {
        let mock_mode_chirho = std::env::var("FPGA_MOCK_CHIRHO").is_ok();
        let slot_chirho = 0;

        // Try to open FPGA hardware (Linux only)
        #[cfg(target_os = "linux")]
        let bar0_mmap_chirho = if !mock_mode_chirho {
            Some(Self::open_pcie_bar_chirho(slot_chirho)?)
        } else {
            None
        };

        // Create instance
        let mut instance_chirho = Self {
            slot_chirho,
            hbm_available_chirho: false,
            version_chirho: 0,
            #[cfg(target_os = "linux")]
            bar0_mmap_chirho,
            mock_mode_chirho,
        };

        // Read and verify version
        let version_chirho = instance_chirho.read_reg_chirho(REG_VERSION_CHIRHO);

        // Accept V5.4 or V5.5 versions (or any V5.x with F2 prefix)
        let is_valid_version_chirho = mock_mode_chirho
            || version_chirho == EXPECTED_VERSION_V54_CHIRHO
            || version_chirho == EXPECTED_VERSION_V55_CHIRHO
            || (version_chirho & 0xFF00_0000) == 0xF200_0000;  // Any F2 version

        if !is_valid_version_chirho {
            return Err(format!(
                "Version mismatch: expected V5.4 (0x{:08X}) or V5.5 (0x{:08X}), got 0x{:08X}. Wrong AFI loaded?",
                EXPECTED_VERSION_V54_CHIRHO, EXPECTED_VERSION_V55_CHIRHO, version_chirho
            ));
        }

        instance_chirho.version_chirho = if mock_mode_chirho {
            EXPECTED_VERSION_V55_CHIRHO
        } else {
            version_chirho
        };

        // Check HBM availability
        let status_chirho = instance_chirho.read_reg_chirho(REG_STATUS_CHIRHO);
        instance_chirho.hbm_available_chirho = (status_chirho & 0x01) != 0 || mock_mode_chirho;

        Ok(instance_chirho)
    }

    /// Open PCIe BAR0 via mmap (Linux only)
    #[cfg(target_os = "linux")]
    fn open_pcie_bar_chirho(slot_chirho: u32) -> Result<MmapMut, String> {
        // AWS F2 FPGA PCIe device paths
        // F2 devices are typically at 0000:34:00.0 (varies by instance)
        let pcie_paths_chirho = [
            // F2 common paths (verified on f2.6xlarge)
            "/sys/bus/pci/devices/0000:34:00.0/resource0".to_string(),
            "/sys/bus/pci/devices/0000:00:1e.0/resource0".to_string(),
            format!("/sys/bus/pci/devices/0000:00:1d.{}/resource0", slot_chirho),
            "/dev/fpga0_ocl".to_string(),  // AWS FPGA SDK path
        ];

        for path_chirho in &pcie_paths_chirho {
            if let Ok(file_chirho) = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path_chirho)
            {
                match unsafe { MmapMut::map_mut(&file_chirho) } {
                    Ok(mmap_chirho) => {
                        log::info!("Opened FPGA at {} ☧", path_chirho);
                        return Ok(mmap_chirho);
                    }
                    Err(e_chirho) => {
                        log::debug!("Failed to mmap {}: {}", path_chirho, e_chirho);
                    }
                }
            }
        }

        Err("Could not open FPGA PCIe BAR0. Check permissions and AFI status.".to_string())
    }

    /// Read a 32-bit register
    fn read_reg_chirho(&self, addr_chirho: u64) -> u32 {
        // Mock mode
        if self.mock_mode_chirho {
            return match addr_chirho {
                REG_VERSION_CHIRHO => EXPECTED_VERSION_V55_CHIRHO,
                REG_STATUS_CHIRHO => 0x07, // done=1, valid=1, hbm_ready=1
                _ => 0,
            };
        }

        // Real hardware access (Linux only)
        #[cfg(target_os = "linux")]
        if let Some(ref mmap_chirho) = self.bar0_mmap_chirho {
            let ptr_chirho = mmap_chirho.as_ptr() as *const u32;
            let offset_chirho = (addr_chirho / 4) as usize;
            return unsafe { ptr_chirho.add(offset_chirho).read_volatile() };
        }

        0
    }

    /// Write a 32-bit register
    fn write_reg_chirho(&self, addr_chirho: u64, value_chirho: u32) {
        // Mock mode - no-op
        if self.mock_mode_chirho {
            return;
        }

        // Real hardware access (Linux only)
        #[cfg(target_os = "linux")]
        if let Some(ref mmap_chirho) = self.bar0_mmap_chirho {
            let ptr_chirho = mmap_chirho.as_ptr() as *mut u32;
            let offset_chirho = (addr_chirho / 4) as usize;
            unsafe { ptr_chirho.add(offset_chirho).write_volatile(value_chirho) };
        }
    }

    /// Write a 64-bit value (two registers)
    fn write_reg_64_chirho(&self, addr_chirho: u64, value_chirho: u64) {
        self.write_reg_chirho(addr_chirho, value_chirho as u32);
        self.write_reg_chirho(addr_chirho + 4, (value_chirho >> 32) as u32);
    }

    /// Read a 64-bit value (two registers)
    fn read_reg_64_chirho(&self, addr_chirho: u64) -> u64 {
        let low_chirho = self.read_reg_chirho(addr_chirho) as u64;
        let high_chirho = self.read_reg_chirho(addr_chirho + 4) as u64;
        low_chirho | (high_chirho << 32)
    }

    /// Pack a command into the 70-bit cmd register format
    /// Format: [op:4][var:3][domain_a:32][domain_b:32] (simplified)
    fn pack_cmd_chirho(&self, op_chirho: u8, domain_a_chirho: u64, domain_b_chirho: u64) -> (u32, u32, u32) {
        // CMD_LO: domain_a[31:0]
        let cmd_lo_chirho = domain_a_chirho as u32;
        // CMD_MID: domain_b[31:0]
        let cmd_mid_chirho = domain_b_chirho as u32;
        // CMD_HI: [op:4][var:3] (top bits)
        let cmd_hi_chirho = (op_chirho as u32) & 0x0F;

        (cmd_lo_chirho, cmd_mid_chirho, cmd_hi_chirho)
    }

    /// Execute a command using the actual hardware register interface
    fn execute_cmd_chirho(&self, op_chirho: u8, a_chirho: u64, b_chirho: u64) -> u64 {
        // Pack and write command
        let (lo_chirho, mid_chirho, hi_chirho) = self.pack_cmd_chirho(op_chirho, a_chirho, b_chirho);

        self.write_reg_chirho(REG_CMD_LO_CHIRHO, lo_chirho);
        self.write_reg_chirho(REG_CMD_MID_CHIRHO, mid_chirho);
        self.write_reg_chirho(REG_CMD_HI_CHIRHO, hi_chirho);

        // Enable execution (MUST set both enable AND hbm_mode!)
        self.write_reg_chirho(REG_CONTROL_CHIRHO, CTRL_ENABLE_CHIRHO | CTRL_HBM_MODE_CHIRHO);

        // Poll for completion
        let mut timeout_chirho = 1000;
        while timeout_chirho > 0 {
            let status_chirho = self.read_reg_chirho(REG_STATUS_CHIRHO);
            if status_chirho & STATUS_DONE_CHIRHO != 0 {
                break;
            }
            timeout_chirho -= 1;
        }

        // Read result from response registers
        self.read_reg_64_chirho(REG_RESP_BASE_CHIRHO)
    }
}

// Operation codes (from OP_* in .vh, or local definitions)
const OP_INTERSECT_LOCAL_CHIRHO: u8 = 0x00;
const OP_UNION_LOCAL_CHIRHO: u8 = 0x01;
const OP_COMPLEMENT_LOCAL_CHIRHO: u8 = 0x02;
const OP_POPCOUNT_LOCAL_CHIRHO: u8 = 0x03;

impl SolverBackendChirho for FpgaBackendChirho {
    fn intersect_64_chirho(&self, a_chirho: u64, b_chirho: u64) -> u64 {
        if self.mock_mode_chirho {
            return a_chirho & b_chirho;
        }
        self.execute_cmd_chirho(OP_INTERSECT_LOCAL_CHIRHO, a_chirho, b_chirho)
    }

    fn union_64_chirho(&self, a_chirho: u64, b_chirho: u64) -> u64 {
        if self.mock_mode_chirho {
            return a_chirho | b_chirho;
        }
        self.execute_cmd_chirho(OP_UNION_LOCAL_CHIRHO, a_chirho, b_chirho)
    }

    fn complement_64_chirho(&self, a_chirho: u64) -> u64 {
        if self.mock_mode_chirho {
            return !a_chirho;
        }
        self.execute_cmd_chirho(OP_COMPLEMENT_LOCAL_CHIRHO, a_chirho, 0)
    }

    fn popcount_64_chirho(&self, a_chirho: u64) -> u32 {
        if self.mock_mode_chirho {
            return a_chirho.count_ones();
        }
        self.execute_cmd_chirho(OP_POPCOUNT_LOCAL_CHIRHO, a_chirho, 0) as u32
    }

    fn intersect_batch_chirho(&self, pairs_chirho: &[(u64, u64)]) -> Vec<u64> {
        if !self.hbm_available_chirho || pairs_chirho.len() < 100 {
            // Fall back to sequential for small batches
            return pairs_chirho
                .iter()
                .map(|(a, b)| self.intersect_64_chirho(*a, *b))
                .collect();
        }

        // TODO: Use DMA to write pairs to HBM, trigger batch, read results
        // This is where FPGA really shines with 460 GB/s HBM bandwidth

        // For now, sequential
        pairs_chirho
            .iter()
            .map(|(a, b)| self.intersect_64_chirho(*a, *b))
            .collect()
    }

    fn propagate_chirho(
        &self,
        domains_chirho: &DomainVecChirho,
        constraints_chirho: &[ConstraintChirho],
    ) -> DomainVecChirho {
        // TODO: Offload to FPGA propagation engine
        // For now, do it on CPU (FPGA just accelerates primitives)

        let mut result_chirho = domains_chirho.clone();
        let mut changed_chirho = true;

        while changed_chirho {
            changed_chirho = false;

            for constraint_chirho in constraints_chirho {
                match constraint_chirho {
                    ConstraintChirho::EqualChirho { var_a_chirho, var_b_chirho } => {
                        let intersection_chirho =
                            self.intersect_64_chirho(result_chirho[*var_a_chirho], result_chirho[*var_b_chirho]);
                        if result_chirho[*var_a_chirho] != intersection_chirho {
                            result_chirho[*var_a_chirho] = intersection_chirho;
                            changed_chirho = true;
                        }
                        if result_chirho[*var_b_chirho] != intersection_chirho {
                            result_chirho[*var_b_chirho] = intersection_chirho;
                            changed_chirho = true;
                        }
                    }
                    ConstraintChirho::NotEqualChirho { var_a_chirho, var_b_chirho } => {
                        if result_chirho[*var_a_chirho].count_ones() == 1 {
                            let new_chirho = result_chirho[*var_b_chirho] & !result_chirho[*var_a_chirho];
                            if result_chirho[*var_b_chirho] != new_chirho {
                                result_chirho[*var_b_chirho] = new_chirho;
                                changed_chirho = true;
                            }
                        }
                        if result_chirho[*var_b_chirho].count_ones() == 1 {
                            let new_chirho = result_chirho[*var_a_chirho] & !result_chirho[*var_b_chirho];
                            if result_chirho[*var_a_chirho] != new_chirho {
                                result_chirho[*var_a_chirho] = new_chirho;
                                changed_chirho = true;
                            }
                        }
                    }
                    ConstraintChirho::FixedChirho { var_chirho, value_chirho } => {
                        let fixed_chirho = 1u64 << value_chirho;
                        let new_chirho = self.intersect_64_chirho(result_chirho[*var_chirho], fixed_chirho);
                        if result_chirho[*var_chirho] != new_chirho {
                            result_chirho[*var_chirho] = new_chirho;
                            changed_chirho = true;
                        }
                    }
                    ConstraintChirho::BinaryChirho { .. } => {
                        // TODO: Binary constraint propagation
                    }
                }
            }
        }

        result_chirho
    }

    fn solve_chirho(
        &self,
        domains_chirho: &DomainVecChirho,
        constraints_chirho: &[ConstraintChirho],
        max_solutions_chirho: usize,
    ) -> Vec<SolutionChirho> {
        // TODO: Offload search to FPGA engine
        // For now, use CPU search with FPGA-accelerated primitives

        let mut solutions_chirho = Vec::new();
        let mut stack_chirho: Vec<DomainVecChirho> = vec![domains_chirho.clone()];

        while let Some(mut current_chirho) = stack_chirho.pop() {
            if max_solutions_chirho > 0 && solutions_chirho.len() >= max_solutions_chirho {
                break;
            }

            // Propagate
            current_chirho = self.propagate_chirho(&current_chirho, constraints_chirho);

            // Check for failure
            if current_chirho.iter().any(|d| *d == 0) {
                continue;
            }

            // Check for solution
            if current_chirho.iter().all(|d| d.count_ones() == 1) {
                solutions_chirho.push(SolutionChirho {
                    assignments_chirho: current_chirho
                        .iter()
                        .map(|d| d.trailing_zeros() as u64)
                        .collect(),
                });
                continue;
            }

            // Branch on smallest domain
            let (var_chirho, domain_chirho) = current_chirho
                .iter()
                .enumerate()
                .filter(|(_, d)| d.count_ones() > 1)
                .min_by_key(|(_, d)| d.count_ones())
                .unwrap();

            let mut val_chirho = *domain_chirho;
            while val_chirho != 0 {
                let bit_chirho = val_chirho & val_chirho.wrapping_neg();
                val_chirho &= val_chirho - 1;

                let mut branch_chirho = current_chirho.clone();
                branch_chirho[var_chirho] = bit_chirho;
                stack_chirho.push(branch_chirho);
            }
        }

        solutions_chirho
    }

    fn info_chirho(&self) -> BackendInfoChirho {
        BackendInfoChirho {
            name_chirho: "fpga",
            version_chirho: format!("0x{:08X}", self.version_chirho),
            throughput_estimate_chirho: 100_000_000, // ~100M ops/sec with HBM
            batch_accelerated_chirho: self.hbm_available_chirho,
        }
    }

    fn is_healthy_chirho(&self) -> bool {
        // In mock mode, always healthy
        if self.mock_mode_chirho {
            return true;
        }

        // Re-read version register to check connection
        let version_read_chirho = self.read_reg_chirho(REG_VERSION_CHIRHO);
        version_read_chirho == self.version_chirho
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_mock_connect_chirho() {
        std::env::set_var("FPGA_MOCK_CHIRHO", "1");
        let result_chirho = FpgaBackendChirho::connect_chirho();
        assert!(result_chirho.is_ok());
        std::env::remove_var("FPGA_MOCK_CHIRHO");
    }
}
