#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16
# Launch F2 HBM Synthesis on c5.9xlarge (72GB RAM) ☧
# Uses AWS HDK standard build flow (like previous successful F2 build)
# ============================================================================

set -euo pipefail

SCRIPT_DIR_CHIRHO="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT_CHIRHO="$(dirname "$SCRIPT_DIR_CHIRHO")"

# Configuration
AWS_REGION_CHIRHO="${AWS_REGION:-us-east-1}"
S3_BUCKET_CHIRHO="${S3_BUCKET:-minikanren-fpga-chirho}"
KEY_NAME_CHIRHO="${KEY_NAME:-minikanren-fpga-key-chirho}"

# c5.9xlarge: 36 vCPUs, 72 GB RAM - needed for HBM synthesis
SYNTH_INSTANCE_CHIRHO="c5.9xlarge"

# FPGA Developer AMI 1.18.0 (Rocky Linux) with Vivado/Vitis 2025.1
# This matches the successful F2 build from Jan 26
FPGA_DEV_AMI_CHIRHO="${FPGA_DEV_AMI:-ami-0cb1b6ae2ff99f8bf}"
SSH_USER_CHIRHO="rocky"  # Rocky Linux uses 'rocky' user

echo "============================================================"
echo "  F2 HBM Synthesis Launch ☧"
echo "============================================================"
echo ""
echo "Instance:    ${SYNTH_INSTANCE_CHIRHO} (36 vCPU, 72 GB RAM)"
echo "Region:      ${AWS_REGION_CHIRHO}"
echo "S3 Bucket:   ${S3_BUCKET_CHIRHO}"
echo ""
echo "COST WARNING: c5.9xlarge costs ~\$1.53/hour"
echo "Expected synthesis time: 3-5 hours (~\$5-8 total)"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Step 1: Upload design files to S3
echo ""
echo "=== Step 1: Uploading design files to S3 ==="

DESIGN_DIR_CHIRHO="${SCRIPT_DIR_CHIRHO}/design"
BUILD_DIR_CHIRHO="${SCRIPT_DIR_CHIRHO}/build"

# Create tarball of design
TARBALL_CHIRHO="/tmp/f2_hbm_design_chirho.tar.gz"
tar -czf "${TARBALL_CHIRHO}" -C "${SCRIPT_DIR_CHIRHO}" design build

# Upload to S3
aws s3 cp "${TARBALL_CHIRHO}" "s3://${S3_BUCKET_CHIRHO}/f2_hbm/design.tar.gz" --region "${AWS_REGION_CHIRHO}"
echo "Uploaded design to s3://${S3_BUCKET_CHIRHO}/f2_hbm/design.tar.gz"

# Step 2: Create user data script (uses HDK build flow)
echo ""
echo "=== Step 2: Creating launch configuration ==="

USER_DATA_CHIRHO=$(cat << 'USERDATA_EOF'
#!/bin/bash
# For God so loved the world - John 3:16 ☧
# F2 HBM Synthesis using AWS HDK standard build flow
set -x
exec > >(tee /var/log/f2_synth_chirho.log) 2>&1

echo "=== F2 HBM Synthesis Starting ☧ ==="
date

export HOME=/root
export AWS_DEFAULT_REGION=REGION_PLACEHOLDER
BUCKET_CHIRHO="BUCKET_PLACEHOLDER"

# Create work directory
WORK_DIR_CHIRHO="/root/f2_hbm_chirho"
mkdir -p ${WORK_DIR_CHIRHO}
cd ${WORK_DIR_CHIRHO}

# Download design
echo "Downloading design..."
aws s3 cp "s3://${BUCKET_CHIRHO}/f2_hbm/design.tar.gz" design.tar.gz
tar -xzf design.tar.gz

# Source Vivado FIRST (required by hdk_setup.sh)
echo "Sourcing Vivado 2025.1..."
source /opt/Xilinx/2025.1/Vivado/settings64.sh

# Verify Vivado is available
echo "Checking Vivado..."
which vivado
vivado -version

# Run synthesis using our TCL script directly
echo "Running Vivado synthesis..."
cd ${WORK_DIR_CHIRHO}/build/scripts

# Run synthesis (out-of-context mode, no shell integration needed for standalone verification)
vivado -mode batch -source synth_cl_minikanren_chirho.tcl 2>&1 | tee ${WORK_DIR_CHIRHO}/build_log_chirho.txt

BUILD_EXIT_CHIRHO=$?
echo "Build exit code: ${BUILD_EXIT_CHIRHO}"

# Collect results
echo "Collecting results..."
cd ${WORK_DIR_CHIRHO}/build/scripts

# Copy checkpoints and reports
cp *.dcp ${WORK_DIR_CHIRHO}/ 2>/dev/null || true
cp *.rpt ${WORK_DIR_CHIRHO}/ 2>/dev/null || true

# Upload all results
echo "Uploading results to S3..."
aws s3 sync ${WORK_DIR_CHIRHO}/ "s3://${BUCKET_CHIRHO}/f2_hbm/results/" --exclude "design.tar.gz"

echo "=== F2 HBM Synthesis Complete ☧ ==="
echo "Exit code: ${BUILD_EXIT_CHIRHO}"
date

# Signal completion
echo "SYNTHESIS_COMPLETE" > /tmp/synth_done_chirho.txt
aws s3 cp /tmp/synth_done_chirho.txt "s3://${BUCKET_CHIRHO}/f2_hbm/results/synth_done_chirho.txt"
USERDATA_EOF
)

