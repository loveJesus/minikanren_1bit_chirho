#!/bin/bash
# ============================================================================
# ☧ For God so loved the world, that He gave His only begotten Son,
# that whosoever believeth in Him should not perish, but have everlasting life.
# - John 3:16
# ============================================================================
#
# AWS F1 CL Build Launcher - miniKanren Search Engine
#
# This script:
#   1. Uploads our CL design to S3
#   2. Launches a z1d.xlarge instance with FPGA Developer AMI 1.18.0
#   3. Builds the CL with Vivado 2025.1
#   4. Creates the AFI
#
# Prerequisites:
#   - AWS CLI configured with appropriate credentials
#   - S3 bucket: minikanren-fpga-chirho-686672719245
#
# Usage: ./launch_f1_build_chirho.sh
# ============================================================================

set -e

# Configuration
S3_BUCKET_CHIRHO="minikanren-fpga-chirho-686672719245"
S3_PREFIX_CHIRHO="cl_minikanren_chirho"
AMI_ID_CHIRHO="ami-01198b89d80ebfdd2"  # FPGA Developer AMI 1.17.0 (Ubuntu, Vivado 2024.2)
INSTANCE_TYPE_CHIRHO="z1d.xlarge"
KEY_NAME_CHIRHO="minikanren-fpga-key-chirho"
REGION_CHIRHO="us-east-1"
SUBNET_ID_CHIRHO="subnet-c5104ea0"  # us-east-1a
SECURITY_GROUP_CHIRHO="sg-0b29ce11e8f0878cd"  # minikanren-fpga-sg-chirho

# Script directory
SCRIPT_DIR_CHIRHO="$(cd "$(dirname "$0")" && pwd)"
CL_DIR_CHIRHO="$SCRIPT_DIR_CHIRHO/cl_minikanren_chirho"

echo "=== miniKanren F1 CL Build Launcher ☧ ==="
date

# ============================================================================
# Step 1: Upload CL design to S3
# ============================================================================

echo ""
echo "Step 1: Uploading CL design to S3..."

# Create tarball of CL design
cd "$CL_DIR_CHIRHO"
tar -czvf /tmp/cl_minikanren_chirho.tar.gz \
    design/ \
    build/scripts/ \
    software/runtime/

# Upload to S3
aws s3 cp /tmp/cl_minikanren_chirho.tar.gz \
    s3://$S3_BUCKET_CHIRHO/$S3_PREFIX_CHIRHO/cl_minikanren_chirho.tar.gz

echo "CL design uploaded to: s3://$S3_BUCKET_CHIRHO/$S3_PREFIX_CHIRHO/cl_minikanren_chirho.tar.gz"

# ============================================================================
# Step 2: Create user-data script
# ============================================================================

echo ""
echo "Step 2: Creating user-data script..."

cat > /tmp/f1_build_userdata_chirho.sh << 'USERDATA_EOF'
#!/bin/bash
set -ex
exec > >(tee /var/log/user-data.log) 2>&1

echo "=== miniKanren F1 CL Build (AMI 1.17.0 - Vivado 2024.2) ☧ ==="
date

export HOME=/root
export AWS_DEFAULT_REGION=us-east-1

# Ubuntu AMI uses /home/ubuntu
cd /home/ubuntu

# Clone HDK if needed
if [ ! -d "aws-fpga" ]; then
    git clone https://github.com/aws/aws-fpga.git
fi

cd aws-fpga
source hdk_setup.sh

# Download CL design from S3
mkdir -p /tmp/cl_build_chirho
cd /tmp/cl_build_chirho
aws s3 cp s3://minikanren-fpga-chirho-686672719245/cl_minikanren_chirho/cl_minikanren_chirho.tar.gz .
tar -xzvf cl_minikanren_chirho.tar.gz

# Copy to HDK developer_designs
export CL_DIR=/tmp/cl_build_chirho
mkdir -p $HDK_DIR/cl/developer_designs/cl_minikanren_chirho
cp -r design build software $HDK_DIR/cl/developer_designs/cl_minikanren_chirho/
export CL_DIR=$HDK_DIR/cl/developer_designs/cl_minikanren_chirho

