#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16
# Create AFI from DCP tarball ☧
# ============================================================================

set -e

# Configuration
BUCKET_CHIRHO="minikanren-fpga-chirho"
REGION_CHIRHO="us-east-1"
DCP_S3_PATH="s3://${BUCKET_CHIRHO}/f2_hbm_hdk/dcp"

# Find the most recent DCP tarball
echo "Looking for DCP tarball in S3..."
DCP_TAR=$(aws s3 ls ${DCP_S3_PATH}/ 2>/dev/null | grep "Developer_CL.tar" | sort | tail -1 | awk '{print $4}')

if [ -z "$DCP_TAR" ]; then
    echo "ERROR: No DCP tarball found in ${DCP_S3_PATH}/"
    echo "Make sure the HDK build completed successfully."
    exit 1
fi

echo "Found DCP: $DCP_TAR"

# Create AFI
echo "Creating AFI..."
aws ec2 create-fpga-image \
    --region ${REGION_CHIRHO} \
    --name "minikanren-f2-chirho-$(date +%Y%m%d-%H%M%S)" \
    --description "miniKanren 1-bit search engine with HBM - John 3:16" \
    --input-storage-location Bucket=${BUCKET_CHIRHO},Key=f2_hbm_hdk/dcp/${DCP_TAR} \
    --logs-storage-location Bucket=${BUCKET_CHIRHO},Key=f2_hbm_hdk/afi_logs/ \
    --output json | tee /tmp/afi_creation_chirho.json

# Extract AFI ID
AFI_ID=$(cat /tmp/afi_creation_chirho.json | grep -o '"FpgaImageId": "[^"]*"' | cut -d'"' -f4)
AGFI_ID=$(cat /tmp/afi_creation_chirho.json | grep -o '"FpgaImageGlobalId": "[^"]*"' | cut -d'"' -f4)

echo ""
echo "============================================"
echo "AFI Creation Started!"
echo "============================================"
echo "AFI ID:  $AFI_ID"
echo "AGFI ID: $AGFI_ID"
echo ""
echo "Check status with:"
echo "  aws ec2 describe-fpga-images --fpga-image-ids $AFI_ID"
echo ""
echo "Wait for State to change from 'pending' to 'available' (~30-60 min)"
echo ""
echo "Save these IDs:"
echo "$AFI_ID" > /tmp/afi_id_chirho.txt
echo "$AGFI_ID" > /tmp/agfi_id_chirho.txt

# Save to project
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
echo "$AFI_ID" > "${SCRIPT_DIR}/../afi_id_chirho.txt"
echo "$AGFI_ID" > "${SCRIPT_DIR}/../agfi_id_chirho.txt"

echo "Soli Deo Gloria ☧"
