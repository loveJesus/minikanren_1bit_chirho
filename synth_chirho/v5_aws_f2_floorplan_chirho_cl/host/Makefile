# ============================================================================
# For God so loved the world - John 3:16 ☧
# Makefile for miniKanren FPGA host tests
# ============================================================================

# AWS FPGA SDK paths (set SDK_DIR if not using default)
SDK_DIR ?= $(HOME)/src/project_data/aws-fpga/sdk
INCLUDES = -I$(SDK_DIR)/userspace/include
LDFLAGS = -L$(SDK_DIR)/userspace/lib/so -Wl,-rpath,$(SDK_DIR)/userspace/lib/so
LDLIBS = -lfpga_mgmt -lrt -lpthread

CC = gcc
CFLAGS = -Wall -Wextra -O2 -g $(INCLUDES)

TARGETS = test_minikanren_chirho

.PHONY: all clean

all: $(TARGETS)

test_minikanren_chirho: test_minikanren_chirho.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS) $(LDLIBS)

clean:
	rm -f $(TARGETS)

# Quick test (requires AFI loaded)
test: test_minikanren_chirho
	sudo ./test_minikanren_chirho

# Benchmark mode
bench: test_minikanren_chirho
	sudo ./test_minikanren_chirho -b

# Soli Deo Gloria ☧
