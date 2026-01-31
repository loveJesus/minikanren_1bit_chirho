// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// John 3:16 ☧
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
    BackendInfoChirho, ConstraintChirho, DomainVecChirho, SolutionChirho,
    SolverBackendChirho, TrainConfigChirho, TrainResultChirho,
};
use crate::hardware_chirho::hardware_chirho::BitVec256Chirho;

#[cfg(target_os = "linux")]
use memmap2::MmapMut;
#[cfg(target_os = "linux")]
use std::fs::OpenOptions;

// AWS FPGA SDK FFI bindings (optional, for better HBM access)
#[cfg(all(target_os = "linux", feature = "aws_sdk_chirho"))]
mod aws_sdk_ffi_chirho {
    use std::os::raw::c_int;

    extern "C" {
        pub fn fpga_mgmt_init() -> c_int;
        pub fn fpga_pci_attach(slot_id: c_int, pf_id: c_int, bar_id: c_int, flags: u32) -> c_int;
        pub fn fpga_pci_poke(handle: c_int, offset: u64, value: u32) -> c_int;
        pub fn fpga_pci_peek(handle: c_int, offset: u64, value: *mut u32) -> c_int;
        pub fn fpga_pci_write_burst(handle: c_int, offset: u64, data: *const u32, dword_len: u64) -> c_int;
        pub fn fpga_pci_read_burst(handle: c_int, offset: u64, data: *mut u32, dword_len: u64) -> c_int;
    }
}

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

/// HBM batch configuration
#[derive(Debug, Clone, Copy)]
pub struct HbmConfigChirho {
    /// Minimum batch size to use HBM (below this, register path is faster)
    pub min_batch_size_chirho: usize,
    /// Maximum batch size per HBM transfer
    pub max_batch_size_chirho: usize,
    /// HBM base offset for input data
    pub input_offset_chirho: u64,
    /// HBM base offset for output data
    pub output_offset_chirho: u64,
}

impl Default for HbmConfigChirho {
    fn default() -> Self {
        Self {
            min_batch_size_chirho: 100,        // Below 100 pairs, register path is faster
            max_batch_size_chirho: 1_000_000,  // 1M pairs per batch (8MB in, 8MB out)
            input_offset_chirho: 0x0000_0000,  // Start of HBM
            output_offset_chirho: 0x0100_0000, // 16MB offset for outputs
        }
    }
}

