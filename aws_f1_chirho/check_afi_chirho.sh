#!/bin/bash
# Check AFI Status ☧
#
# Monitors the AFI creation progress.
# Part of P5-01: Physical FPGA Incarnation.

set -euo pipefail

SCRIPT_DIR_CHIRHO="$(cd "$(dirname "$0")" && pwd)"

# Load configuration
if [ -f "${SCRIPT_DIR_CHIRHO}/config_chirho.sh" ]; then
    source "${SCRIPT_DIR_CHIRHO}/config_chirho.sh"
else
    echo "ERROR: Run setup_chirho.sh first"
    exit 1
fi

# Check if AFI ID exists
AFI_ID_FILE="${SCRIPT_DIR_CHIRHO}/afi_id_chirho.txt"
if [ ! -f "${AFI_ID_FILE}" ]; then
    echo "ERROR: No AFI ID found. Run './create_afi_chirho.sh' first."
    exit 1
fi

AFI_ID_CHIRHO=$(cat "${AFI_ID_FILE}")

echo "=== Check AFI Status ☧ ==="
echo "AFI ID: ${AFI_ID_CHIRHO}"
echo ""

# Get status
RESULT=$(aws ec2 describe-fpga-images \
    --fpga-image-ids "${AFI_ID_CHIRHO}" \
    --region "${AWS_REGION_CHIRHO}" \
    --output json)

STATE=$(echo "${RESULT}" | grep -o '"Code": "[^"]*"' | head -1 | cut -d'"' -f4)
STATE_MSG=$(echo "${RESULT}" | grep -o '"Message": "[^"]*"' | head -1 | cut -d'"' -f4 || echo "")

echo "State: ${STATE}"
if [ -n "${STATE_MSG}" ]; then
    echo "Message: ${STATE_MSG}"
fi

case "${STATE}" in
    "available")
        echo ""
        echo "AFI is READY for use!"
        echo "You can now run: ./run_f1_chirho.sh"
        ;;
    "pending")
        echo ""
        echo "AFI is still being created..."
        echo "This typically takes 1-2 hours."
        echo ""
        echo "Re-run this script to check status, or watch with:"
        echo "  watch -n 60 './check_afi_chirho.sh'"
        ;;
    "failed")
        echo ""
        echo "ERROR: AFI creation FAILED"
        echo "Check logs in S3: s3://${S3_BUCKET_CHIRHO}/logs/"
        ;;
    *)
        echo ""
        echo "Unknown state: ${STATE}"
        ;;
esac

echo ""
echo "Full details:"
echo "${RESULT}" | head -50
