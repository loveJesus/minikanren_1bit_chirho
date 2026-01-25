//! Hardware-accelerated primitives ☧
//!
//! FPGA-friendly and SIMD-optimized data structures:
//! - BitVec64/BitVec256: Packed bit domains
//! - CAM: Content-addressable memory for parallel lookup
//! - SIMD: AVX2 bulk operations on domain arrays
//! - Optics: Hardware prisms/lenses over bit domains (3000× faster than heap)
//! - Goals: miniKanren goal combinators over bit-parallel states

pub mod bitmatrix_chirho;
pub mod bitmatrix_packed_chirho;
pub mod goals_chirho;
pub mod hardware_chirho;
pub mod optics_hw_chirho;
pub mod simd_chirho;

// Re-export key types
pub use bitmatrix_chirho::{BitMatrixChirho, BitTensor3Chirho};
pub use bitmatrix_packed_chirho::{BitMatrix64Chirho, BitMatrixPackedChirho, Word64Chirho};
pub use hardware_chirho::{
    BitVec256Chirho, BitVec64Chirho, CamHwChirho, SearchState256HwChirho, SearchStateHwChirho,
    UnifyUnitHwChirho,
};
pub use optics_hw_chirho::{
    collect_nonempty_chirho, traverse_all_chirho, DomainHwChirho, LensHwChirho,
    PartitionedDomainChirho, PrismHwChirho, StateHwChirho,
};
pub use simd_chirho::{
    bulk_and_chirho, bulk_not_chirho, bulk_or_chirho, bulk_popcount_chirho, bulk_xor_chirho,
    AlignedBitMatrixChirho,
};
pub use goals_chirho::{
    GoalHwChirho, conde_hw_chirho, conj_hw_chirho, disj_hw_chirho, eq_hw_chirho,
    fail_hw_chirho, run_hw_chirho, succeed_hw_chirho, unify_hw_chirho,
};
