#!/bin/bash
# Launch Vivado Synthesis Instance ☧
#
# Spins up a c5.4xlarge with FPGA Developer AMI and runs synthesis.
# Part of P5-01: Physical FPGA Incarnation.
#
# COST WARNING: c5.4xlarge is ~$0.68/hour

set -euo pipefail

SCRIPT_DIR_CHIRHO="$(cd "$(dirname "$0")" && pwd)"

# Load configuration
if [ -f "${SCRIPT_DIR_CHIRHO}/config_chirho.sh" ]; then
    source "${SCRIPT_DIR_CHIRHO}/config_chirho.sh"
else
    echo "ERROR: Run setup_chirho.sh first"
    exit 1
fi

echo "=== Launch Vivado Synthesis ☧ ==="
echo "Instance: ${SYNTH_INSTANCE_CHIRHO}"
echo "AMI: ${FPGA_DEV_AMI_CHIRHO}"
echo ""
echo "COST WARNING: This will launch a ${SYNTH_INSTANCE_CHIRHO} instance (~\$0.68/hour)"
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Create user data script
USER_DATA_CHIRHO=$(cat << 'USERDATA'
#!/bin/bash
set -x

# Log everything
exec > >(tee /var/log/user-data.log) 2>&1

echo "=== miniKanren Synthesis Starting ☧ ==="
date

# Get instance metadata
INSTANCE_ID=$(curl -s http://169.254.169.254/latest/meta-data/instance-id)
REGION=$(curl -s http://169.254.169.254/latest/meta-data/placement/region)

# Set up AWS SDK
export AWS_DEFAULT_REGION=${REGION}

# Get bucket name from tag
BUCKET=$(aws ec2 describe-tags --filters "Name=resource-id,Values=${INSTANCE_ID}" "Name=key,Values=S3Bucket" --query "Tags[0].Value" --output text)

echo "Bucket: ${BUCKET}"

# Create work directory
WORK_DIR="/home/centos/minikanren_chirho"
mkdir -p ${WORK_DIR}
cd ${WORK_DIR}

# Download design files
echo "Downloading design files..."
aws s3 sync "s3://${BUCKET}/design/" .

# Source Vivado
source /opt/Xilinx/Vivado/2024.2/settings64.sh || source /opt/Xilinx/Vivado/*/settings64.sh

# Run synthesis
echo "Running Vivado synthesis..."
vivado -mode batch -source synth_vivado_chirho.tcl 2>&1 | tee vivado_log_chirho.txt

# Upload results
echo "Uploading results..."
aws s3 cp utilization_chirho.rpt "s3://${BUCKET}/results/"
aws s3 cp utilization_hierarchical_chirho.rpt "s3://${BUCKET}/results/" || true
aws s3 cp timing_chirho.rpt "s3://${BUCKET}/results/"
aws s3 cp timing_paths_chirho.rpt "s3://${BUCKET}/results/" || true
aws s3 cp clock_utilization_chirho.rpt "s3://${BUCKET}/results/" || true
aws s3 cp vivado_log_chirho.txt "s3://${BUCKET}/results/"
aws s3 cp minikanren_chirho/minikanren_chirho_routed.dcp "s3://${BUCKET}/results/" || true

echo "=== Synthesis Complete ☧ ==="
date

# Signal completion
aws s3 cp /var/log/user-data.log "s3://${BUCKET}/results/user-data.log"

# Optional: self-terminate after completion (uncomment to enable)
# aws ec2 terminate-instances --instance-ids ${INSTANCE_ID}
USERDATA
)

# Launch instance
echo "Launching instance..."
INSTANCE_ID_CHIRHO=$(aws ec2 run-instances \
    --image-id "${FPGA_DEV_AMI_CHIRHO}" \
    --instance-type "${SYNTH_INSTANCE_CHIRHO}" \
    --key-name "${KEY_NAME_CHIRHO}" \
    --security-group-ids "${SECURITY_GROUP_CHIRHO}" \
    --subnet-id "${SUBNET_ID_CHIRHO}" \
    --associate-public-ip-address \
    --region "${AWS_REGION_CHIRHO}" \
    --iam-instance-profile Name=minikanren-fpga-role-chirho \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=minikanren-synth-chirho},{Key=S3Bucket,Value=${S3_BUCKET_CHIRHO}}]" \
    --user-data "${USER_DATA_CHIRHO}" \
    --query 'Instances[0].InstanceId' \
    --output text 2>/dev/null || \
    # Fallback without IAM role
    aws ec2 run-instances \
        --image-id "${FPGA_DEV_AMI_CHIRHO}" \
        --instance-type "${SYNTH_INSTANCE_CHIRHO}" \
        --key-name "${KEY_NAME_CHIRHO}" \
        --security-group-ids "${SECURITY_GROUP_CHIRHO}" \
        --subnet-id "${SUBNET_ID_CHIRHO}" \
        --associate-public-ip-address \
        --region "${AWS_REGION_CHIRHO}" \
        --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=minikanren-synth-chirho},{Key=S3Bucket,Value=${S3_BUCKET_CHIRHO}}]" \
        --query 'Instances[0].InstanceId' \
        --output text)

echo "Instance launched: ${INSTANCE_ID_CHIRHO}"

# Save instance ID
echo "${INSTANCE_ID_CHIRHO}" > "${SCRIPT_DIR_CHIRHO}/synth_instance_id_chirho.txt"

# Wait for instance to be running
echo "Waiting for instance to start..."
aws ec2 wait instance-running --instance-ids "${INSTANCE_ID_CHIRHO}" --region "${AWS_REGION_CHIRHO}"

# Get public IP
PUBLIC_IP_CHIRHO=$(aws ec2 describe-instances \
    --instance-ids "${INSTANCE_ID_CHIRHO}" \
    --region "${AWS_REGION_CHIRHO}" \
    --query 'Reservations[0].Instances[0].PublicIpAddress' \
    --output text)

echo ""
echo "=== Instance Ready ☧ ==="
echo "Instance ID: ${INSTANCE_ID_CHIRHO}"
echo "Public IP:   ${PUBLIC_IP_CHIRHO}"
echo ""
echo "To connect:"
echo "  ssh -i ~/.ssh/${KEY_NAME_CHIRHO}.pem centos@${PUBLIC_IP_CHIRHO}"
echo ""
echo "To check synthesis progress:"
echo "  aws s3 ls s3://${S3_BUCKET_CHIRHO}/results/"
echo ""
echo "To download results when done:"
echo "  ./download_results_chirho.sh"
echo ""
echo "REMEMBER to terminate when done:"
echo "  aws ec2 terminate-instances --instance-ids ${INSTANCE_ID_CHIRHO}"