/// FPGA backend using AWS F2
///
/// Communicates with the FPGA via:
/// - Register reads/writes for simple operations (PCIe BAR0)
/// - DMA for batch operations (HBM via BAR4)
///
/// # HBM Batch Protocol
///
/// 1. Write input pairs to HBM (BAR4 + input_offset)
/// 2. Write batch count to REG_HBM_COUNT_CHIRHO
/// 3. Set CTRL_HBM_MODE and CTRL_ENABLE
/// 4. Poll STATUS_DONE
/// 5. Read results from HBM (BAR4 + output_offset)
#[derive(Debug)]
pub struct FpgaBackendChirho {
    /// FPGA slot (usually 0)
    slot_chirho: u32,
    /// Whether HBM batch mode is available
    hbm_available_chirho: bool,
    /// Cached version for health checks
    version_chirho: u32,
    /// Memory-mapped PCIe BAR0 (registers, 64MB)
    #[cfg(target_os = "linux")]
    bar0_mmap_chirho: Option<MmapMut>,
    /// Memory-mapped PCIe BAR4 (HBM, 128GB)
    #[cfg(target_os = "linux")]
    bar4_mmap_chirho: Option<MmapMut>,
    /// HBM configuration
    hbm_config_chirho: HbmConfigChirho,
    /// AWS SDK handle (if using SDK instead of mmap)
    #[cfg(all(target_os = "linux", feature = "aws_sdk_chirho"))]
    sdk_handle_chirho: Option<i32>,
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
        Self::connect_with_config_chirho(HbmConfigChirho::default())
    }

    /// Connect with custom HBM configuration
    pub fn connect_with_config_chirho(hbm_config_chirho: HbmConfigChirho) -> Result<Self, String> {
        let mock_mode_chirho = std::env::var("FPGA_MOCK_CHIRHO").is_ok();
        let slot_chirho = 0;

        // Try AWS SDK first (feature-gated)
        #[cfg(all(target_os = "linux", feature = "aws_sdk_chirho"))]
        let sdk_handle_chirho = if !mock_mode_chirho {
            Self::init_aws_sdk_chirho(slot_chirho).ok()
        } else {
            None
        };

        // Fall back to mmap for BAR0 (Linux only)
        #[cfg(target_os = "linux")]
        let bar0_mmap_chirho = if !mock_mode_chirho {
            #[cfg(feature = "aws_sdk_chirho")]
            if sdk_handle_chirho.is_some() {
                None // Using SDK, don't need mmap
            } else {
                Some(Self::open_pcie_bar_chirho(slot_chirho, 0)?)
            }
            #[cfg(not(feature = "aws_sdk_chirho"))]
            Some(Self::open_pcie_bar_chirho(slot_chirho, 0)?)
        } else {
            None
        };

        // Try to open BAR4 for HBM (Linux only)
        #[cfg(target_os = "linux")]
        let bar4_mmap_chirho = if !mock_mode_chirho {
            Self::open_pcie_bar_chirho(slot_chirho, 4).ok()
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
            #[cfg(target_os = "linux")]
            bar4_mmap_chirho,
            hbm_config_chirho,
            #[cfg(all(target_os = "linux", feature = "aws_sdk_chirho"))]
            sdk_handle_chirho,
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

        // Check HBM availability (STATUS_HBM_READY bit + BAR4 mapped)
        #[cfg(target_os = "linux")]
        {
            let status_chirho = instance_chirho.read_reg_chirho(REG_STATUS_CHIRHO);
            instance_chirho.hbm_available_chirho = ((status_chirho & STATUS_HBM_READY_CHIRHO) != 0
                && instance_chirho.bar4_mmap_chirho.is_some())
                || mock_mode_chirho;
        }
        #[cfg(not(target_os = "linux"))]
        {
            instance_chirho.hbm_available_chirho = mock_mode_chirho;
        }

        if instance_chirho.hbm_available_chirho {
            log::info!("HBM batch mode available ☧");
        } else {
            log::info!("HBM not available, using register path ☧");
        }

        Ok(instance_chirho)
    }

    /// Initialize AWS FPGA SDK (feature-gated)
    #[cfg(all(target_os = "linux", feature = "aws_sdk_chirho"))]
    fn init_aws_sdk_chirho(slot_chirho: u32) -> Result<i32, String> {
        use aws_sdk_ffi_chirho::*;
        unsafe {
            let rc_chirho = fpga_mgmt_init();
            if rc_chirho != 0 {
                return Err(format!("fpga_mgmt_init failed: {}", rc_chirho));
            }
            let handle_chirho = fpga_pci_attach(slot_chirho as i32, 0, 0, 0);
            if handle_chirho < 0 {
                return Err(format!("fpga_pci_attach failed: {}", handle_chirho));
            }
            Ok(handle_chirho)
        }
    }

    /// Open PCIe BAR via mmap (Linux only)
    ///
    /// # Arguments
    /// - `slot_chirho`: FPGA slot (usually 0)
    /// - `bar_num_chirho`: BAR number (0=registers, 4=HBM)
    #[cfg(target_os = "linux")]
    fn open_pcie_bar_chirho(slot_chirho: u32, bar_num_chirho: u32) -> Result<MmapMut, String> {
        // AWS F2 FPGA PCIe device paths
        // F2 devices are typically at 0000:34:00.0 (varies by instance)
        let resource_name_chirho = format!("resource{}", bar_num_chirho);
        let wc_suffix_chirho = if bar_num_chirho == 0 { "_wc" } else { "" }; // Write-combine for BAR0

        let pcie_paths_chirho = [
            // F2 common paths (verified on f2.6xlarge)
            format!("/sys/bus/pci/devices/0000:34:00.0/{}{}", resource_name_chirho, wc_suffix_chirho),
            format!("/sys/bus/pci/devices/0000:34:00.0/{}", resource_name_chirho),
            format!("/sys/bus/pci/devices/0000:00:1e.0/{}", resource_name_chirho),
            format!("/sys/bus/pci/devices/0000:00:1d.{}/{}", slot_chirho, resource_name_chirho),
            // AWS FPGA SDK paths
            if bar_num_chirho == 0 { "/dev/fpga0_ocl".to_string() } else { format!("/dev/fpga0_bar{}", bar_num_chirho) },
        ];

        for path_chirho in &pcie_paths_chirho {
            if let Ok(file_chirho) = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path_chirho)
            {
                match unsafe { MmapMut::map_mut(&file_chirho) } {
                    Ok(mmap_chirho) => {
                        log::info!("Opened FPGA BAR{} at {} ☧", bar_num_chirho, path_chirho);
                        return Ok(mmap_chirho);
                    }
                    Err(e_chirho) => {
                        log::debug!("Failed to mmap {}: {}", path_chirho, e_chirho);
                    }
                }
            }
        }

        Err(format!("Could not open FPGA PCIe BAR{}. Check permissions and AFI status.", bar_num_chirho))
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

    // ========================================================================
    // HBM Batch Operations ☧
    // ========================================================================

    /// Write data to HBM (BAR4)
    #[cfg(target_os = "linux")]
    fn write_hbm_chirho(&self, offset_chirho: u64, data_chirho: &[u64]) -> Result<(), String> {
        if let Some(ref mmap_chirho) = self.bar4_mmap_chirho {
            let ptr_chirho = mmap_chirho.as_ptr() as *mut u64;
            let base_offset_chirho = (offset_chirho / 8) as usize;

            for (i_chirho, val_chirho) in data_chirho.iter().enumerate() {
                unsafe {
                    ptr_chirho.add(base_offset_chirho + i_chirho).write_volatile(*val_chirho);
                }
            }
            Ok(())
        } else {
            Err("HBM BAR4 not mapped".to_string())
        }
    }

    /// Read data from HBM (BAR4)
    #[cfg(target_os = "linux")]
    fn read_hbm_chirho(&self, offset_chirho: u64, count_chirho: usize) -> Result<Vec<u64>, String> {
        if let Some(ref mmap_chirho) = self.bar4_mmap_chirho {
            let ptr_chirho = mmap_chirho.as_ptr() as *const u64;
            let base_offset_chirho = (offset_chirho / 8) as usize;

            let mut result_chirho = Vec::with_capacity(count_chirho);
            for i_chirho in 0..count_chirho {
                let val_chirho = unsafe { ptr_chirho.add(base_offset_chirho + i_chirho).read_volatile() };
                result_chirho.push(val_chirho);
            }
            Ok(result_chirho)
        } else {
            Err("HBM BAR4 not mapped".to_string())
        }
    }

    /// Execute batch intersect using HBM
    ///
    /// # Protocol
    /// 1. Write pairs to HBM input region (interleaved: a0, b0, a1, b1, ...)
    /// 2. Write count to REG_HBM_COUNT
    /// 3. Set HBM_MODE | ENABLE
    /// 4. Poll STATUS_DONE
    /// 5. Read results from HBM output region
    #[cfg(target_os = "linux")]
    fn execute_hbm_batch_chirho(&self, pairs_chirho: &[(u64, u64)]) -> Result<Vec<u64>, String> {
        let count_chirho = pairs_chirho.len();
        if count_chirho > self.hbm_config_chirho.max_batch_size_chirho {
            return Err(format!(
                "Batch size {} exceeds max {}",
                count_chirho, self.hbm_config_chirho.max_batch_size_chirho
            ));
        }

        // Step 1: Write pairs to HBM (interleaved format)
        let mut input_data_chirho = Vec::with_capacity(count_chirho * 2);
        for (a_chirho, b_chirho) in pairs_chirho {
            input_data_chirho.push(*a_chirho);
            input_data_chirho.push(*b_chirho);
        }
        self.write_hbm_chirho(self.hbm_config_chirho.input_offset_chirho, &input_data_chirho)?;

        // Step 2: Write count and start
        self.write_reg_chirho(REG_HBM_COUNT_CHIRHO, count_chirho as u32);

        // Step 3: Enable HBM batch mode
        self.write_reg_chirho(REG_CONTROL_CHIRHO, CTRL_HBM_MODE_CHIRHO | CTRL_ENABLE_CHIRHO);

        // Step 4: Poll for completion (with timeout)
        let mut timeout_chirho = 100_000; // ~100ms at 1μs per iter
        while timeout_chirho > 0 {
            let status_chirho = self.read_reg_chirho(REG_STATUS_CHIRHO);
            if status_chirho & STATUS_DONE_CHIRHO != 0 {
                break;
            }
            timeout_chirho -= 1;
            std::hint::spin_loop();
        }

        if timeout_chirho == 0 {
            return Err("HBM batch timeout".to_string());
        }

        // Step 5: Read results
        self.read_hbm_chirho(self.hbm_config_chirho.output_offset_chirho, count_chirho)
    }

    /// Check if HBM batch should be used for this batch size
    fn should_use_hbm_chirho(&self, batch_size_chirho: usize) -> bool {
        self.hbm_available_chirho
            && batch_size_chirho >= self.hbm_config_chirho.min_batch_size_chirho
    }

    /// Get HBM configuration
    pub fn hbm_config_chirho(&self) -> &HbmConfigChirho {
        &self.hbm_config_chirho
    }

    /// Check if HBM is available
    pub fn hbm_available_chirho(&self) -> bool {
        self.hbm_available_chirho
    }
}

