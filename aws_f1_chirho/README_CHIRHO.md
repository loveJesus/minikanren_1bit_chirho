# AWS F1 FPGA Deployment ☧

Scripts for deploying the miniKanren search engine to AWS F1 FPGAs.

## Prerequisites

1. AWS CLI configured with credentials
2. SSH key for EC2 access
3. Sufficient IAM permissions for EC2, S3, and FPGA operations

## Workflow

### 1. Initial Setup
```bash
./setup_chirho.sh
```
Creates S3 bucket, SSH key, and security group.

### 2. Upload Design
```bash
./upload_design_chirho.sh
```
Uploads Verilog and constraints to S3.

### 3. Run Synthesis (~2-4 hours)
```bash
./launch_synth_chirho.sh
```
Launches c5.4xlarge (~$0.68/hr) with Vivado 2024.2.

Check progress:
```bash
aws s3 ls s3://YOUR_BUCKET/results/
```

### 4. Download Results
```bash
./download_results_chirho.sh
```
Downloads timing reports, utilization, and DCP checkpoint.

### 5. Create AFI (~1-2 hours)
```bash
./create_afi_chirho.sh
```
Creates Amazon FPGA Image from DCP.

Check AFI status:
```bash
./check_afi_chirho.sh
```

### 6. Run on F1 FPGA
```bash
./run_f1_chirho.sh
```
Launches f1.2xlarge (~$1.65/hr) and loads the AFI.

## Cost Estimate

| Step | Instance | Cost/Hour | Typical Duration |
|------|----------|-----------|------------------|
| Synthesis | c5.4xlarge | $0.68 | 2-4 hours |
| AFI Creation | N/A | Free | 1-2 hours |
| F1 Run | f1.2xlarge | $1.65 | 0.5-1 hour |

**Total estimated cost:** $3-10 per full run

## Files

| Script | Purpose |
|--------|---------|
| `setup_chirho.sh` | Create AWS infrastructure |
| `upload_design_chirho.sh` | Upload Verilog to S3 |
| `launch_synth_chirho.sh` | Launch Vivado synthesis |
| `download_results_chirho.sh` | Download synthesis artifacts |
| `create_afi_chirho.sh` | Create Amazon FPGA Image |
| `check_afi_chirho.sh` | Check AFI creation status |
| `run_f1_chirho.sh` | Run on F1 FPGA |
| `cleanup_chirho.sh` | Terminate instances and cleanup |

## Artifacts Produced

After successful run:
- `results_chirho/timing_chirho.rpt` - Post-route timing
- `results_chirho/utilization_chirho.rpt` - Resource usage
- `results_chirho/minikanren_chirho_routed.dcp` - Vivado checkpoint
- `results_chirho/vivado_log_chirho.txt` - Build log
- `afi_id_chirho.txt` - Amazon FPGA Image ID

---

*Soli Deo Gloria* ☧
