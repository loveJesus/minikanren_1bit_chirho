#!/bin/bash
# ============================================================================
# miniKanren F1 FPGA Test Script ☧
# ============================================================================
# Usage: ./test_f1_chirho.sh
#
# Prerequisites:
#   - Running on an F1 instance (f1.2xlarge or larger)
#   - AWS FPGA Management Tools installed
#   - AFI must be in 'available' state
# ============================================================================

set -e

AGFI_ID="agfi-038ca2f7a81352cb4"
AFI_ID="afi-00e111cc7004d41c5"
SLOT=0

echo "=== miniKanren F1 FPGA Test ☧ ==="
echo "AFI: $AFI_ID"
echo "AGFI: $AGFI_ID"
echo ""

# Check if we're on an F1 instance
if ! lspci | grep -q "Xilinx"; then
    echo "ERROR: This doesn't appear to be an F1 instance (no Xilinx FPGA found)"
    echo "Please run this on an f1.2xlarge or larger instance"
    exit 1
fi

# Check AFI status
echo "=== Checking AFI Status ==="
aws ec2 describe-fpga-images --fpga-image-ids $AFI_ID --region us-east-1 \
    --query 'FpgaImages[0].State.Code' --output text

# Clear any existing FPGA image
echo "=== Clearing FPGA Slot $SLOT ==="
sudo fpga-clear-local-image -S $SLOT

# Load the AFI
echo "=== Loading AFI ==="
sudo fpga-load-local-image -S $SLOT -I $AGFI_ID

# Wait for load to complete
echo "=== Waiting for FPGA Load ==="
sleep 5

# Check load status
echo "=== FPGA Slot Status ==="
sudo fpga-describe-local-image -S $SLOT -R -H

# Verify the image is loaded
STATUS=$(sudo fpga-describe-local-image -S $SLOT -R -H 2>/dev/null | grep "FPGA Image Slot" -A 5 | grep "Status" | awk '{print $NF}')
if [ "$STATUS" != "loaded" ]; then
    echo "ERROR: FPGA image not loaded correctly. Status: $STATUS"
    exit 1
fi

echo ""
echo "=== FPGA Loaded Successfully! ==="
echo ""

# Now test the OCL interface
echo "=== Testing OCL Interface ==="

# The OCL BAR should be accessible via /sys/bus/pci
OCL_BAR=$(lspci -v -d 1d0f:f001 | grep "Memory at" | head -1 | awk '{print $3}')
echo "OCL BAR: $OCL_BAR"

# If fpga-sdk-tools are available, use fpga-read-register
if command -v fpga-read-register &> /dev/null; then
    echo ""
    echo "=== Reading miniKanren Registers ==="

    # Read the hello register (offset 0x500)
    # Should return 0xDEADBEEF or similar magic value
    echo "Hello Register (0x500):"
    sudo fpga-read-register -S $SLOT -o 0x500

    # Read status (offset 0x504)
    echo "Status Register (0x504):"
    sudo fpga-read-register -S $SLOT -o 0x504

    # Read response count (offset 0x508)
    echo "Response Count (0x508):"
    sudo fpga-read-register -S $SLOT -o 0x508

    echo ""
    echo "=== Writing Test Command ==="

    # Write a test command
    # Command format: 70-bit command packed into registers
    # For now, write a simple test pattern

    # Write command low (offset 0x00)
    sudo fpga-write-register -S $SLOT -o 0x00 -v 0x12345678

    # Write command high (offset 0x04)
    sudo fpga-write-register -S $SLOT -o 0x04 -v 0x9ABCDEF0

    # Trigger command (write to control register at 0x10)
    sudo fpga-write-register -S $SLOT -o 0x10 -v 0x1

    # Wait for processing
    sleep 1

    echo ""
    echo "=== Reading Response ==="

    # Read response low (offset 0x100)
    echo "Response[31:0]:"
    sudo fpga-read-register -S $SLOT -o 0x100

    # Read response high (offset 0x104)
    echo "Response[63:32]:"
    sudo fpga-read-register -S $SLOT -o 0x104

else
    echo "fpga-read-register not found. Install AWS FPGA SDK tools."
    echo "Or use devmem2 to access OCL BAR directly."
fi

echo ""
echo "=== Test Complete ☧ ==="
echo ""
echo "To run miniKanren queries, use the Rust driver:"
echo "  cd rust_chirho && cargo run --example f1_driver_chirho"
