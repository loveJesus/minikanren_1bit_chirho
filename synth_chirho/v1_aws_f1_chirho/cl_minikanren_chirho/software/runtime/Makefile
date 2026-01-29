# ============================================================================
# ☧ miniKanren FPGA Runtime Makefile
# ============================================================================

CC = gcc
CFLAGS = -Wall -Wextra -O2 -g
LDFLAGS = -lfpga_mgmt -lrt

# AWS FPGA SDK paths
SDK_DIR ?= $(AWS_FPGA_REPO_DIR)/sdk
INCLUDES = -I$(SDK_DIR)/userspace/include

TARGETS = test_minikanren_chirho

all: $(TARGETS)

test_minikanren_chirho: test_minikanren_chirho.c
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ $< $(LDFLAGS)

clean:
	rm -f $(TARGETS) *.o

.PHONY: all clean
