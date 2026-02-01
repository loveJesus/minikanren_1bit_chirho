# V5.6 Implementation: PCIS-to-HBM Connectivity ☧

*For God so loved the world - John 3:16*

## Summary

V5.6 fixes the critical HBM connectivity bug discovered on 2026-01-31. HBM has never worked in any previous version (V2-V5.5) because PCIS (BAR4 DMA) was tied off.

## Files Created

| File | Lines | Purpose |
|------|-------|---------|
| `cl_pcis_handler_chirho.sv` | 360 | Routes BAR4 (PCIS) traffic to HBM |
| `cl_axi_arbiter_chirho.sv` | 349 | Arbitrates PCIS vs FSM for HBM access |
| `test_hbm_dma_chirho.c` | ~200 | HBM DMA verification test |

## Files Modified

| File | Changes |
|------|---------|
| `cl_minikanren_chirho.sv` | Removed PCIS tie-offs, added arbiter signals, instantiated PCIS handler and arbiter |
| `cl_minikanren_chirho_defines.vh` | Updated VERSION to 0xF2560001, added debug register defines |
| `cl_id_defines.vh` | Updated Device ID to 0xF056 |
| `prepare_hdk_build_chirho.sh` | Added new files to copy list |

## Architecture

```
Host BAR0 (OCL) ──► Registers ──► FSM ──┐
                                        │
                              ┌─────────▼─────────┐
                              │  AXI Arbiter      │
                              │  (PCIS priority)  │
                              └─────────┬─────────┘
                                        │
Host BAR4 (PCIS) ──► PCIS Handler ──────┘
                                        │
                              ┌─────────▼─────────┐
                              │   cl_hbm_axi4     │
                              │  (existing)       │
                              └─────────┬─────────┘
                                        │
                                   HBM (16GB)
```

## Key Changes

### 1. PCIS Handler (`cl_pcis_handler_chirho.sv`)

- Accepts 512-bit PCIS traffic from shell
- Translates addresses (subtracts BAR4 base)
- Splits 512-bit to 256-bit for HBM
- Tracks write/read counts for debug

### 2. AXI Arbiter (`cl_axi_arbiter_chirho.sv`)

- PCIS has priority over FSM
- Simple state machine (not full crossbar)
- Minimal resource overhead (~200 LUTs)
- Switches only at transaction boundaries

### 3. FSM AXI Interface Change

Old (V5.5):
```systemverilog
// FSM drove hbm_axi4_bus_chirho directly
assign hbm_axi4_bus_chirho.arvalid = axi_read_req_chirho;

// Bug: Transitioned to LOAD state without waiting for arready
FSM_IDLE: begin
    axi_read_req_chirho <= 1'b1;
    fsm_state <= FSM_LOAD_VAR1;  // Assumes arready=1!
end
```

New (V5.6):
```systemverilog
// FSM drives fsm_hbm_*_chirho signals → arbiter → hbm_axi4_bus_chirho
assign fsm_hbm_arvalid_chirho = axi_read_req_chirho;

// Fix: Wait for arready before expecting data
FSM_IDLE: begin
    axi_read_req_chirho <= 1'b1;
    fsm_state <= FSM_WAIT_ARREADY;  // New state
end

FSM_WAIT_ARREADY: begin
    if (fsm_hbm_arvalid_chirho && fsm_hbm_arready_chirho) begin
        // Address accepted, now transition to data state
        fsm_state <= FSM_LOAD_VAR1;
    end
    // Keep arvalid=1 until accepted
end
```

### 4. New Debug Registers

| Address | Name | Purpose |
|---------|------|---------|
| 0xE0 | ARBITER | `{grant_fsm, grant_pcis}` |
| 0xE4 | PCIS_ADDR_LO | PCIS address [31:0] |
| 0xE8 | PCIS_ADDR_HI | PCIS address [33:32] |
| 0xEC | PCIS_ACTIVE | PCIS transaction in progress |
| 0xF0 | PCIS_WRITE_CNT | PCIS write count |
| 0xF4 | PCIS_READ_CNT | PCIS read count |

## Version Identifiers

| Register | Value | Meaning |
|----------|-------|---------|
| VERSION (0x00) | 0xF2560001 | F2 platform, V5.6, revision 1 |
| CL_SH_ID0 | 0xF0561D0F | Device=0xF056, Vendor=0x1D0F (Amazon) |
| CL_SH_ID1 | 0x1D51F056 | SubsysVID=0x1D51, SubsysID=0xF056 |

## Build Instructions

```bash
# On AWS F2 build instance
source $AWS_FPGA_REPO_DIR/hdk_setup.sh
cd /path/to/v56_aws_f2_pcis_chirho_cl
./prepare_hdk_build_chirho.sh
./launch_hdk_build_chirho.sh
```

## Verification Test

After AFI is loaded:

```bash
# Build test
gcc -I$SDK_DIR/userspace/include -L$SDK_DIR/userspace/lib \
    -o test_hbm_dma_chirho benchmarks_chirho/test_hbm_dma_chirho.c \
    -lfpga_mgmt -lrt -lpthread

# Run test
sudo ./test_hbm_dma_chirho
```

Expected output:
```
SUCCESS: HBM DMA VERIFIED! ☧
All 4096 bytes match.
```

## Success Criteria

1. `fpga_dma_burst_write()` returns 0 (success)
2. `fpga_dma_burst_read()` returns matching data
3. PCIS_WRITE_CNT > 0 after DMA write
4. PCIS_READ_CNT > 0 after DMA read
5. No AXI errors (bresp/rresp = 0)

## Resource Estimate

| Component | LUTs | FFs |
|-----------|------|-----|
| V5.5 baseline | ~15K | ~3K |
| PCIS handler | +800 | +600 |
| AXI arbiter | +200 | +100 |
| **V5.6 total** | ~16K | ~3.7K |

SLR0 capacity: ~300K LUTs - plenty of margin.

---

*Soli Deo Gloria* ☧
