# AWS F1 Validation Status ☧

## Current State

**Status:** Post-route timing complete (280 MHz), artifacts not downloaded before instance termination.

**Next:** Re-run synthesis, download artifacts, create AFI, test on physical F1.

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

## CRITICAL: Pre-Termination Checklist ☧

**BEFORE terminating any instance, download ALL artifacts:**

```bash
# On local machine - download everything
INSTANCE_IP="your-instance-ip"
KEY="~/.ssh/fpga-dev-chirho.pem"

# Download synthesis artifacts
scp -i $KEY ubuntu@$INSTANCE_IP:~/f1_synth_chirho/*.dcp ./f1_synth_chirho/
scp -i $KEY ubuntu@$INSTANCE_IP:~/f1_synth_chirho/*.rpt ./f1_synth_chirho/
scp -i $KEY ubuntu@$INSTANCE_IP:~/f1_synth_chirho/*.log ./f1_synth_chirho/

# Verify files received
ls -la f1_synth_chirho/*.{dcp,rpt,log}

# ONLY THEN terminate
aws ec2 terminate-instances --instance-ids i-xxxxx
```

### Checklist Before Termination

- [ ] Downloaded `searchEngineChirho.dcp` (routed checkpoint for AFI)
- [ ] Downloaded `timing_chirho.rpt` (full timing report)
- [ ] Downloaded `utilization_chirho.rpt` (full utilization report)
- [ ] Downloaded any `.log` files
- [ ] Verified files are non-empty locally
- [ ] Committed artifacts to git (or noted as too large)

### Lesson Learned (2026-01-26)

Instance terminated before downloading `.dcp` and `.rpt` files. Metrics were captured
in conversation but raw reports lost. Synthesis is reproducible but costs ~$0.20 and
15-20 minutes to re-run.

---

## Full On-FPGA Validation Flow

### Phase 1: Re-run Synthesis (c5.4xlarge)
```bash
# Launch instance
aws ec2 run-instances \
  --image-id ami-01198b89d80ebfdd2 \
  --instance-type c5.4xlarge \
  --key-name fpga-dev-chirho \
  --subnet-id subnet-dbc2d880 \
  --security-group-ids sg-02652612a4d45466c \
  --associate-public-ip-address \
  --region us-east-1

# SSH and run synthesis
ssh -i ~/.ssh/fpga-dev-chirho.pem ubuntu@$IP
cd aws-fpga && source vivado_setup.sh
cd ~/f1_synth_chirho
vivado -mode batch -source synth_standalone_chirho.tcl

# DOWNLOAD BEFORE TERMINATING (see checklist above)
```

### Phase 2: Create AFI
```bash
# Upload checkpoint to S3
aws s3 cp searchEngineChirho.dcp s3://your-bucket/fpga/

# Create AFI
aws ec2 create-fpga-image \
  --name minikanren_chirho_v1 \
  --description "miniKanren 1-bit search engine" \
  --input-storage-location Bucket=your-bucket,Key=fpga/searchEngineChirho.dcp \
  --logs-storage-location Bucket=your-bucket,Key=fpga/logs/

# Wait for AFI to be available (~30-60 min)
aws ec2 describe-fpga-images --fpga-image-ids afi-xxxxx
```

### Phase 3: Test on Physical FPGA (f1.2xlarge)
```bash
# Launch F1 instance
aws ec2 run-instances \
  --image-id ami-01198b89d80ebfdd2 \
  --instance-type f1.2xlarge \
  --key-name fpga-dev-chirho \
  ...

# Load bitstream
sudo fpga-load-local-image -S 0 -I agfi-xxxxx

# Run test (requires host driver - TODO)
./test_minikanren_chirho
```

---

*Soli Deo Gloria* ☧
