#!/bin/bash
# Create Amazon FPGA Image (AFI) ☧
#
# Creates an AFI from the synthesized DCP checkpoint.
# Part of P5-01: Physical FPGA Incarnation.
#
# Note: AFI creation takes 1-2 hours to complete.

set -euo pipefail

SCRIPT_DIR_CHIRHO="$(cd "$(dirname "$0")" && pwd)"

# Load configuration
if [ -f "${SCRIPT_DIR_CHIRHO}/config_chirho.sh" ]; then
    source "${SCRIPT_DIR_CHIRHO}/config_chirho.sh"
else
    echo "ERROR: Run setup_chirho.sh first"
    exit 1
fi

echo "=== Create Amazon FPGA Image ☧ ==="

# Check if DCP exists
DCP_S3_PATH="s3://${S3_BUCKET_CHIRHO}/results/minikanren_chirho_routed.dcp"
if ! aws s3 ls "${DCP_S3_PATH}" 2>/dev/null; then
    echo "ERROR: DCP checkpoint not found at ${DCP_S3_PATH}"
    echo "Run synthesis first with ./launch_synth_chirho.sh"
    exit 1
fi

echo "DCP found: ${DCP_S3_PATH}"

# Create logs directory in S3
aws s3api put-object --bucket "${S3_BUCKET_CHIRHO}" --key "logs/" || true

# Create AFI
echo ""
echo "Creating AFI (this will take 1-2 hours to complete)..."
AFI_RESULT=$(aws ec2 create-fpga-image \
    --region "${AWS_REGION_CHIRHO}" \
    --name "minikanren-search-engine-chirho" \
    --description "miniKanren 1-bit search engine - bit-parallel logic programming" \
    --input-storage-location "Bucket=${S3_BUCKET_CHIRHO},Key=results/minikanren_chirho_routed.dcp" \
    --logs-storage-location "Bucket=${S3_BUCKET_CHIRHO},Key=logs/" \
    --output json)

AFI_ID_CHIRHO=$(echo "${AFI_RESULT}" | grep -o '"FpgaImageId": "[^"]*"' | cut -d'"' -f4)
AGFI_ID_CHIRHO=$(echo "${AFI_RESULT}" | grep -o '"FpgaImageGlobalId": "[^"]*"' | cut -d'"' -f4)

echo ""
echo "=== AFI Creation Initiated ☧ ==="
echo "AFI ID:  ${AFI_ID_CHIRHO}"
echo "AGFI ID: ${AGFI_ID_CHIRHO}"
echo ""

# Save IDs
echo "${AFI_ID_CHIRHO}" > "${SCRIPT_DIR_CHIRHO}/afi_id_chirho.txt"
echo "${AGFI_ID_CHIRHO}" > "${SCRIPT_DIR_CHIRHO}/agfi_id_chirho.txt"

echo "IDs saved to:"
echo "  ${SCRIPT_DIR_CHIRHO}/afi_id_chirho.txt"
echo "  ${SCRIPT_DIR_CHIRHO}/agfi_id_chirho.txt"
echo ""
echo "To check status:"
echo "  ./check_afi_chirho.sh"
echo ""
echo "Or manually:"
echo "  aws ec2 describe-fpga-images --fpga-image-ids ${AFI_ID_CHIRHO}"