# Create proper build directory structure
mkdir -p $CL_DIR/build/checkpoints
mkdir -p $CL_DIR/build/logs
mkdir -p $CL_DIR/build/reports
mkdir -p $CL_DIR/build/constraints

# Create timing constraints
cat > $CL_DIR/build/constraints/cl_timing_chirho.xdc << 'XDC_EOF'
# miniKanren CL Timing Constraints
# The engine runs on a divided clock (62.5MHz from 250MHz)
# Main shell clock is 250MHz (4ns period)
XDC_EOF

# Create synthesis script
cat > $CL_DIR/build/scripts/synth_cl_chirho.tcl << 'TCL_EOF'
# ============================================================================
# miniKanren CL Synthesis Script
# ============================================================================

# Get environment
set CL_DIR $::env(CL_DIR)
set HDK_DIR $::env(HDK_DIR)

puts "CL_DIR: $CL_DIR"
puts "HDK_DIR: $HDK_DIR"

# Create project
create_project -force cl_minikanren_chirho $CL_DIR/build/vivado_project -part xcvu9p-flgb2104-2-i

# Add our design files
add_files -fileset sources_1 $CL_DIR/design/cl_minikanren_chirho_defines.vh
add_files -fileset sources_1 $CL_DIR/design/searchEngineChirho.v
add_files -fileset sources_1 $CL_DIR/design/cl_minikanren_chirho.sv

# Set top module
set_property top cl_minikanren_chirho [current_fileset]

# Add constraints
if {[file exists $CL_DIR/build/constraints/cl_timing_chirho.xdc]} {
    add_files -fileset constrs_1 $CL_DIR/build/constraints/cl_timing_chirho.xdc
}

# Create 20ns clock constraint (50MHz)
create_clock -period 20.000 -name clk_main_a0 [get_ports clk_main_a0]

# Run synthesis
synth_design -top cl_minikanren_chirho -part xcvu9p-flgb2104-2-i
report_timing_summary -file $CL_DIR/build/reports/post_synth_timing_chirho.rpt
report_utilization -file $CL_DIR/build/reports/post_synth_util_chirho.rpt
write_checkpoint -force $CL_DIR/build/checkpoints/post_synth_chirho.dcp

# Run place
opt_design
place_design
report_timing_summary -file $CL_DIR/build/reports/post_place_timing_chirho.rpt
write_checkpoint -force $CL_DIR/build/checkpoints/post_place_chirho.dcp

# Run route
route_design
report_timing_summary -file $CL_DIR/build/reports/post_route_timing_chirho.rpt
report_utilization -file $CL_DIR/build/reports/post_route_util_chirho.rpt
report_power -file $CL_DIR/build/reports/power_chirho.rpt

# Write final checkpoint
write_checkpoint -force $CL_DIR/build/checkpoints/cl_minikanren_chirho.SH_CL_routed.dcp

puts "=== Synthesis Complete ☧ ==="
TCL_EOF

# Run Vivado synthesis
cd $CL_DIR/build/scripts
vivado -mode batch -source synth_cl_chirho.tcl \
    -log $CL_DIR/build/logs/vivado_chirho.log \
    -journal $CL_DIR/build/logs/vivado_chirho.jou \
    2>&1 | tee $CL_DIR/build/logs/vivado_console_chirho.log

# Check if synthesis succeeded
if [ -f "$CL_DIR/build/checkpoints/cl_minikanren_chirho.SH_CL_routed.dcp" ]; then
    echo "=== Synthesis SUCCESSFUL ☧ ==="

    # Get shell info (use default if not available)
    SHELL_VERSION_CHIRHO="0x04261818"
    HDK_VER_CHIRHO="v2.2.2"
    if [ -f "$HDK_DIR/hdk_version.txt" ]; then
        HDK_VER_CHIRHO=$(cat $HDK_DIR/hdk_version.txt | head -1)
    fi

    # Create manifest
    cat > $CL_DIR/build/checkpoints/manifest.txt << MANIFEST_EOF