# Replace placeholders
USER_DATA_CHIRHO="${USER_DATA_CHIRHO//REGION_PLACEHOLDER/${AWS_REGION_CHIRHO}}"
USER_DATA_CHIRHO="${USER_DATA_CHIRHO//BUCKET_PLACEHOLDER/${S3_BUCKET_CHIRHO}}"

# Step 3: Get security group and subnet
echo ""
echo "=== Step 3: Getting network configuration ==="

# Check for environment variable overrides first
if [ -n "${SUBNET_ID:-}" ] && [ -n "${SECURITY_GROUP:-}" ]; then
    echo "Using environment variables for network config"
    SUBNET_ID_CHIRHO="${SUBNET_ID}"
    SECURITY_GROUP_CHIRHO="${SECURITY_GROUP}"
    VPC_ID_CHIRHO="(from env)"
else
    # Get default VPC
    VPC_ID_CHIRHO=$(aws ec2 describe-vpcs --filters "Name=is-default,Values=true" --query 'Vpcs[0].VpcId' --output text --region "${AWS_REGION_CHIRHO}" 2>/dev/null || echo "")

    if [ -z "${VPC_ID_CHIRHO}" ] || [ "${VPC_ID_CHIRHO}" == "None" ]; then
        # Try to find any VPC
        VPC_ID_CHIRHO=$(aws ec2 describe-vpcs --query 'Vpcs[0].VpcId' --output text --region "${AWS_REGION_CHIRHO}" 2>/dev/null || echo "")
    fi

    if [ -z "${VPC_ID_CHIRHO}" ] || [ "${VPC_ID_CHIRHO}" == "None" ]; then
        echo "No VPC found. Please specify SUBNET_ID and SECURITY_GROUP environment variables."
        exit 1
    fi

    # Get subnet
    SUBNET_ID_CHIRHO=$(aws ec2 describe-subnets --filters "Name=vpc-id,Values=${VPC_ID_CHIRHO}" --query 'Subnets[0].SubnetId' --output text --region "${AWS_REGION_CHIRHO}")

    # Get or create security group
    SECURITY_GROUP_CHIRHO=$(aws ec2 describe-security-groups --filters "Name=group-name,Values=minikanren-synth-sg-chirho" --query 'SecurityGroups[0].GroupId' --output text --region "${AWS_REGION_CHIRHO}" 2>/dev/null || echo "")

    if [ -z "${SECURITY_GROUP_CHIRHO}" ] || [ "${SECURITY_GROUP_CHIRHO}" == "None" ]; then
        echo "Creating security group..."
        SECURITY_GROUP_CHIRHO=$(aws ec2 create-security-group \
            --group-name minikanren-synth-sg-chirho \
            --description "Security group for miniKanren FPGA synthesis" \
            --vpc-id "${VPC_ID_CHIRHO}" \
            --region "${AWS_REGION_CHIRHO}" \
            --query 'GroupId' --output text)

        # Allow SSH
        aws ec2 authorize-security-group-ingress \
            --group-id "${SECURITY_GROUP_CHIRHO}" \
            --protocol tcp --port 22 --cidr 0.0.0.0/0 \
            --region "${AWS_REGION_CHIRHO}"
    fi
fi

echo "VPC: ${VPC_ID_CHIRHO}"
echo "Subnet: ${SUBNET_ID_CHIRHO}"
echo "Security Group: ${SECURITY_GROUP_CHIRHO}"

