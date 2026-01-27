// ============================================================================
// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// AWS F2 Shell Ports (Stub for standalone synthesis verification)
//
// This file provides port definitions for out-of-context synthesis.
// When integrating with the actual AWS F2 shell, replace with HDK version.
// ============================================================================

// HBM Reference Clock
input  logic         clk_hbm_ref,

// Clock and Reset
input  logic         clk_main_a0,
input  logic         clk_extra_a1,
input  logic         clk_extra_a2,
input  logic         clk_extra_a3,
input  logic         clk_extra_b0,
input  logic         clk_extra_b1,
input  logic         clk_extra_c0,
input  logic         clk_extra_c1,
input  logic         rst_main_n,
input  logic         sh_cl_pwr_state,

// Status/ID signals
output logic         cl_sh_flr_done,
output logic [31:0]  cl_sh_status0,
output logic [31:0]  cl_sh_status1,
output logic [31:0]  cl_sh_status2,
output logic [31:0]  cl_sh_id0,
output logic [31:0]  cl_sh_id1,
output logic         cl_sh_dma_wr_full,
output logic         cl_sh_dma_rd_full,

// PCIM AXI Master Interface (CL to Shell)
output logic [15:0]  cl_sh_pcim_awid,
output logic [63:0]  cl_sh_pcim_awaddr,
output logic [7:0]   cl_sh_pcim_awlen,
output logic [2:0]   cl_sh_pcim_awsize,
output logic [1:0]   cl_sh_pcim_awburst,
output logic [3:0]   cl_sh_pcim_awcache,
output logic         cl_sh_pcim_awlock,
output logic [2:0]   cl_sh_pcim_awprot,
output logic [3:0]   cl_sh_pcim_awqos,
output logic [54:0]  cl_sh_pcim_awuser,
output logic         cl_sh_pcim_awvalid,
input  logic         sh_cl_pcim_awready,

output logic [15:0]  cl_sh_pcim_wid,
output logic [511:0] cl_sh_pcim_wdata,
output logic [63:0]  cl_sh_pcim_wstrb,
output logic         cl_sh_pcim_wlast,
output logic [63:0]  cl_sh_pcim_wuser,
output logic         cl_sh_pcim_wvalid,
input  logic         sh_cl_pcim_wready,

input  logic [15:0]  sh_cl_pcim_bid,
input  logic [1:0]   sh_cl_pcim_bresp,
input  logic [17:0]  sh_cl_pcim_buser,
input  logic         sh_cl_pcim_bvalid,
output logic         cl_sh_pcim_bready,

output logic [15:0]  cl_sh_pcim_arid,
output logic [63:0]  cl_sh_pcim_araddr,
output logic [7:0]   cl_sh_pcim_arlen,
output logic [2:0]   cl_sh_pcim_arsize,
output logic [1:0]   cl_sh_pcim_arburst,
output logic [3:0]   cl_sh_pcim_arcache,
output logic         cl_sh_pcim_arlock,
output logic [2:0]   cl_sh_pcim_arprot,
output logic [3:0]   cl_sh_pcim_arqos,
output logic [54:0]  cl_sh_pcim_aruser,
output logic         cl_sh_pcim_arvalid,
input  logic         sh_cl_pcim_arready,

input  logic [15:0]  sh_cl_pcim_rid,
input  logic [511:0] sh_cl_pcim_rdata,
input  logic [1:0]   sh_cl_pcim_rresp,
input  logic         sh_cl_pcim_rlast,
input  logic [17:0]  sh_cl_pcim_ruser,
input  logic         sh_cl_pcim_rvalid,
output logic         cl_sh_pcim_rready,

// PCIS DMA Slave Interface (Shell to CL)
input  logic [5:0]   sh_cl_dma_pcis_awid,
input  logic [63:0]  sh_cl_dma_pcis_awaddr,
input  logic [7:0]   sh_cl_dma_pcis_awlen,
input  logic [2:0]   sh_cl_dma_pcis_awsize,
input  logic [1:0]   sh_cl_dma_pcis_awburst,
input  logic         sh_cl_dma_pcis_awvalid,
output logic         cl_sh_dma_pcis_awready,

input  logic [511:0] sh_cl_dma_pcis_wdata,
input  logic [63:0]  sh_cl_dma_pcis_wstrb,
input  logic         sh_cl_dma_pcis_wlast,
input  logic         sh_cl_dma_pcis_wvalid,
output logic         cl_sh_dma_pcis_wready,

