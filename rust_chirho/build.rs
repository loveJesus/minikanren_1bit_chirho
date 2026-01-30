//! Build script for miniKanren 1-bit ☧
//!
//! - Generates FPGA register constants from Verilog defines (fpga_chirho feature)
//! - Handles Verilator compilation (verilator_chirho feature)
//!
//! "Except the Lord build the house, they labour in vain that build it" — Psalm 127:1

fn main() {
    // Generate FPGA register constants from Verilog
    #[cfg(feature = "fpga_chirho")]
    {
        generate_fpga_regs_chirho();
    }

    // Verilator build if enabled
    #[cfg(feature = "verilator_chirho")]
    {
        build_verilator_chirho();
    }

    // Always rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

/// Generate Rust constants from cl_minikanren_chirho_defines.vh
#[cfg(feature = "fpga_chirho")]
fn generate_fpga_regs_chirho() {
    use std::env;
    use std::fs;
    use std::path::Path;

    let out_dir_chirho = env::var("OUT_DIR").unwrap();
    let dest_path_chirho = Path::new(&out_dir_chirho).join("fpga_regs_chirho.rs");

    // Try to find the Verilog defines file
    let vh_paths_chirho = [
        "../synth_chirho/v5_aws_f2_floorplan_chirho_cl/design/cl_minikanren_chirho_defines.vh",
        "../../synth_chirho/v5_aws_f2_floorplan_chirho_cl/design/cl_minikanren_chirho_defines.vh",
        "../synth_chirho/v4_aws_f2_hier_ns_chirho_cl/design/cl_minikanren_chirho_defines.vh",
    ];

    let vh_content_chirho = vh_paths_chirho
        .iter()
        .find_map(|path| {
            if Path::new(path).exists() {
                println!("cargo:rerun-if-changed={}", path);
                fs::read_to_string(path).ok()
            } else {
                None
            }
        });

    let mut output_chirho = String::new();
    output_chirho.push_str("// Auto-generated from cl_minikanren_chirho_defines.vh ☧\n");
    output_chirho.push_str("// Do not edit - run `cargo build` to regenerate from Verilog\n");
    output_chirho.push_str("// Source of truth: synth_chirho/.../cl_minikanren_chirho_defines.vh\n\n");

    if let Some(content_chirho) = vh_content_chirho {
        // Parse `define REG_NAME_CHIRHO 8'hXX lines
        for line_chirho in content_chirho.lines() {
            if let Some(reg_chirho) = parse_verilog_define_chirho(line_chirho) {
                output_chirho.push_str(&reg_chirho);
            }
        }
        println!("cargo:warning=Generated FPGA registers from Verilog defines ☧");
    } else {
        // Fallback: use hardcoded defaults if .vh not found
        println!("cargo:warning=Verilog defines not found, using fallback register addresses");
        output_chirho.push_str("// Fallback values (Verilog file not found at build time)\n\n");
        output_chirho.push_str(FALLBACK_REGS_CHIRHO);
    }

    fs::write(&dest_path_chirho, output_chirho).unwrap();
}

#[cfg(feature = "fpga_chirho")]
fn parse_verilog_define_chirho(line_chirho: &str) -> Option<String> {
    let line_chirho = line_chirho.trim();

    // Match: `define REG_NAME_CHIRHO 8'hXX // comment
    // Also match OP_ and HBM_ defines
    if !line_chirho.starts_with("`define REG_")
        && !line_chirho.starts_with("`define OP_")
        && !line_chirho.starts_with("`define HBM_")
    {
        return None;
    }

    let parts_chirho: Vec<&str> = line_chirho.split_whitespace().collect();
    if parts_chirho.len() < 3 {
        return None;
    }

    let name_chirho = parts_chirho[1];
    let value_str_chirho = parts_chirho[2];

    // Parse Verilog literal: 8'h00, 32'h1234, etc.
    let value_chirho = parse_verilog_literal_chirho(value_str_chirho)?;

    // Extract comment if present
    let comment_chirho = if let Some(idx) = line_chirho.find("//") {
        line_chirho[idx + 2..].trim()
    } else {
        ""
    };

    Some(format!(
        "/// {}\npub const {}: u64 = 0x{:X};\n\n",
        comment_chirho, name_chirho, value_chirho
    ))
}

#[cfg(feature = "fpga_chirho")]
fn parse_verilog_literal_chirho(s_chirho: &str) -> Option<u64> {
    // Handle formats: 8'h00, 32'hDEADBEEF, 34'h0_0000_0000, 4'h0
    if let Some(idx) = s_chirho.find("'h") {
        let hex_part_chirho = &s_chirho[idx + 2..];
        let clean_chirho: String = hex_part_chirho.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        u64::from_str_radix(&clean_chirho, 16).ok()
    } else if let Some(idx) = s_chirho.find("'d") {
        let dec_part_chirho = &s_chirho[idx + 2..];
        dec_part_chirho.parse().ok()
    } else if let Some(idx) = s_chirho.find("'b") {
        let bin_part_chirho = &s_chirho[idx + 2..];
        let clean_chirho: String = bin_part_chirho.chars().filter(|c| *c == '0' || *c == '1').collect();
        u64::from_str_radix(&clean_chirho, 2).ok()
    } else {
        // Plain number
        s_chirho.parse().ok()
    }
}

#[cfg(feature = "fpga_chirho")]
const FALLBACK_REGS_CHIRHO: &str = r#"
/// Read-only version
pub const REG_VERSION_CHIRHO: u64 = 0x00;
/// bit0=enable, bit1=reset, bit2=hbm_mode
pub const REG_CONTROL_CHIRHO: u64 = 0x04;
/// bit0=done, bit1=valid, bit2=hbm_ready
pub const REG_STATUS_CHIRHO: u64 = 0x08;
/// cmdChirho[31:0]
pub const REG_CMD_LO_CHIRHO: u64 = 0x10;
/// cmdChirho[63:32]
pub const REG_CMD_MID_CHIRHO: u64 = 0x14;
/// cmdChirho[69:64]
pub const REG_CMD_HI_CHIRHO: u64 = 0x18;
/// Response registers start
pub const REG_RESP_BASE_CHIRHO: u64 = 0x20;
/// Hierarchical mode selection
pub const REG_HIER_MODE_CHIRHO: u64 = 0x40;
/// Training mode control
pub const REG_TRAIN_MODE_CHIRHO: u64 = 0x50;
/// Inference mode control
pub const REG_INFER_MODE_CHIRHO: u64 = 0x70;
"#;

#[cfg(feature = "verilator_chirho")]
fn build_verilator_chirho() {
    use std::env;
    use std::path::PathBuf;
    use std::process::Command;

    // Paths
    let manifest_dir_chirho = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir_chirho = env::var("OUT_DIR").unwrap();
    let verilog_dir_chirho = PathBuf::from(&manifest_dir_chirho)
        .parent()
        .unwrap()
        .join("clash_chirho/verilog/MiniKanrenChirho.searchEngineChirho");
    let wrapper_dir_chirho = PathBuf::from(&manifest_dir_chirho)
        .join("src/verify_hw_chirho/verilator");

    // Check if Verilator is available
    let verilator_check_chirho = Command::new("verilator")
        .arg("--version")
        .output();

    if verilator_check_chirho.is_err() {
        println!("cargo:warning=Verilator not found. Skipping hardware verification build.");
        println!("cargo:warning=Install Verilator to enable co-simulation: brew install verilator");
        return;
    }

    let version_chirho = verilator_check_chirho.unwrap();
    if !version_chirho.status.success() {
        println!("cargo:warning=Verilator found but failed version check");
        return;
    }

    println!("cargo:warning=Building Verilator model (this may take a moment)...");

    // Check if Verilog exists
    let verilog_file_chirho = verilog_dir_chirho.join("searchEngineChirho.v");
    if !verilog_file_chirho.exists() {
        println!("cargo:warning=Verilog file not found: {:?}", verilog_file_chirho);
        println!("cargo:warning=Run 'cd clash_chirho && cabal run clash -- --verilog MiniKanrenChirho' first");
        return;
    }

    // Run Verilator to generate C++ model
    let verilator_out_chirho = PathBuf::from(&out_dir_chirho).join("verilator");
    std::fs::create_dir_all(&verilator_out_chirho).unwrap();

    let verilator_status_chirho = Command::new("verilator")
        .arg("--cc")                           // Generate C++
        .arg("--exe")                          // Build executable
        .arg("-Mdir").arg(&verilator_out_chirho)  // Output directory
        .arg("-CFLAGS").arg("-fPIC")           // Position independent code
        .arg("-CFLAGS").arg("-DVERILATOR_AVAILABLE")
        .arg("--trace")                        // Enable VCD tracing (optional)
        .arg("-Wno-WIDTHTRUNC")                // Suppress width truncation warnings
        .arg("-Wno-WIDTHEXPAND")               // Suppress width expansion warnings
        .arg(&verilog_file_chirho)             // Input Verilog
        .arg(wrapper_dir_chirho.join("wrapper_chirho.cpp"))  // Our wrapper
        .current_dir(&out_dir_chirho)
        .status();

    match verilator_status_chirho {
        Ok(status) if status.success() => {
            println!("cargo:warning=Verilator codegen succeeded");
        }
        Ok(status) => {
            println!("cargo:warning=Verilator failed with exit code: {:?}", status.code());
            return;
        }
        Err(e) => {
            println!("cargo:warning=Verilator execution failed: {}", e);
            return;
        }
    }

    // Build the Verilator model using make
    let make_status_chirho = Command::new("make")
        .arg("-C").arg(&verilator_out_chirho)
        .arg("-f").arg("VsearchEngineChirho.mk")
        .arg("VsearchEngineChirho__ALL.a")     // Build static library
        .status();

    match make_status_chirho {
        Ok(status) if status.success() => {
            println!("cargo:warning=Verilator make succeeded");
        }
        Ok(status) => {
            println!("cargo:warning=Make failed with exit code: {:?}", status.code());
            return;
        }
        Err(e) => {
            println!("cargo:warning=Make execution failed: {}", e);
            return;
        }
    }

    // Link the Verilator library
    println!("cargo:rustc-link-search=native={}", verilator_out_chirho.display());
    println!("cargo:rustc-link-lib=static=VsearchEngineChirho__ALL");

    // Link Verilator runtime (find VERILATOR_ROOT)
    if let Ok(verilator_root_chirho) = env::var("VERILATOR_ROOT") {
        let verilator_lib_chirho = PathBuf::from(&verilator_root_chirho).join("include");
        println!("cargo:rustc-link-search=native={}", verilator_lib_chirho.display());
    }

    // Link C++ standard library
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=c++");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:rerun-if-changed=src/verify_hw_chirho/verilator/wrapper_chirho.cpp");
    println!("cargo:rerun-if-changed={}", verilog_file_chirho.display());

    println!("cargo:warning=Verilator build complete! Co-simulation ready.");
}
