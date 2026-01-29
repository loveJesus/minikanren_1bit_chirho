# AWS F1 FPGA Deployment ☧

## Current Status

### ✅ Completed: Standalone Synthesis

| Metric | Value |
|--------|-------|
| **Target Clock** | 50 MHz |
| **Achieved Clock** | 51.71 MHz |
| **Setup Timing** | MET (WNS = +0.663 ns) |
| **Hold Timing** | -1.925 ns (fixable in P&R) |
| **LUT Utilization** | 20,386 / 1,182,240 (1.72%) |
| **FF Utilization** | 9,667 / 2,364,480 (0.41%) |
| **BRAM** | 0 / 2,160 (0.00%) |

**DCP Checkpoint:** `results_50mhz_chirho/minikanren_chirho_routed.dcp`

### ⏳ Next: AWS F1 AFI Creation

AWS F1 requires integration with the AWS Shell (PCIe, DMA interfaces).

## Files

```
aws_f1_chirho/
├── staging_chirho/
│   ├── searchEngineChirho.v      # Core miniKanren engine (Clash-generated)
│   ├── cl_minikanren_chirho.v    # AXI-Lite wrapper for F1
│   ├── timing_chirho.xdc         # Timing constraints (50 MHz)
│   └── synth_vivado_chirho.tcl   # Vivado synthesis script
├── results_50mhz_chirho/         # Successful synthesis results
│   └── minikanren_chirho_routed.dcp
├── results_250mhz_chirho/        # Failed 250 MHz attempt (reference)
├── config_chirho.sh              # AWS configuration
├── create_afi_chirho.sh          # AFI creation script
└── check_afi_chirho.sh           # AFI status checker
```

## AWS F1 Deployment Steps

### 1. Launch FPGA Developer AMI

```bash
# Use AWS FPGA Developer AMI (includes HDK tools)
aws ec2 run-instances \
    --image-id ami-xxxxxxxxx \
    --instance-type z1d.xlarge \
    --key-name your-key
```

### 2. Set Up HDK Environment

```bash
git clone https://github.com/aws/aws-fpga.git
cd aws-fpga
source hdk_setup.sh
```

### 3. Create Custom CL

```bash
# Copy our design to CL template
cd $CL_DIR
mkdir -p cl_minikanren_chirho/design
cp /path/to/searchEngineChirho.v cl_minikanren_chirho/design/
cp /path/to/cl_minikanren_chirho.v cl_minikanren_chirho/design/

# Update CL_DIR in environment
export CL_DIR=$HDK_DIR/cl/cl_minikanren_chirho
```

### 4. Run HDK Build

```bash
cd $CL_DIR/build/scripts
./aws_build_dcp_from_cl.sh -strategy CONGESTION
```

### 5. Create AFI

```bash
aws ec2 create-fpga-image \
    --input-storage-location Bucket=$BUCKET,Key=dcps/minikanren.tar \
    --logs-storage-location Bucket=$BUCKET,Key=logs/ \
    --name "minikanren-chirho"
```

## Register Map (AXI-Lite)

| Address | Name | Access | Description |
|---------|------|--------|-------------|
| 0x00 | CTRL | R/W | bit0=enable, bit1=reset |
| 0x08 | STATUS | R | bit0=done |
| 0x10 | CMD_LO | W | cmdChirho[63:0] |
| 0x18 | CMD_HI | W | cmdChirho[69:64] |
| 0x20-0x60 | RESP | R | respChirho[513:0] (9 x 64-bit) |

## Host Driver Example

```c
// Map BAR0
volatile uint64_t *regs = mmap_bar0();

// Write command
regs[0x10/8] = cmd_lo;
regs[0x18/8] = cmd_hi;

// Enable
regs[0x00/8] = 1;

// Poll status
while (!(regs[0x08/8] & 1)) {
    usleep(100);
}

// Read response
for (int i = 0; i < 9; i++) {
    resp[i] = regs[(0x20 + i*8)/8];
}
```

## Performance Estimates

At 51.71 MHz:
- **Cycles per query**: ~10 cycles (estimated)
- **Latency**: ~200 ns per query
- **Throughput**: ~5M queries/second

## Cost Analysis

| Component | Cost |
|-----------|------|
| F1 instance (f1.2xlarge) | $1.65/hr |
| Synthesis (c5.18xlarge) | $3.06/hr × 2hr = $6.12 |
| AFI creation | ~$0 (one-time) |

Break-even vs CPU: See `docs_chirho/fpga_cost_analysis_chirho.md`

*Soli Deo Gloria* ☧