# Step 4: Create IAM role for S3 access
echo ""
echo "=== Step 4: Setting up IAM role for S3 access ==="

ROLE_NAME_CHIRHO="minikanren-synth-role-chirho"
INSTANCE_PROFILE_CHIRHO="minikanren-synth-profile-chirho"

# Check if role exists
if ! aws iam get-role --role-name ${ROLE_NAME_CHIRHO} --region "${AWS_REGION_CHIRHO}" 2>/dev/null; then
    echo "Creating IAM role..."

    # Create trust policy
    cat > /tmp/trust-policy.json << 'TRUSTPOLICY'
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": {
                "Service": "ec2.amazonaws.com"
            },
            "Action": "sts:AssumeRole"
        }
    ]
}
TRUSTPOLICY

    aws iam create-role \
        --role-name ${ROLE_NAME_CHIRHO} \
        --assume-role-policy-document file:///tmp/trust-policy.json \
        --region "${AWS_REGION_CHIRHO}"

    # Attach S3 policy
    aws iam attach-role-policy \
        --role-name ${ROLE_NAME_CHIRHO} \
        --policy-arn arn:aws:iam::aws:policy/AmazonS3FullAccess \
        --region "${AWS_REGION_CHIRHO}"

    # Create instance profile
    aws iam create-instance-profile \
        --instance-profile-name ${INSTANCE_PROFILE_CHIRHO} \
        --region "${AWS_REGION_CHIRHO}" 2>/dev/null || true

    # Add role to instance profile
    aws iam add-role-to-instance-profile \
        --instance-profile-name ${INSTANCE_PROFILE_CHIRHO} \
        --role-name ${ROLE_NAME_CHIRHO} \
        --region "${AWS_REGION_CHIRHO}" 2>/dev/null || true

    # Wait for profile to be ready
    echo "Waiting for IAM profile to propagate..."
    sleep 10
fi

echo "IAM role: ${ROLE_NAME_CHIRHO}"

# Step 5: Launch instance
echo ""
echo "=== Step 5: Launching c5.9xlarge instance ==="

INSTANCE_ID_CHIRHO=$(aws ec2 run-instances \
    --image-id "${FPGA_DEV_AMI_CHIRHO}" \
    --instance-type "${SYNTH_INSTANCE_CHIRHO}" \
    --key-name "${KEY_NAME_CHIRHO}" \
    --security-group-ids "${SECURITY_GROUP_CHIRHO}" \
    --subnet-id "${SUBNET_ID_CHIRHO}" \
    --associate-public-ip-address \
    --iam-instance-profile Name=${INSTANCE_PROFILE_CHIRHO} \
    --region "${AWS_REGION_CHIRHO}" \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=minikanren-f2-hbm-synth-chirho},{Key=Project,Value=miniKanren}]" \
    --user-data "${USER_DATA_CHIRHO}" \
    --query 'Instances[0].InstanceId' \
    --output text)

echo "Instance launched: ${INSTANCE_ID_CHIRHO}"
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
echo "============================================================"
echo "  Instance Ready ☧"
echo "============================================================"
echo ""
echo "Instance ID: ${INSTANCE_ID_CHIRHO}"
echo "Public IP:   ${PUBLIC_IP_CHIRHO}"
echo ""
echo "To connect:"
echo "  ssh -i ~/.ssh/${KEY_NAME_CHIRHO}.pem ${SSH_USER_CHIRHO}@${PUBLIC_IP_CHIRHO}"
echo ""
echo "To check synthesis progress:"
echo "  ssh -i ~/.ssh/${KEY_NAME_CHIRHO}.pem ${SSH_USER_CHIRHO}@${PUBLIC_IP_CHIRHO} 'sudo tail -f /var/log/f2_synth_chirho.log'"
echo ""
echo "To check S3 results:"
echo "  aws s3 ls s3://${S3_BUCKET_CHIRHO}/f2_hbm/results/"
echo ""
echo "To check completion:"
echo "  aws s3 ls s3://${S3_BUCKET_CHIRHO}/f2_hbm/results/synth_done_chirho.txt"
echo ""
echo "To download results:"
echo "  aws s3 sync s3://${S3_BUCKET_CHIRHO}/f2_hbm/results/ ./f2_results_chirho/"
echo ""
echo "REMEMBER to terminate when done:"
echo "  aws ec2 terminate-instances --instance-ids ${INSTANCE_ID_CHIRHO}"
echo ""
echo "============================================================"
