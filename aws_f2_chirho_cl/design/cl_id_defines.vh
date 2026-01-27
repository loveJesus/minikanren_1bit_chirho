// ============================================================================
// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// - John 3:16
// ============================================================================
//
// miniKanren F2 CL ID Defines ☧
//
// PCIe identification for the miniKanren custom logic.
// ============================================================================

// CL_SH_ID0
// - PCIe Vendor/Device ID Values
//    31:16: PCIe Device ID (0xF216 = F2 miniKanren v16)
//    15: 0: PCIe Vendor ID (0x1D0F = Amazon)
`define CL_SH_ID0       32'hF216_1D0F

// CL_SH_ID1
// - PCIe Subsystem/Subsystem Vendor ID Values
//    31:16: PCIe Subsystem ID (0x316 = John 3:16)
//    15: 0: PCIe Subsystem Vendor ID
`define CL_SH_ID1       32'h0316_1D51
