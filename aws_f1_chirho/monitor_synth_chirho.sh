#!/bin/bash
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
INSTANCE_FILE="${SCRIPT_DIR_CHIRHO}/synth_instance_id_chirho.txt"
if [ -f "${INSTANCE_FILE}" ]; then
    INSTANCE_ID=$(cat "${INSTANCE_FILE}")
    echo "Instance ID: ${INSTANCE_ID}"

    STATUS=$(aws ec2 describe-instances \
        --instance-ids "${INSTANCE_ID}" \
        --region "${AWS_REGION_CHIRHO}" \
        --query "Reservations[0].Instances[0].[State.Name,LaunchTime]" \
        --output text 2>/dev/null || echo "terminated unknown")

    STATE=$(echo "${STATUS}" | cut -f1)
    LAUNCH=$(echo "${STATUS}" | cut -f2)

    echo "State: ${STATE}"
    echo "Launched: ${LAUNCH}"

    if [ "${STATE}" == "running" ]; then
        IP=$(aws ec2 describe-instances \
            --instance-ids "${INSTANCE_ID}" \
            --region "${AWS_REGION_CHIRHO}" \
            --query "Reservations[0].Instances[0].PublicIpAddress" \
            --output text 2>/dev/null || echo "N/A")
        echo "IP: ${IP}"
    fi
else
    echo "No instance file found"
fi

echo ""
echo "=== S3 Results ==="
aws s3 ls "s3://${S3_BUCKET_CHIRHO}/results/" --region "${AWS_REGION_CHIRHO}" 2>/dev/null || echo "(none yet)"

echo ""
echo "=== Estimated Time Remaining ==="
if [ -f "${INSTANCE_FILE}" ] && [ "${STATE}" == "running" ]; then
    # Calculate elapsed time
    LAUNCH_EPOCH=$(date -j -f "%Y-%m-%dT%H:%M:%S+00:00" "${LAUNCH}" "+%s" 2>/dev/null || echo "0")
    NOW_EPOCH=$(date "+%s")
    ELAPSED=$(( (NOW_EPOCH - LAUNCH_EPOCH) / 60 ))
    echo "Elapsed: ${ELAPSED} minutes"

    if [ "${ELAPSED}" -lt 120 ]; then
        echo "Likely remaining: 2-4 hours (synthesis phase)"
    elif [ "${ELAPSED}" -lt 240 ]; then
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
echo "To connect: ssh -i ~/.ssh/${KEY_NAME_CHIRHO}.pem centos@\${IP}"
echo "To download: ./download_results_chirho.sh"
echo "To terminate: aws ec2 terminate-instances --instance-ids ${INSTANCE_ID:-YOUR_INSTANCE_ID}"
