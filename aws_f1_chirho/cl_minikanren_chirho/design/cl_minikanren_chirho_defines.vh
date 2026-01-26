// ============================================================================
// ☧ For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// AWS F1 Custom Logic Defines for miniKanren Search Engine
//
// This file defines which AXI interfaces are used by our design.
// Unused interfaces will be tied off automatically.
// ============================================================================

`ifndef CL_MINIKANREN_CHIRHO_DEFINES
`define CL_MINIKANREN_CHIRHO_DEFINES

// ============================================================================
// AXI Interface Enables
// ============================================================================
// We only use the OCL (AXI-Lite) interface for host register access.
// All other interfaces are unused and will be tied off.

// OCL: AXI-Lite for register access (ENABLED)
`define USE_AXIL_OCL_CHIRHO

// USR: Additional AXI-Lite (NOT USED)
// `define USE_AXIL_USR_CHIRHO

// SDA: Another AXI-Lite interface (NOT USED)
// `define USE_AXIL_SDA_CHIRHO

// DMA_PCIS: DMA from shell to CL (NOT USED)
// `define USE_DMA_PCIS_CHIRHO

// DDR4_SH: Shell DDR4 (NOT USED - we use on-chip BRAM)
// `define USE_DDR4_SH_CHIRHO

// DDR4_CL: CL DDR4 channels (NOT USED)
// `define USE_DDR4_CL_CHIRHO

// PCIM: PCIe master interface (NOT USED)
// `define USE_PCIM_CHIRHO

// ============================================================================
// Clock Configuration
// ============================================================================
// Using clock recipe A0 (250MHz main clock, divided down internally if needed)
// Our design runs at 50MHz, so we'll use a clock divider

`define CLOCK_RECIPE_A_CHIRHO A0
`define CLOCK_RECIPE_B_CHIRHO B0
`define CLOCK_RECIPE_C_CHIRHO C0

// ============================================================================
// Design Parameters
// ============================================================================
`define MINIKANREN_VERSION_CHIRHO 32'h0001_0000  // v1.0.0

`endif // CL_MINIKANREN_CHIRHO_DEFINES
