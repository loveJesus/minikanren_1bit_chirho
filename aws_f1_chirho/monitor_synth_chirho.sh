#!/bin/bash
# For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
# Monitor Synthesis Progress ☧
#
# Checks S3 for synthesis results and displays instance status.
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

echo "=== Monitor Synthesis Progress ☧ ==="
echo ""

# Check instance status
INSTANCE_FILE_CHIRHO="${SCRIPT_DIR_CHIRHO}/synth_instance_id_chirho.txt"
if [ -f "${INSTANCE_FILE_CHIRHO}" ]; then
    INSTANCE_ID_CHIRHO=$(cat "${INSTANCE_FILE_CHIRHO}")
    echo "Instance ID: ${INSTANCE_ID_CHIRHO}"

    STATUS_CHIRHO=$(aws ec2 describe-instances \
        --instance-ids "${INSTANCE_ID_CHIRHO}" \
        --region "${AWS_REGION_CHIRHO}" \
        --query "Reservations[0].Instances[0].[State.Name,LaunchTime]" \
        --output text 2>/dev/null || echo "terminated unknown")

    STATE_CHIRHO=$(echo "${STATUS_CHIRHO}" | cut -f1)
    LAUNCH_CHIRHO=$(echo "${STATUS_CHIRHO}" | cut -f2)

    echo "State: ${STATE_CHIRHO}"
    echo "Launched: ${LAUNCH_CHIRHO}"

    if [ "${STATE_CHIRHO}" == "running" ]; then
        IP_CHIRHO=$(aws ec2 describe-instances \
            --instance-ids "${INSTANCE_ID_CHIRHO}" \
            --region "${AWS_REGION_CHIRHO}" \
            --query "Reservations[0].Instances[0].PublicIpAddress" \
            --output text 2>/dev/null || echo "N/A")
        echo "IP: ${IP_CHIRHO}"
    fi
else
    echo "No instance file found"
fi

echo ""
echo "=== S3 Results ==="
aws s3 ls "s3://${S3_BUCKET_CHIRHO}/results/" --region "${AWS_REGION_CHIRHO}" 2>/dev/null || echo "(none yet)"

echo ""
echo "=== Estimated Time Remaining ==="
if [ -f "${INSTANCE_FILE_CHIRHO}" ] && [ "${STATE_CHIRHO:-}" == "running" ]; then
    # Calculate elapsed time
    LAUNCH_EPOCH_CHIRHO=$(date -j -f "%Y-%m-%dT%H:%M:%S+00:00" "${LAUNCH_CHIRHO}" "+%s" 2>/dev/null || echo "0")
    NOW_EPOCH_CHIRHO=$(date "+%s")
    ELAPSED_CHIRHO=$(( (NOW_EPOCH_CHIRHO - LAUNCH_EPOCH_CHIRHO) / 60 ))
    echo "Elapsed: ${ELAPSED_CHIRHO} minutes"

    if [ "${ELAPSED_CHIRHO}" -lt 120 ]; then
        echo "Likely remaining: 2-4 hours (synthesis phase)"
    elif [ "${ELAPSED_CHIRHO}" -lt 240 ]; then
        echo "Likely remaining: 1-2 hours"
    else
        echo "Should be completing soon - check S3 results"
    fi
else
    echo "Instance not running"
fi

echo ""
echo "=== Commands ==="
echo "To watch: watch -n 60 './monitor_synth_chirho.sh'"
echo "To connect: ssh -i ~/.ssh/${KEY_NAME_CHIRHO}.pem centos@\${IP_CHIRHO:-IP}"
echo "To download: ./download_results_chirho.sh"
echo "To terminate: aws ec2 terminate-instances --instance-ids ${INSTANCE_ID_CHIRHO:-YOUR_INSTANCE_ID}"
