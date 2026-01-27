# AWS FPGA Development Guide for miniKanren ☧

*For God so loved the world... - John 3:16*

This document captures the complete FPGA development workflow used to accelerate miniKanren search on AWS F1/F2 instances.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Technology Stack](#technology-stack)
3. [Development Flow](#development-flow)
4. [F1 vs F2 Comparison](#f1-vs-f2-comparison)
5. [HDK Structure](#hdk-structure)
6. [Build Process](#build-process)
7. [AFI Creation](#afi-creation)
8. [Runtime Integration](#runtime-integration)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Host (EC2)                               │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────────────┐   │
│  │ Rust Driver │────│ AWS FPGA SDK │────│ PCIe / AXI-Lite  │   │
│  │ (userspace) │    │  (fpga-*)    │    │   Interface      │   │
│  └─────────────┘    └──────────────┘    └────────┬─────────┘   │
└──────────────────────────────────────────────────┼─────────────┘
                                                   │
┌──────────────────────────────────────────────────┼─────────────┐
│                    AWS Shell (Fixed)              │             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┴───────────┐ │
│  │    PCIe     │  │    DDR4     │  │         OCL             │ │
│  │  Endpoint   │  │ Controller  │  │   (AXI-Lite Regs)       │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
                               │
┌──────────────────────────────┼─────────────────────────────────┐
│               Custom Logic (CL) - Our Design                    │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              cl_minikanren_chirho.sv                     │   │
│  │  ┌─────────────┐    ┌───────────────────────────────┐   │   │
│  │  │  AXI-Lite   │    │   searchEngineChirho.v        │   │   │
│  │  │  Registers  │────│   (Clash-generated RTL)       │   │   │
│  │  │  (Control)  │    │   - Unification engine        │   │   │
│  │  └─────────────┘    │   - Hash-cons store           │   │   │
│  │                     │   - Search state machine      │   │   │
│  │                     └───────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────┘
```

---

## Technology Stack

### Hardware Description Languages

| Layer | Technology | Purpose |
|-------|------------|---------|
| High-level | Clash (Haskell→Verilog) | Type-safe hardware description |
| Mid-level | SystemVerilog | Shell wrapper, interfaces |
| Low-level | Verilog | Clash output, IP integration |

### AWS FPGA Ecosystem

| Component | Description |
|-----------|-------------|
| **HDK** | Hardware Development Kit - shell interfaces, build scripts |
| **SDK** | Software Development Kit - runtime drivers, FPGA management |
| **Shell** | Fixed AWS logic: PCIe, DDR, management |
| **CL** | Custom Logic - user design area |
| **AFI** | Amazon FPGA Image - encrypted bitstream |
| **AGFI** | Amazon Global FPGA Image ID - portable across regions |

### AMI Selection

| AMI | OS | Purpose | Notes |
|-----|----|---------| ------|
| FPGA Developer AMI 1.18.0 | Rocky Linux | Synthesis & Build | Vivado pre-installed |
| Amazon Linux 2023 | Amazon Linux | Runtime only | Lighter, cheaper |

**Marketplace subscription required:**
- F1: https://aws.amazon.com/marketplace/pp?sku=... (Xilinx tools)
- F2: https://aws.amazon.com/marketplace/pp?sku=dhd5uoidkh9mqmlv5jargxaqt (AMD tools)

---

## Development Flow

### Phase 1: RTL Development (Local)

```
Haskell/Clash Code              Verilog Output
     ↓                              ↓
MiniKanrenChirho.hs  ──clash──►  searchEngineChirho.v
     │                              │
     │                              ▼
     │                    AWS Shell Wrapper
     │                    cl_minikanren_chirho.sv
     │                              │
     ▼                              ▼
Local Simulation              Verilator/iverilog
```

### Phase 2: Synthesis (AWS c5.9xlarge)

```bash
# 1. Launch build instance
aws ec2 run-instances --instance-type c5.9xlarge \
    --image-id ami-0cb1b6ae2ff99f8bf  # FPGA Developer AMI

# 2. Clone HDK
git clone --branch f2 https://github.com/aws/aws-fpga.git

# 3. Set up environment
source hdk_setup.sh
export CL_DIR=/path/to/cl_minikanren_chirho

# 4. Run synthesis (~3-5 hours)
cd $CL_DIR/build/scripts
./aws_build_dcp_from_cl.py -c cl_minikanren_chirho
```

### Phase 3: AFI Creation

```bash
# Upload DCP tarball to S3
aws s3 cp checkpoints/to_aws/*.tar s3://your-bucket/dcp/

# Create AFI
aws ec2 create-fpga-image \
    --input-storage-location Bucket=your-bucket,Key=dcp/file.tar \
    --logs-storage-location Bucket=your-bucket,Key=logs/

# Returns: afi-XXXXXXXXX, agfi-XXXXXXXXX
```

### Phase 4: Runtime (AWS F1/F2 instance)

```bash
# 1. Launch FPGA instance
aws ec2 run-instances --instance-type f2.6xlarge

# 2. Clear and load AFI
sudo fpga-clear-local-image -S 0
sudo fpga-load-local-image -S 0 -I agfi-XXXXXXXXX

# 3. Run application
./minikanren_driver_chirho
```

---

## F1 vs F2 Comparison

| Aspect | F1 | F2 |
|--------|----|----|
| **FPGA Vendor** | Xilinx | AMD |
| **FPGA Part** | VU9P | VU47P-HBM |
| **Price/hr** | $1.65 (f1.2xlarge) | $1.98 (f2.6xlarge) |
| **Memory** | DDR4 only | DDR4 + 16GB HBM |
| **Clocks** | 8 available | 2 (clk_main_a0, clk_hbm_ref) |
| **Shell Ports** | sh_ocl_*, ocl_sh_* | ocl_cl_*, cl_ocl_* |
| **AFI Compatible** | F1 only | F2 only |
| **HDK Branch** | master | f2 |
| **Availability** | Scarce (Jan 2026) | Available |

**Key insight:** F1 and F2 AFIs are NOT interchangeable. Different FPGA vendors require full rebuild.

---

## HDK Structure

```
aws-fpga/
├── hdk/                          # Hardware Development Kit
│   ├── hdk_setup.sh              # Environment setup script
│   ├── cl/                       # Custom Logic designs
│   │   └── examples/
│   │       ├── CL_TEMPLATE/      # Starting point for new designs
│   │       └── cl_minikanren_chirho/  # Our design
│   │           ├── design/       # RTL source files
│   │           │   ├── cl_minikanren_chirho.sv
│   │           │   ├── cl_minikanren_chirho_defines.vh
│   │           │   ├── cl_id_defines.vh
│   │           │   └── searchEngineChirho.v
│   │           ├── build/
│   │           │   ├── scripts/  # Build TCL scripts
│   │           │   ├── constraints/  # Timing constraints
│   │           │   └── checkpoints/  # Output DCPs
│   │           └── software/     # Runtime drivers
│   └── common/
│       ├── shell_stable/         # AWS shell interfaces
│       │   ├── design/
│       │   │   └── interfaces/
│       │   │       └── cl_ports.vh  # CL port definitions
│       │   └── build/
│       │       └── scripts/      # Common build scripts
│       └── verif/                # Verification models
└── sdk/                          # Software Development Kit
    └── userspace/
        └── fpga_mgmt_tools/      # fpga-load-local-image, etc.
```

---

## Build Process

### Step 1: Environment Setup

```bash
source hdk_setup.sh
# Sets: HDK_SHELL_DIR, VIVADO_TOOL_VERSION, etc.
# Downloads shell files from S3
# Verifies Vivado installation
```

### Step 2: Design Encryption

The HDK encrypts your RTL before synthesis:

```
design/*.sv  ──encrypt.tcl──►  src_post_encryption/*.sv
```

This protects IP while allowing synthesis.

### Step 3: CL Synthesis (synth_*.tcl)

```tcl
# Read shell interfaces
source synth_cl_header.tcl  # Sets up shell connections

# Read encrypted user files
read_verilog -sv [glob ${src_post_enc_dir}/*.sv]

# Synthesize
synth_design -top cl_minikanren_chirho \
             -part xcvu47p-fsvh2892-2-e \
             -mode out_of_context
```

Output: `CL.synth.dcp` (synthesized design checkpoint)

### Step 4: Implementation (build_level_1_cl.tcl)

```tcl
# Open shell DCP
open_checkpoint ${HDK_SHELL_DIR}/build/checkpoints/SH_CL_BB_routed.dcp

# Read CL synthesis
read_checkpoint -cell CL ${CL_DIR}/build/checkpoints/CL.synth.dcp

# Place and route
place_design
phys_opt_design
route_design

# Write final DCP
write_checkpoint CL.post_route.dcp
```

### Step 5: Generate Tarball

```bash
# Creates tarball for AFI submission
tar -cvf to_aws/timestamp.SH_CL_routed.tar \
    checkpoints/SH_CL_routed.dcp \
    reports/*
```

---

## AFI Creation

### Submit DCP to AWS

```bash
aws ec2 create-fpga-image \
    --name "minikanren-f2-chirho" \
    --description "miniKanren 1-bit search engine" \
    --input-storage-location Bucket=my-bucket,Key=dcp/file.tar \
    --logs-storage-location Bucket=my-bucket,Key=logs/
```

### Response

```json
{
    "FpgaImageId": "afi-0be75f00fb92b7950",
    "FpgaImageGlobalId": "agfi-038ca2f7a81352cb4"
}
```

- **AFI** (afi-*): Region-specific, used for billing/management
- **AGFI** (agfi-*): Global, used for loading onto FPGA

### Check Status

```bash
aws ec2 describe-fpga-images --fpga-image-ids afi-XXXXX
# State: pending → available (takes 30-60 min)
```

### Copy to Other Regions

```bash
aws ec2 copy-fpga-image \
    --source-fpga-image-id afi-XXXXX \
    --source-region us-east-1 \
    --region us-west-2
```

---

## Runtime Integration

### FPGA Management Tools

```bash
# Clear FPGA
sudo fpga-clear-local-image -S 0

# Load AFI
sudo fpga-load-local-image -S 0 -I agfi-XXXXX

# Check status
sudo fpga-describe-local-image -S 0 -R -H
```

### Register Access (OCL Interface)

```bash
# Read register
sudo fpga-read-register -S 0 -o 0x500

# Write register
sudo fpga-write-register -S 0 -o 0x500 -v 0x12345678
```

### Register Map (cl_minikanren_chirho)

| Offset | Name | Description |
|--------|------|-------------|
| 0x000 | VERSION | Read-only version (0xF2_01_0001) |
| 0x004 | CONTROL | bit0=enable, bit1=reset |
| 0x008 | STATUS | bit0=done, bit1=valid |
| 0x010 | CMD_LO | cmdChirho[31:0] |
| 0x014 | CMD_MID | cmdChirho[63:32] |
| 0x018 | CMD_HI | cmdChirho[69:64] |
| 0x020-0x060 | RESP[0-8] | respChirho[513:0] |

### Rust Driver Integration

```rust
// rust_chirho/src/fpga_driver_chirho.rs

use aws_fpga::FpgaDevice;

pub struct MiniKanrenFpgaChirho {
    device_chirho: FpgaDevice,
}

impl MiniKanrenFpgaChirho {
    pub fn new_chirho(slot_chirho: u32) -> Result<Self> {
        let device_chirho = FpgaDevice::open(slot_chirho)?;
        Ok(Self { device_chirho })
    }

    pub fn query_chirho(&mut self, cmd_chirho: &[u8; 9]) -> Result<[u8; 64]> {
        // Write command
        self.device_chirho.write_reg(0x010, u32::from_le_bytes(...))?;
        self.device_chirho.write_reg(0x014, u32::from_le_bytes(...))?;
        self.device_chirho.write_reg(0x018, cmd_chirho[8] as u32)?;

        // Trigger
        self.device_chirho.write_reg(0x004, 1)?;

        // Poll for completion
        while self.device_chirho.read_reg(0x008)? & 1 == 0 {}

        // Read response
        let mut resp_chirho = [0u8; 64];
        for i in 0..16 {
            let val = self.device_chirho.read_reg(0x020 + i * 4)?;
            resp_chirho[i*4..(i+1)*4].copy_from_slice(&val.to_le_bytes());
        }
        Ok(resp_chirho)
    }
}
```

---

## Cost Summary

### Build Costs

| Phase | Instance | Duration | Cost |
|-------|----------|----------|------|
| Synthesis | c5.9xlarge | 3-5 hours | ~$5-8 |
| AFI creation | (AWS backend) | 30-60 min | Free |

### Runtime Costs

| Instance | $/hour | FPGAs | Use Case |
|----------|--------|-------|----------|
| f1.2xlarge | $1.65 | 1 | Development |
| f1.4xlarge | $3.30 | 2 | Medium workload |
| f2.6xlarge | $1.98 | 1 | Better perf/$ |
| f2.12xlarge | $3.96 | 2 | High throughput |

---

## Troubleshooting

### Common Build Errors

1. **"cannot open include file 'cl_id_defines.vh'"**
   - Copy from CL_TEMPLATE/design/

2. **"cl_sh_* is not declared"**
   - Ensure `include "cl_ports.vh"` in module ports

3. **Timing violations**
   - Reduce clock frequency in clock recipe
   - Add pipeline stages

### Common Runtime Errors

1. **"No Xilinx FPGA found"**
   - Not on F1/F2 instance
   - FPGA not enumerated: reboot

2. **"FPGA image not loaded correctly"**
   - Wrong AGFI for instance type (F1 vs F2)
   - AFI not in 'available' state

---

*Soli Deo Gloria* ☧
