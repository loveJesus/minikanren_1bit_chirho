# F2 FPGA Test Plan ☧

For God so loved the world - John 3:16

## Prerequisites

1. DCP build completed successfully
2. AFI created and in "available" state
3. F2 instance launched (f2.6xlarge recommended)

## Test Sequence

### Phase 1: AFI Loading

```bash
# Check current FPGA status
fpga-describe-local-image -S 0 -H

# Clear any existing AFI
fpga-clear-local-image -S 0

# Load our AFI (replace with actual AGFI ID)
fpga-load-local-image -S 0 -I agfi-XXXXXXXXXXXXXXXXX

# Verify loaded
fpga-describe-local-image -S 0 -H
# Should show: loaded, vendor-id=0x1d0f, device-id=0xf216
```

### Phase 2: Basic Register Tests

```bash
# Build test software
cd host
source ~/src/project_data/aws-fpga/sdk_setup.sh
make

# Run basic tests
sudo ./test_minikanren_chirho
```

**Expected results:**
- Version register: 0xF216316A
- Status register shows engine ready
- Command/response path working

### Phase 3: HBM Memory Tests

1. Write test patterns to HBM via PCIS
2. Read back and verify
3. Test different addresses across HBM channels

### Phase 4: Search Engine Tests

1. Initialize search state
2. Load simple term database
3. Run unification query
4. Verify results

## Test Cases

| ID | Test | Expected Result | Priority |
|----|------|-----------------|----------|
| T1 | Version register read | 0xF216316A | P0 |
| T2 | Status register read | Engine ready bit set | P0 |
| T3 | Command write | No error | P0 |
| T4 | Response read | Valid response | P0 |
| T5 | HBM write/read | Data matches | P1 |
| T6 | Search NOP | No-op completes | P1 |
| T7 | Search INIT | State initialized | P2 |
| T8 | Search STEP | State advances | P2 |

## Troubleshooting

### AFI won't load
- Check AFI state: `aws ec2 describe-fpga-images --fpga-image-ids afi-xxx`
- Check for errors in AFI creation logs in S3

### Version mismatch
- Verify correct AFI is loaded
- Check DCP was built from correct source

### HBM errors
- Check HBM calibration in status register
- HBM init can take a few seconds after AFI load

## Cost Tracking

| Resource | Rate | Est. Duration | Est. Cost |
|----------|------|---------------|-----------|
| f2.6xlarge | $1.98/hr | 2 hours testing | $4 |
| Data transfer | minimal | - | <$1 |

---

Soli Deo Gloria ☧
