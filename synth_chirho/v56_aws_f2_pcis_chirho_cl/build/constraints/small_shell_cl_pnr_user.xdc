# ============================================================================
# For God so loved the world - John 3:16
# Place and Route constraints for cl_minikanren_chirho ☧
# V5.2: Minimal floorplanning - NO custom pblocks due to HBM conflict
# ============================================================================

# VU47P has 3 Super Logic Regions (SLRs):
#   SLR0: HBM interface - HBM IP has FIXED physical placement
#   SLR1: Overflow / future expansion
#   SLR2: Reserved

# ============================================================================
# IMPORTANT: V5 learned that custom pblocks conflict with HBM IP placement.
# HBM IP must be placed at BLI_HBM_APB_INTF sites at chip edges, which are
# outside any user-defined pblock. V5.2 removes custom pblocks entirely.
#
# The AWS HDK shell already defines pblock_CL for the CL region.
# Adding our own pblocks on top causes placement failures.
# ============================================================================

# No custom pblocks - let Vivado auto-place with HBM proximity
# Our design is small enough (<8% of SLR0) that it will naturally
# be placed near HBM without explicit floorplanning

# ============================================================================
# SLR Resource Estimation (VU47P per SLR)
# ============================================================================
# Each SLR has approximately:
#   - ~300K LUTs
#   - ~600K FFs
#   - ~600 BRAMs (36Kb each = ~2.7MB total)
#   - ~1500 DSPs
#
# Our V5 SLR0 usage estimate (with 262K enabled):
#   - 65K buffers: 3 vars × 8KB = 24KB = ~6 BRAMs
#   - 262K buffers: 3 vars × 33KB = 99KB = ~24 BRAMs
#   - Flat buffers: 3 vars × 32B = negligible
#   - Neurosymbolic: ~50 DSPs, ~10K LUTs
#   - Control FSM: ~5K LUTs
#   - Total: ~30 BRAMs, ~15K LUTs = <8% of SLR0 capacity

# ============================================================================
# Physical Optimization Directives
# ============================================================================

set_property STEPS.PHYS_OPT_DESIGN.ARGS.DIRECTIVE AggressiveExplore [get_runs impl_1]
set_property STEPS.ROUTE_DESIGN.ARGS.DIRECTIVE AggressiveExplore [get_runs impl_1]

# Soli Deo Gloria ☧
