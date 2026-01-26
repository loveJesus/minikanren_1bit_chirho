# AWS F1 Validation Status ☧

## Current State

**Blocked on:** AWS Marketplace subscription for FPGA Developer AMI

### Action Required

1. Visit: https://aws.amazon.com/marketplace/pp?sku=e4txuxx6uz6371b7tgmotozac
2. Click "Continue to Subscribe"
3. Accept terms (free - you only pay for instance hours)
4. Wait ~1 minute for activation

## Infrastructure Ready

| Component | Status |
|-----------|--------|
| Security Group | ✅ `sg-02652612a4d45466c` (SSH enabled) |
| Subnet | ✅ `subnet-dbc2d880` (us-east-1d) |
| Key Pair | ✅ `c5-aws-aleluya` |
| AMI | ⏳ `ami-01198b89d80ebfdd2` (FPGA Developer 1.17.0) |
| Synthesis Package | ✅ `f1_synth_chirho.tar.gz` (16KB) |

## Prepared Files

```
f1_synth_chirho/
├── searchEngineChirho.v      # Clash-generated Verilog
├── synth_standalone_chirho.tcl # Quick Vivado synthesis
├── synth_f1_chirho.sh        # Full F1 AFI flow
└── README_CHIRHO.md          # Instructions
```

## Once Subscription Active

Run this command to launch:

```bash
aws ec2 run-instances \
  --image-id ami-01198b89d80ebfdd2 \
  --instance-type c5.4xlarge \
  --key-name c5-aws-aleluya \
  --subnet-id subnet-dbc2d880 \
  --security-group-ids sg-02652612a4d45466c \
  --associate-public-ip-address \
  --region us-east-1 \
  --tag-specifications 'ResourceType=instance,Tags=[{Key=Name,Value=minikanren-fpga-dev-chirho}]'
```

Then:
1. SSH into instance
2. Upload `f1_synth_chirho.tar.gz`
3. Run `vivado -mode batch -source synth_standalone_chirho.tcl`
4. Check `timing_chirho.rpt` for real Fmax

## Cost Estimate

- c5.4xlarge: $0.68/hr (development/synthesis)
- f1.2xlarge: $1.65/hr (actual FPGA testing)
- Estimated total: ~$10 for full validation

---

*Soli Deo Gloria* ☧
