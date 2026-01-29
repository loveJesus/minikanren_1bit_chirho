#!/bin/bash
# ============================================================================
# For God so loved the world - John 3:16
# Check AFI creation status ☧
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
AFI_FILE="${SCRIPT_DIR}/../afi_id_chirho.txt"

if [ -f "$AFI_FILE" ]; then
    AFI_ID=$(cat "$AFI_FILE")
elif [ -n "$1" ]; then
    AFI_ID="$1"
else
    echo "Usage: $0 [afi-id]"
    echo "Or create afi_id_chirho.txt with the AFI ID"
    exit 1
fi

echo "Checking AFI: $AFI_ID"
aws ec2 describe-fpga-images --fpga-image-ids "$AFI_ID" \
    --query 'FpgaImages[0].{State:State.Code,Name:Name,CreateTime:CreateTime}' \
    --output table

# Full details
echo ""
echo "Full status:"
aws ec2 describe-fpga-images --fpga-image-ids "$AFI_ID"
