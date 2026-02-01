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
//    31:16: PCIe Device ID (0xF056 = Valid F0xx range, V5.6)
//    15: 0: PCIe Vendor ID (0x1D0F = Amazon)
//
// IMPORTANT: For Amazon VID 0x1D0F, Device ID MUST be in range 0xF000-0xF0FF
// - 0x1042 is FORBIDDEN (reserved for F1)
// - 0xF200+ is FORBIDDEN (reserved by AWS shell)
// - V5.3: 0xF053, V5.4: 0xF054, V5.5: 0xF055, V5.6: 0xF056, V5.7: 0xF057
`define CL_SH_ID0       32'hF057_1D0F

// CL_SH_ID1
// - PCIe Subsystem/Subsystem Vendor ID Values
//    31:16: PCIe Subsystem ID (0xF057 = matching our device ID)
//    15: 0: PCIe Subsystem Vendor ID (0x1D51)
`define CL_SH_ID1       32'h1D51_F057
