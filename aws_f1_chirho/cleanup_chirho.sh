#!/bin/bash
# For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
# Cleanup AWS Resources ☧
#
# Terminates any running instances to stop billing.
# Part of P5-01: Physical FPGA Incarnation.

set -euo pipefail

SCRIPT_DIR_CHIRHO="$(cd "$(dirname "$0")" && pwd)"

# Load configuration
if [ -f "${SCRIPT_DIR_CHIRHO}/config_chirho.sh" ]; then
    source "${SCRIPT_DIR_CHIRHO}/config_chirho.sh"
else
    echo "No config found - checking for instance files..."
fi

echo "=== Cleanup AWS Resources ☧ ==="

# Terminate synthesis instance if exists
if [ -f "${SCRIPT_DIR_CHIRHO}/synth_instance_id_chirho.txt" ]; then
    SYNTH_ID_CHIRHO=$(cat "${SCRIPT_DIR_CHIRHO}/synth_instance_id_chirho.txt")
    echo "Terminating synthesis instance: ${SYNTH_ID_CHIRHO}"
    aws ec2 terminate-instances --instance-ids "${SYNTH_ID_CHIRHO}" --region "${AWS_REGION_CHIRHO:-us-east-1}" 2>/dev/null || echo "  (already terminated or not found)"
    rm "${SCRIPT_DIR_CHIRHO}/synth_instance_id_chirho.txt"
fi

# Terminate F1 instance if exists
if [ -f "${SCRIPT_DIR_CHIRHO}/f1_instance_id_chirho.txt" ]; then
    F1_ID_CHIRHO=$(cat "${SCRIPT_DIR_CHIRHO}/f1_instance_id_chirho.txt")
    echo "Terminating F1 instance: ${F1_ID_CHIRHO}"
    aws ec2 terminate-instances --instance-ids "${F1_ID_CHIRHO}" --region "${AWS_REGION_CHIRHO:-us-east-1}" 2>/dev/null || echo "  (already terminated or not found)"
    rm "${SCRIPT_DIR_CHIRHO}/f1_instance_id_chirho.txt"
fi

# List any remaining instances with our tag
echo ""
echo "Checking for any remaining minikanren instances..."
aws ec2 describe-instances \
    --region "${AWS_REGION_CHIRHO:-us-east-1}" \
    --filters "Name=tag:Name,Values=minikanren-*-chirho" "Name=instance-state-name,Values=running,pending" \
    --query "Reservations[*].Instances[*].[InstanceId,InstanceType,State.Name]" \
    --output text || echo "None found"

echo ""
echo "=== Cleanup Complete ☧ ==="
echo ""
echo "Note: S3 bucket and AFI are NOT deleted (they may be useful later)."
echo "To delete S3 bucket:"
echo "  aws s3 rb s3://${S3_BUCKET_CHIRHO:-minikanren-fpga-chirho-*} --force"