manifest_format_version=2
pci_vendor_id=0x1D0F
pci_device_id=0xF000
pci_subsystem_id=0x1D51
pci_subsystem_vendor_id=0xFEDD
dcp_file_name=cl_minikanren_chirho.SH_CL_routed.dcp
shell_version=$SHELL_VERSION_CHIRHO
hdk_version=$HDK_VER_CHIRHO
date=$(date +%Y_%m_%d-%H%M%S)
clock_recipe_a=A0
clock_recipe_b=B0
clock_recipe_c=C0
MANIFEST_EOF

    echo "=== Manifest ==="
    cat $CL_DIR/build/checkpoints/manifest.txt

    # Create tarball for AFI
    cd $CL_DIR/build/checkpoints
    tar -cvf cl_minikanren_afi_chirho.tar manifest.txt cl_minikanren_chirho.SH_CL_routed.dcp

    # Upload tarball to S3
    aws s3 cp cl_minikanren_afi_chirho.tar s3://minikanren-fpga-chirho-686672719245/afi_build/cl_minikanren_afi_chirho.tar

    # Create AFI
    echo "=== Creating AFI ☧ ==="
    aws ec2 create-fpga-image \
        --name "minikanren-logic-engine-v1-chirho" \
        --description "miniKanren 1-bit Logic Engine - Vivado 2024.2" \
        --input-storage-location Bucket=minikanren-fpga-chirho-686672719245,Key=afi_build/cl_minikanren_afi_chirho.tar \
        --logs-storage-location Bucket=minikanren-fpga-chirho-686672719245,Key=afi_logs/ \
        | tee /tmp/afi_result_chirho.json

    # Upload AFI result
    aws s3 cp /tmp/afi_result_chirho.json s3://minikanren-fpga-chirho-686672719245/afi_build/afi_result_chirho.json

else
    echo "=== Synthesis FAILED ☧ ==="
fi

# Upload all logs and reports
aws s3 sync $CL_DIR/build/logs/ s3://minikanren-fpga-chirho-686672719245/afi_build/logs/
aws s3 sync $CL_DIR/build/reports/ s3://minikanren-fpga-chirho-686672719245/afi_build/reports/
aws s3 cp /var/log/user-data.log s3://minikanren-fpga-chirho-686672719245/afi_build/user-data.log

echo "=== Build Script Complete ☧ ==="
date
USERDATA_EOF

# ============================================================================
# Step 3: Launch EC2 Instance
# ============================================================================

echo ""
echo "Step 3: Launching EC2 build instance..."

# Launch instance
INSTANCE_ID_CHIRHO=$(aws ec2 run-instances \
    --image-id $AMI_ID_CHIRHO \
    --instance-type $INSTANCE_TYPE_CHIRHO \
    --key-name $KEY_NAME_CHIRHO \
    --region $REGION_CHIRHO \
    --subnet-id $SUBNET_ID_CHIRHO \
    --security-group-ids $SECURITY_GROUP_CHIRHO \
    --associate-public-ip-address \
    --iam-instance-profile Name=minikanren-fpga-role-chirho \
    --user-data file:///tmp/f1_build_userdata_chirho.sh \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=minikanren-f1-build-chirho},{Key=Project,Value=minikanren-chirho}]" \
    --query 'Instances[0].InstanceId' \
    --output text)

echo ""
echo "=== Instance Launched ☧ ==="
echo "Instance ID: $INSTANCE_ID_CHIRHO"
echo ""
echo "Monitor build progress:"
echo "  aws s3 cp s3://$S3_BUCKET_CHIRHO/afi_build/user-data.log - | tail -50"
echo ""
echo "Check AFI status (after build completes):"
echo "  aws ec2 describe-fpga-images --owners self"
echo ""
echo "SSH to instance:"
echo "  ssh -i ~/.ssh/$KEY_NAME_CHIRHO.pem ubuntu@\$(aws ec2 describe-instances --instance-ids $INSTANCE_ID_CHIRHO --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)"
echo ""
echo "=== Launch Complete ☧ ==="