// HBM count register (add to generated constants if not present)
#[allow(dead_code)]
const REG_HBM_COUNT_CHIRHO: u64 = 0x30; // Batch count for HBM operations

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
        // Mock mode: compute on CPU
        if self.mock_mode_chirho {
            return pairs_chirho.iter().map(|(a, b)| a & b).collect();
        }

        // Check if HBM batch path should be used
        #[cfg(target_os = "linux")]
        if self.should_use_hbm_chirho(pairs_chirho.len()) {
            // Try HBM batch path (460 GB/s bandwidth)
            match self.execute_hbm_batch_chirho(pairs_chirho) {
                Ok(results_chirho) => return results_chirho,
                Err(e_chirho) => {
                    log::warn!("HBM batch failed, falling back to register path: {}", e_chirho);
                    // Fall through to register path
                }
            }
        }

        // Register path fallback (for small batches or HBM failure)
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

    // ========================================================================
    // BitVec256 Operations (FPGA-accelerated) ☧
    // ========================================================================

    fn intersect_256_chirho(&self, a_chirho: &BitVec256Chirho, b_chirho: &BitVec256Chirho) -> BitVec256Chirho {
        if self.mock_mode_chirho {
            return a_chirho.and_chirho(*b_chirho);
        }
        // Execute 4 64-bit ANDs via FPGA (could be batched for HBM)
        BitVec256Chirho([
            self.intersect_64_chirho(a_chirho.0[0], b_chirho.0[0]),
            self.intersect_64_chirho(a_chirho.0[1], b_chirho.0[1]),
            self.intersect_64_chirho(a_chirho.0[2], b_chirho.0[2]),
            self.intersect_64_chirho(a_chirho.0[3], b_chirho.0[3]),
        ])
    }

    fn intersect_256_batch_chirho(&self, pairs_chirho: &[(BitVec256Chirho, BitVec256Chirho)]) -> Vec<BitVec256Chirho> {
        if pairs_chirho.is_empty() {
            return Vec::new();
        }

        // Convert to 64-bit pairs (4 per 256-bit)
        let mut pairs_64_chirho = Vec::with_capacity(pairs_chirho.len() * 4);
        for (a_chirho, b_chirho) in pairs_chirho {
            for i_chirho in 0..4 {
                pairs_64_chirho.push((a_chirho.0[i_chirho], b_chirho.0[i_chirho]));
            }
        }

        // Execute batch on FPGA
        let results_64_chirho = self.intersect_batch_chirho(&pairs_64_chirho);

        // Reassemble into 256-bit results
        results_64_chirho
            .chunks(4)
            .map(|chunk_chirho| {
                BitVec256Chirho([
                    chunk_chirho.get(0).copied().unwrap_or(0),
                    chunk_chirho.get(1).copied().unwrap_or(0),
                    chunk_chirho.get(2).copied().unwrap_or(0),
                    chunk_chirho.get(3).copied().unwrap_or(0),
                ])
            })
            .collect()
    }

    // ========================================================================
    // Neurosymbolic Training (on-chip Gumbel-softmax) ☧
    // ========================================================================

    fn neurosym_available_chirho(&self) -> bool {
        // Check if training engine responds
        if self.mock_mode_chirho {
            return true;
        }
        // Read TRAIN_MODE register - non-zero means engine exists
        let train_mode_chirho = self.read_reg_chirho(REG_TRAIN_MODE_CHIRHO);
        train_mode_chirho != 0xFFFF_FFFF // Not unmapped
    }

    fn train_neurosym_chirho(&self, config_chirho: &TrainConfigChirho) -> Result<TrainResultChirho, String> {
        if self.mock_mode_chirho {
            // Simulate training
            return Ok(TrainResultChirho {
                final_loss_q16_chirho: 0x0000_1000, // ~0.0625 loss
                final_epoch_chirho: config_chirho.num_epochs_chirho,
                converged_chirho: true,
            });
        }

        // Write training configuration to registers
        self.write_reg_chirho(REG_TRAIN_CMD_LO_CHIRHO, config_chirho.learning_rate_q16_chirho);
        self.write_reg_chirho(REG_TRAIN_CMD_MID_CHIRHO, config_chirho.temperature_q16_chirho);
        self.write_reg_chirho(
            REG_TRAIN_CMD_HI_CHIRHO,
            ((config_chirho.num_samples_chirho as u32) << 16) | (config_chirho.num_epochs_chirho as u32),
        );
        self.write_reg_chirho(REG_TRAIN_CMD_TOP_CHIRHO, config_chirho.clause_count_chirho);

        // Start training (bit 0 = enable)
        self.write_reg_chirho(REG_TRAIN_MODE_CHIRHO, 0x01);

        // Poll for completion (bit 17 = done in TRAIN_RESP_HI)
        let mut timeout_chirho = 10_000_000; // Long timeout for training
        loop {
            let resp_hi_chirho = self.read_reg_chirho(REG_TRAIN_RESP_HI_CHIRHO);
            if resp_hi_chirho & 0x0002_0000 != 0 {
                // Done bit set
                let resp_lo_chirho = self.read_reg_chirho(REG_TRAIN_RESP_LO_CHIRHO);
                let final_epoch_chirho = ((resp_hi_chirho >> 0) & 0xFFFF) as u16;
                let converged_chirho = resp_lo_chirho < 0x0000_8000; // Loss < 0.5

                return Ok(TrainResultChirho {
                    final_loss_q16_chirho: resp_lo_chirho,
                    final_epoch_chirho,
                    converged_chirho,
                });
            }
            timeout_chirho -= 1;
            if timeout_chirho == 0 {
                return Err("Training timeout".to_string());
            }
            std::hint::spin_loop();
        }
    }

    fn soft_and_q16_chirho(&self, a_q16_chirho: u32, b_q16_chirho: u32) -> u32 {
        if self.mock_mode_chirho {
            return ((a_q16_chirho as u64 * b_q16_chirho as u64) >> 16) as u32;
        }
        // Use FPGA probabilistic engine
        // Write to INFER_MODE to enable probabilistic, then use domain registers
        self.write_reg_chirho(REG_INFER_MODE_CHIRHO, 0x01); // Probabilistic mode
        // Execute via command path (op = soft_and)
        let result_chirho = self.execute_cmd_chirho(OP_SOFT_AND_CHIRHO, a_q16_chirho as u64, b_q16_chirho as u64);
        self.write_reg_chirho(REG_INFER_MODE_CHIRHO, 0x00); // Back to Boolean
        result_chirho as u32
    }
}

// Soft AND operation code (not in generated file)
const OP_SOFT_AND_CHIRHO: u8 = 0x10;

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
