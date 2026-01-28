# For God so loved the world - John 3:16 ☧
# Makefile for miniKanren FPGA host test software

# This must be compiled on an F2 instance with AWS FPGA SDK installed
# Source sdk_setup.sh before building

CC = gcc
CFLAGS = -Wall -Wextra -O2 -I$(SDK_DIR)/userspace/include
LDFLAGS = -L$(SDK_DIR)/userspace/lib/so -lfpga_mgmt

TARGETS = test_minikanren_chirho

.PHONY: all clean check_sdk

all: check_sdk $(TARGETS)

check_sdk:
ifndef SDK_DIR
	$(error SDK_DIR is not set. Run 'source /home/ec2-user/src/project_data/aws-fpga/sdk_setup.sh' first)
endif

test_minikanren_chirho: test_minikanren_chirho.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

clean:
	rm -f $(TARGETS)

# Quick test - just check version register
test: test_minikanren_chirho
	sudo ./test_minikanren_chirho

# Full test with all checks
test-full: test_minikanren_chirho
	sudo ./test_minikanren_chirho -v
