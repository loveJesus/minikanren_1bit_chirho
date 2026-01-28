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
//    31:16: PCIe Device ID (0xF016 = Valid F0xx range + John 3:16)
//    15: 0: PCIe Vendor ID (0x1D0F = Amazon)
//
// IMPORTANT: For Amazon VID 0x1D0F, Device ID MUST be in range 0xF000-0xF0FF
// - 0x1042 is FORBIDDEN (reserved for F1)
// - 0xF200+ is FORBIDDEN (reserved by AWS shell)
// - We use 0xF016 = F0xx valid range + tribute to John 3:16 ☧
`define CL_SH_ID0       32'hF016_1D0F

// CL_SH_ID1
// - PCIe Subsystem/Subsystem Vendor ID Values
//    31:16: PCIe Subsystem ID (0xF016 = matching our device ID)
//    15: 0: PCIe Subsystem Vendor ID (0x1D51)
`define CL_SH_ID1       32'h1D51_F016
