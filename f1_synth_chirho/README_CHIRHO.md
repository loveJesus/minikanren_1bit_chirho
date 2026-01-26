# AWS F1 FPGA Synthesis Package ☧

## Contents

| File | Description |
|------|-------------|
| `searchEngineChirho.v` | Clash-generated Verilog (full search engine) |
| `synth_f1_chirho.sh` | Full F1 flow (creates AFI) |
| `synth_standalone_chirho.tcl` | Quick Vivado synthesis (timing/utilization) |

## Quick Start (Standalone Timing)

On FPGA Developer AMI:

```bash
# Clone and setup
git clone https://github.com/aws/aws-fpga
cd aws-fpga && source vivado_setup.sh

# Run synthesis
cd /path/to/f1_synth_chirho
vivado -mode batch -source synth_standalone_chirho.tcl

# Check results
cat utilization_chirho.rpt
cat timing_chirho.rpt
```

## Full F1 Flow

```bash
# Setup HDK
cd ~/aws-fpga && source hdk_setup.sh

# Run synthesis (2-4 hours)
./synth_f1_chirho.sh

# Create AFI (after synthesis completes)
aws ec2 create-fpga-image \
  --name minikanren_chirho \
  --input-storage-location Bucket=your-bucket,Key=path/to/dcp
```

## Expected Results

| Metric | Target | Actual |
|--------|--------|--------|
| LUTs | <100K | TBD |
| FFs | <50K | TBD |
| Fmax | >100 MHz | TBD |
| Latency | <100 ns | TBD |

## Design Metrics from Yosys

Synthesized locally with Yosys (technology-independent):
- 43,136 cells
- 9,781 flip-flops
- Estimated Fmax: 100-200 MHz

---

*Soli Deo Gloria* ☧
