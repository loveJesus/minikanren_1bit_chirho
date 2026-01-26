//! Build script for miniKanren 1-bit ☧
//!
//! Handles Verilator compilation when the `verilator_chirho` feature is enabled.
//!
//! "Except the Lord build the house, they labour in vain that build it" — Psalm 127:1

fn main() {
    // Only run Verilator build if the feature is enabled
    #[cfg(feature = "verilator_chirho")]
    {
        build_verilator_chirho();
    }

    // Always rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

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