output logic [5:0]   cl_sh_dma_pcis_bid,
output logic [1:0]   cl_sh_dma_pcis_bresp,
output logic         cl_sh_dma_pcis_bvalid,
input  logic         sh_cl_dma_pcis_bready,

input  logic [5:0]   sh_cl_dma_pcis_arid,
input  logic [63:0]  sh_cl_dma_pcis_araddr,
input  logic [7:0]   sh_cl_dma_pcis_arlen,
input  logic [2:0]   sh_cl_dma_pcis_arsize,
input  logic [1:0]   sh_cl_dma_pcis_arburst,
input  logic         sh_cl_dma_pcis_arvalid,
output logic         cl_sh_dma_pcis_arready,

output logic [5:0]   cl_sh_dma_pcis_rid,
output logic [511:0] cl_sh_dma_pcis_rdata,
output logic [1:0]   cl_sh_dma_pcis_rresp,
output logic         cl_sh_dma_pcis_rlast,
output logic         cl_sh_dma_pcis_ruser,
output logic         cl_sh_dma_pcis_rvalid,
input  logic         sh_cl_dma_pcis_rready,

// OCL (OpenCL) AXI-Lite Interface - F2 naming convention
// Inputs from shell to CL
input  logic         ocl_cl_awvalid,
input  logic [31:0]  ocl_cl_awaddr,
input  logic         ocl_cl_wvalid,
input  logic [31:0]  ocl_cl_wdata,
input  logic [3:0]   ocl_cl_wstrb,
input  logic         ocl_cl_bready,
input  logic         ocl_cl_arvalid,
input  logic [31:0]  ocl_cl_araddr,
input  logic         ocl_cl_rready,
// Outputs from CL to shell
output logic         cl_ocl_awready,
output logic         cl_ocl_wready,
output logic         cl_ocl_bvalid,
output logic [1:0]   cl_ocl_bresp,
output logic         cl_ocl_arready,
output logic         cl_ocl_rvalid,
output logic [31:0]  cl_ocl_rdata,
output logic [1:0]   cl_ocl_rresp,

// Status LED
output logic [15:0]  cl_sh_status_vled,

// SDA (Shell Debug Access) AXI-Lite Interface
input  logic         sda_cl_awvalid,
input  logic [31:0]  sda_cl_awaddr,
output logic         cl_sda_awready,

input  logic         sda_cl_wvalid,
input  logic [31:0]  sda_cl_wdata,
input  logic [3:0]   sda_cl_wstrb,
output logic         cl_sda_wready,

output logic         cl_sda_bvalid,
output logic [1:0]   cl_sda_bresp,
input  logic         sda_cl_bready,

input  logic         sda_cl_arvalid,
input  logic [31:0]  sda_cl_araddr,
output logic         cl_sda_arready,

output logic         cl_sda_rvalid,
output logic [31:0]  cl_sda_rdata,
output logic [1:0]   cl_sda_rresp,
input  logic         sda_cl_rready,

// DDR4 Interface (directly to DIMM)
output logic         CLK_DIMM_DP,
output logic         CLK_DIMM_DN,
output logic         M_ACT_N,
output logic [16:0]  M_MA,
output logic [1:0]   M_BA,
output logic [1:0]   M_BG,
output logic [1:0]   M_CKE,
output logic [1:0]   M_ODT,
output logic [1:0]   M_CS_N,
output logic         M_CLK_DN,
output logic         M_CLK_DP,
output logic         M_PAR,
inout  wire  [71:0]  M_DQ,
inout  wire  [8:0]   M_ECC,
inout  wire  [17:0]  M_DQS_DP,
inout  wire  [17:0]  M_DQS_DN,
output logic         RST_DIMM_N,

// HBM Interface (if EN_HBM=1)
input  logic         hbm_ref_clk,
output logic         HBM_CATTRIP,

