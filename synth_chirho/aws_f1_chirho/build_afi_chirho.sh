#!/bin/bash
# For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
# Build AFI with AWS HDK ☧
#
# Run this script on an FPGA Developer AMI (z1d.xlarge recommended)
# It integrates our design with the AWS Shell and creates the AFI.

set -euo pipefail

echo "=== miniKanren AFI Build ☧ ==="
date

# Configuration
BUCKET_CHIRHO="minikanren-fpga-chirho-686672719245"
REGION_CHIRHO="us-east-1"
export AWS_DEFAULT_REGION="${REGION_CHIRHO}"

# Set up HDK
if [ ! -d "$HOME/aws-fpga" ]; then
    echo "Cloning AWS FPGA repo..."
    cd "$HOME"
    git clone https://github.com/aws/aws-fpga.git
fi

cd "$HOME/aws-fpga"
source hdk_setup.sh

# Create CL directory from template
CL_NAME_CHIRHO="cl_minikanren_chirho"
CL_PATH_CHIRHO="$HDK_DIR/cl/developer_designs/${CL_NAME_CHIRHO}"

if [ -d "$CL_PATH_CHIRHO" ]; then
    echo "Removing existing CL directory..."
    rm -rf "$CL_PATH_CHIRHO"
fi

echo "Creating CL from template..."
mkdir -p "$CL_PATH_CHIRHO"
cp -r "$HDK_DIR/cl/cl_hello_world/"* "$CL_PATH_CHIRHO/"

# Download our design files from S3
echo "Downloading design files..."
aws s3 cp "s3://${BUCKET_CHIRHO}/design/searchEngineChirho.v" "$CL_PATH_CHIRHO/design/"
aws s3 cp "s3://${BUCKET_CHIRHO}/design/cl_minikanren_chirho.v" "$CL_PATH_CHIRHO/design/"
aws s3 cp "s3://${BUCKET_CHIRHO}/design/timing_chirho.xdc" "$CL_PATH_CHIRHO/build/constraints/"

# Update the CL sources list
cat > "$CL_PATH_CHIRHO/build/scripts/encrypt.tcl" << 'ENCRYPT_TCL_CHIRHO'
# miniKanren CL encryption script ☧
set CL_MODULE cl_minikanren_chirho

# Add design sources
set_property top ${CL_MODULE} [current_fileset]

# Add our design files
add_files -norecurse [list \
    $CL_DIR/design/cl_minikanren_chirho.v \
    $CL_DIR/design/searchEngineChirho.v \
]
ENCRYPT_TCL_CHIRHO

# Update filelist
cat > "$CL_PATH_CHIRHO/build/scripts/filelist.tcl" << 'FILELIST_TCL_CHIRHO'
# miniKanren CL file list ☧
set CL_MODULE cl_minikanren_chirho

# Design files
set cl_design_files [list \
    $CL_DIR/design/cl_minikanren_chirho.v \
    $CL_DIR/design/searchEngineChirho.v \
]

foreach file $cl_design_files {
    add_files -norecurse $file
}

set_property top ${CL_MODULE} [current_fileset]
FILELIST_TCL_CHIRHO

# Run the build
export CL_DIR="$CL_PATH_CHIRHO"
cd "$CL_PATH_CHIRHO/build/scripts"

echo "Starting HDK build (this takes 4-8 hours)..."
./aws_build_dcp_from_cl.sh -strategy CONGESTION -clock_recipe_a A0 2>&1 | tee build_log_chirho.txt

BUILD_EXIT_CHIRHO=$?
echo "Build exit code: ${BUILD_EXIT_CHIRHO}"

# Find and upload the generated tarball
if [ -f "$CL_PATH_CHIRHO/build/checkpoints/to_aws/"*.tar ]; then
    TARBALL_CHIRHO=$(ls "$CL_PATH_CHIRHO/build/checkpoints/to_aws/"*.tar | head -1)
    echo "Uploading tarball: $TARBALL_CHIRHO"
    aws s3 cp "$TARBALL_CHIRHO" "s3://${BUCKET_CHIRHO}/afi-input/"

    TARBALL_KEY_CHIRHO="afi-input/$(basename $TARBALL_CHIRHO)"

    # Create AFI
    echo "Creating AFI..."
    aws ec2 create-fpga-image \
        --name "minikanren-chirho-v1" \
        --description "miniKanren 1-bit FPGA solver with AWS Shell ☧" \
        --input-storage-location "Bucket=${BUCKET_CHIRHO},Key=${TARBALL_KEY_CHIRHO}" \
        --logs-storage-location "Bucket=${BUCKET_CHIRHO},Key=afi-logs/" \
        --region "${REGION_CHIRHO}" | tee afi_result_chirho.json

    aws s3 cp afi_result_chirho.json "s3://${BUCKET_CHIRHO}/results/"
else
    echo "ERROR: No tarball found in checkpoints/to_aws/"
    ls -la "$CL_PATH_CHIRHO/build/checkpoints/" || true
fi

# Upload build log
aws s3 cp build_log_chirho.txt "s3://${BUCKET_CHIRHO}/results/"

echo "=== Build Complete ☧ ==="
date
