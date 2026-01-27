#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16
# Launch F2 HBM Synthesis on c5.9xlarge (72GB RAM) ☧
# ============================================================================

set -euo pipefail

SCRIPT_DIR_CHIRHO="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT_CHIRHO="$(dirname "$SCRIPT_DIR_CHIRHO")"

# Configuration
AWS_REGION_CHIRHO="${AWS_REGION:-us-east-1}"
S3_BUCKET_CHIRHO="${S3_BUCKET:-minikanren-fpga-chirho}"
KEY_NAME_CHIRHO="${KEY_NAME:-minikanren-fpga-key}"

# c5.9xlarge: 36 vCPUs, 72 GB RAM - needed for HBM synthesis
SYNTH_INSTANCE_CHIRHO="c5.9xlarge"

# FPGA Developer AMI (Ubuntu 24.04 with Vivado 2024.2)
# Find latest: aws ec2 describe-images --owners amazon --filters "Name=name,Values=*FPGA*Developer*" --query 'Images[*].[ImageId,Name]'
FPGA_DEV_AMI_CHIRHO="${FPGA_DEV_AMI:-ami-0123456789abcdef0}"  # Update with actual AMI

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

# Step 2: Create user data script
echo ""
echo "=== Step 2: Creating launch configuration ==="

USER_DATA_CHIRHO=$(cat << 'USERDATA_EOF'
#!/bin/bash
# For God so loved the world - John 3:16 ☧
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

# Clone F2 HDK if needed
if [ ! -d "/root/aws-fpga" ]; then
    echo "Cloning AWS FPGA HDK..."
    cd /root
    git clone --depth 1 https://github.com/aws/aws-fpga.git
fi

# Set up HDK environment
export HDK_DIR=/root/aws-fpga/hdk
export HDK_COMMON_DIR=${HDK_DIR}/common
export HDK_SHELL_DESIGN_DIR=${HDK_COMMON_DIR}/shell_stable

# Source Vivado
echo "Sourcing Vivado..."
source /opt/Xilinx/Vivado/2024.2/settings64.sh || source /opt/Xilinx/Vivado/*/settings64.sh

# Copy design to HDK structure
CL_DIR_CHIRHO="${HDK_DIR}/cl/developer_designs/cl_minikanren_chirho"
mkdir -p ${CL_DIR_CHIRHO}/design
mkdir -p ${CL_DIR_CHIRHO}/build/scripts

cp ${WORK_DIR_CHIRHO}/design/*.sv ${CL_DIR_CHIRHO}/design/
cp ${WORK_DIR_CHIRHO}/design/*.vh ${CL_DIR_CHIRHO}/design/
cp ${WORK_DIR_CHIRHO}/design/*.v ${CL_DIR_CHIRHO}/design/
cp ${WORK_DIR_CHIRHO}/build/scripts/*.tcl ${CL_DIR_CHIRHO}/build/scripts/

# Run synthesis
echo "Running Vivado synthesis..."
cd ${CL_DIR_CHIRHO}/build/scripts
vivado -mode batch -source synth_cl_minikanren_chirho.tcl 2>&1 | tee ${WORK_DIR_CHIRHO}/vivado_log_chirho.txt

SYNTH_EXIT_CHIRHO=$?
echo "Vivado exit code: ${SYNTH_EXIT_CHIRHO}"

# Upload results
echo "Uploading results..."
RESULTS_DIR_CHIRHO="${CL_DIR_CHIRHO}/build/checkpoints"
mkdir -p ${RESULTS_DIR_CHIRHO}

# Copy reports
cp *.rpt ${WORK_DIR_CHIRHO}/ 2>/dev/null || true
cp *.dcp ${WORK_DIR_CHIRHO}/ 2>/dev/null || true

# Upload all results
aws s3 sync ${WORK_DIR_CHIRHO}/ "s3://${BUCKET_CHIRHO}/f2_hbm/results/" --exclude "design.tar.gz"

echo "=== F2 HBM Synthesis Complete ☧ ==="
echo "Exit code: ${SYNTH_EXIT_CHIRHO}"
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

# Get default VPC
VPC_ID_CHIRHO=$(aws ec2 describe-vpcs --filters "Name=is-default,Values=true" --query 'Vpcs[0].VpcId' --output text --region "${AWS_REGION_CHIRHO}" 2>/dev/null || echo "")

if [ -z "${VPC_ID_CHIRHO}" ] || [ "${VPC_ID_CHIRHO}" == "None" ]; then
    echo "No default VPC found. Please specify SUBNET_ID and SECURITY_GROUP environment variables."
    exit 1
fi

# Get default subnet
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

echo "VPC: ${VPC_ID_CHIRHO}"
echo "Subnet: ${SUBNET_ID_CHIRHO}"
echo "Security Group: ${SECURITY_GROUP_CHIRHO}"

# Step 4: Launch instance
echo ""
echo "=== Step 4: Launching c5.9xlarge instance ==="

INSTANCE_ID_CHIRHO=$(aws ec2 run-instances \
    --image-id "${FPGA_DEV_AMI_CHIRHO}" \
    --instance-type "${SYNTH_INSTANCE_CHIRHO}" \
    --key-name "${KEY_NAME_CHIRHO}" \
    --security-group-ids "${SECURITY_GROUP_CHIRHO}" \
    --subnet-id "${SUBNET_ID_CHIRHO}" \
    --associate-public-ip-address \
    --region "${AWS_REGION_CHIRHO}" \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=minikanren-f2-synth-chirho},{Key=Project,Value=miniKanren}]" \
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
echo "  ssh -i ~/.ssh/${KEY_NAME_CHIRHO}.pem ubuntu@${PUBLIC_IP_CHIRHO}"
echo ""
echo "To check synthesis progress:"
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