// HBM AXI Interfaces (32 channels, using channels 0-7 for now)
// Channel 0
output logic [5:0]   cl_hbm_axi00_awid,
output logic [32:0]  cl_hbm_axi00_awaddr,
output logic [3:0]   cl_hbm_axi00_awlen,
output logic [2:0]   cl_hbm_axi00_awsize,
output logic [1:0]   cl_hbm_axi00_awburst,
output logic         cl_hbm_axi00_awlock,
output logic [3:0]   cl_hbm_axi00_awcache,
output logic [2:0]   cl_hbm_axi00_awprot,
output logic [3:0]   cl_hbm_axi00_awqos,
output logic [3:0]   cl_hbm_axi00_awregion,
output logic         cl_hbm_axi00_awvalid,
input  logic         hbm_cl_axi00_awready,
output logic [255:0] cl_hbm_axi00_wdata,
output logic [31:0]  cl_hbm_axi00_wstrb,
output logic         cl_hbm_axi00_wlast,
output logic         cl_hbm_axi00_wvalid,
input  logic         hbm_cl_axi00_wready,
input  logic [5:0]   hbm_cl_axi00_bid,
input  logic [1:0]   hbm_cl_axi00_bresp,
input  logic         hbm_cl_axi00_bvalid,
output logic         cl_hbm_axi00_bready,
output logic [5:0]   cl_hbm_axi00_arid,
output logic [32:0]  cl_hbm_axi00_araddr,
output logic [3:0]   cl_hbm_axi00_arlen,
output logic [2:0]   cl_hbm_axi00_arsize,
output logic [1:0]   cl_hbm_axi00_arburst,
output logic         cl_hbm_axi00_arlock,
output logic [3:0]   cl_hbm_axi00_arcache,
output logic [2:0]   cl_hbm_axi00_arprot,
output logic [3:0]   cl_hbm_axi00_arqos,
output logic [3:0]   cl_hbm_axi00_arregion,
output logic         cl_hbm_axi00_arvalid,
input  logic         hbm_cl_axi00_arready,
input  logic [5:0]   hbm_cl_axi00_rid,
input  logic [255:0] hbm_cl_axi00_rdata,
input  logic [1:0]   hbm_cl_axi00_rresp,
input  logic         hbm_cl_axi00_rlast,
input  logic         hbm_cl_axi00_rvalid,
output logic         cl_hbm_axi00_rready,

// HBM Ready signal
input  logic         hbm_ready,

// DDR Stat Interface
input  logic [7:0]   sh_ddr_stat_addr,
input  logic [31:0]  sh_ddr_stat_wdata,
input  logic         sh_ddr_stat_wr,
input  logic         sh_ddr_stat_rd,
output logic         cl_sh_ddr_stat_ack,
output logic [31:0]  cl_sh_ddr_stat_rdata,
output logic         cl_sh_ddr_stat_int,
input  logic         sh_cl_ddr_is_ready,

// Interrupt Interface
output logic [15:0]  cl_sh_apppf_irq_req,
input  logic [15:0]  sh_cl_apppf_irq_ack,

// Virtual JTAG
input  logic         tck,
input  logic         tms,
input  logic         tdi,
output logic         tdo,

// PCIe EP/RP Interfaces (unused but required)
output logic [15:0]  PCIE_EP_TXP,
output logic [15:0]  PCIE_EP_TXN,
input  logic [15:0]  PCIE_EP_RXP,
input  logic [15:0]  PCIE_EP_RXN,
output logic         PCIE_RP_PERSTN,
input  logic [15:0]  PCIE_RP_RXP,
input  logic [15:0]  PCIE_RP_RXN,
output logic [15:0]  PCIE_RP_TXP,
output logic [15:0]  PCIE_RP_TXN,

// HBM APB Interface 0
input  logic         hbm_apb_preset_n_0,
output logic [21:0]  hbm_apb_paddr_0,
output logic [2:0]   hbm_apb_pprot_0,
output logic         hbm_apb_psel_0,
output logic         hbm_apb_penable_0,
output logic         hbm_apb_pwrite_0,
output logic [31:0]  hbm_apb_pwdata_0,
output logic [3:0]   hbm_apb_pstrb_0,
output logic         hbm_apb_pready_0,
output logic [31:0]  hbm_apb_prdata_0,
output logic         hbm_apb_pslverr_0,

// HBM APB Interface 1
input  logic         hbm_apb_preset_n_1,
output logic [21:0]  hbm_apb_paddr_1,
output logic [2:0]   hbm_apb_pprot_1,
output logic         hbm_apb_psel_1,
output logic         hbm_apb_penable_1,
output logic         hbm_apb_pwrite_1,
output logic [31:0]  hbm_apb_pwdata_1,
output logic [3:0]   hbm_apb_pstrb_1,
output logic         hbm_apb_pready_1,
output logic [31:0]  hbm_apb_prdata_1,
output logic         hbm_apb_pslverr_1
