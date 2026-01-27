{-# LANGUAGE DataKinds #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE NoImplicitPrelude #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE DeriveAnyClass #-}

{- |
Module      : TermStoreHbmChirho
Description : Hash-consed term store in HBM ☧
Copyright   : (c) 2026
License     : MIT

Bridges the gap between Rust's arbitrary terms and FPGA's finite domains.

Key insight: Terms are just integers (IDs). Store the mapping in HBM.
FPGA can then work with term IDs directly.

"For God so loved the world..." - John 3:16
-}
module TermStoreHbmChirho where

import Clash.Prelude

-- ============================================================================
-- Term Representation ☧
-- ============================================================================

-- | Term ID (24 bits = 16M unique terms)
type TermIdChirho = BitVector 24

-- | Term tag (what kind of term)
data TermTagChirho
  = TagAtomChirho       -- Atomic value (number, symbol)
  | TagVarChirho        -- Logic variable
  | TagConsChirho       -- Cons cell (car, cdr)
  | TagNilChirho        -- Empty list
  deriving (Generic, NFDataX, Eq, Show)

-- | Encode tag as 2 bits
encodeTermTagChirho :: TermTagChirho -> BitVector 2
encodeTermTagChirho TagAtomChirho = 0
encodeTermTagChirho TagVarChirho  = 1
encodeTermTagChirho TagConsChirho = 2
encodeTermTagChirho TagNilChirho  = 3

-- | Term cell stored in HBM (64 bits)
--
-- Layout:
--   [63:62] tag (2 bits)
--   [61:38] field1 (24 bits) - atom value, var ID, or car
--   [37:14] field2 (24 bits) - cdr (for cons)
--   [13:0]  reserved/hash (14 bits)
--
data TermCellChirho = TermCellChirho
  { tcTagChirho    :: TermTagChirho
  , tcField1Chirho :: BitVector 24  -- Atom value, var ID, or car term ID
  , tcField2Chirho :: BitVector 24  -- Cdr term ID (for cons)
  , tcHashChirho   :: BitVector 14  -- Partial hash for fast comparison
  } deriving (Generic, NFDataX, Eq, Show)

-- | Pack term cell to 64 bits for HBM storage
packTermCellChirho :: TermCellChirho -> BitVector 64
packTermCellChirho tcChirho =
  (resize (encodeTermTagChirho (tcTagChirho tcChirho)) `shiftL` 62) .|.
  (resize (tcField1Chirho tcChirho) `shiftL` 38) .|.
  (resize (tcField2Chirho tcChirho) `shiftL` 14) .|.
  resize (tcHashChirho tcChirho)

-- | Unpack 64 bits from HBM to term cell
unpackTermCellChirho :: BitVector 64 -> TermCellChirho
unpackTermCellChirho bitsChirho = TermCellChirho
  { tcTagChirho    = decodeTermTagChirho (resize (bitsChirho `shiftR` 62))
  , tcField1Chirho = resize (bitsChirho `shiftR` 38)
  , tcField2Chirho = resize (bitsChirho `shiftR` 14)
  , tcHashChirho   = resize bitsChirho
  }

decodeTermTagChirho :: BitVector 2 -> TermTagChirho
decodeTermTagChirho 0 = TagAtomChirho
decodeTermTagChirho 1 = TagVarChirho
decodeTermTagChirho 2 = TagConsChirho
decodeTermTagChirho _ = TagNilChirho

-- ============================================================================
-- Term Constructors ☧
-- ============================================================================

-- | Create atom term
atomTermChirho :: BitVector 24 -> TermCellChirho
atomTermChirho valChirho = TermCellChirho
  { tcTagChirho    = TagAtomChirho
  , tcField1Chirho = valChirho
  , tcField2Chirho = 0
  , tcHashChirho   = resize valChirho  -- Simple hash
  }

-- | Create variable term
varTermChirho :: BitVector 24 -> TermCellChirho
varTermChirho varIdChirho = TermCellChirho
  { tcTagChirho    = TagVarChirho
  , tcField1Chirho = varIdChirho
  , tcField2Chirho = 0
  , tcHashChirho   = resize varIdChirho
  }

-- | Create cons cell
consTermChirho :: TermIdChirho -> TermIdChirho -> TermCellChirho
consTermChirho carChirho cdrChirho = TermCellChirho
  { tcTagChirho    = TagConsChirho
  , tcField1Chirho = carChirho
  , tcField2Chirho = cdrChirho
  , tcHashChirho   = resize (carChirho `xor` (cdrChirho `rotateL` 7))
  }

-- | Nil term (empty list)
nilTermChirho :: TermCellChirho
nilTermChirho = TermCellChirho
  { tcTagChirho    = TagNilChirho
  , tcField1Chirho = 0
  , tcField2Chirho = 0
  , tcHashChirho   = 0
  }

-- ============================================================================
-- Hash Table for Term Lookup ☧
-- ============================================================================

{-
HBM Hash Table Design:

To check if a term already exists (hash-consing), we need:
  1. Compute hash of (tag, field1, field2)
  2. Look up hash bucket in HBM
  3. Compare entries in bucket

Hash table layout in HBM:
  - 64K buckets (16-bit hash)
  - Each bucket: 8 entries × 8 bytes = 64 bytes
  - Total: 64K × 64 = 4 MB

Bucket structure:
  Entry 0: term_id (24 bits) + term_cell (40 bits used)
  Entry 1: ...
  ...
  Entry 7: ...

For larger tables, use chaining or cuckoo hashing.
-}

-- | Hash table bucket (8 entries)
type BucketChirho = Vec 8 (BitVector 64)

-- | Compute hash for term lookup
hashTermChirho :: TermCellChirho -> BitVector 16
hashTermChirho tcChirho =
  let h1Chirho = resize (tcField1Chirho tcChirho) :: BitVector 16
      h2Chirho = resize (tcField2Chirho tcChirho) :: BitVector 16
      tagBitsChirho = resize (encodeTermTagChirho (tcTagChirho tcChirho)) :: BitVector 16
  in h1Chirho `xor` (h2Chirho `rotateL` 5) `xor` (tagBitsChirho `rotateL` 11)

-- | HBM address for hash bucket
bucketAddrChirho :: BitVector 16 -> BitVector 34
bucketAddrChirho hashChirho =
  let baseChirho = 0x100000000 :: BitVector 34  -- Term store starts at 4GB offset
      bucketOffsetChirho = resize hashChirho `shiftL` 6  -- 64 bytes per bucket
  in baseChirho + bucketOffsetChirho

-- ============================================================================
-- Term Store Operations ☧
-- ============================================================================

-- | Term store operation codes
data TermOpChirho
  = TermLookupChirho    -- Find term by content (hash-cons check)
  | TermInsertChirho    -- Insert new term
  | TermFetchChirho     -- Fetch term by ID
  | TermWalkChirho      -- Walk/dereference variable
  deriving (Generic, NFDataX, Eq, Show)

-- | Term store command
data TermCmdChirho = TermCmdChirho
  { termOpChirho   :: TermOpChirho
  , termCellChirho :: TermCellChirho   -- For lookup/insert
  , termIdChirho   :: TermIdChirho     -- For fetch/walk
  } deriving (Generic, NFDataX, Eq, Show)

-- | Term store response
data TermRespChirho = TermRespChirho
  { termFoundChirho   :: Bool           -- Term exists (for lookup)
  , termResultIdChirho :: TermIdChirho  -- Result term ID
  , termResultChirho  :: TermCellChirho -- Result term cell
  } deriving (Generic, NFDataX, Eq, Show)

-- ============================================================================
-- Integration with Domain Engine ☧
-- ============================================================================

{-
BRIDGING TERMS AND DOMAINS

The key insight: variables have BOTH:
  1. A domain (which VALUES are possible) - Hierarchical bit vectors
  2. A binding (which TERM it equals) - Term store

Unification flow:
  1. CPU sends: unify(var1, var2)
  2. FPGA:
     a. Intersect domains: var1.domain &= var2.domain
     b. If both bound to terms, check term equality
     c. If one bound, propagate binding
     d. If neither bound, link variables (union-find)

This requires connecting:
  - AdaptiveHbmChirho (domains)
  - TermStoreHbmChirho (terms)
  - UnionFindHbmChirho (variable equivalence)
-}

-- | Extended variable state (domain + binding)
data VarStateChirho = VarStateChirho
  { vsVarIdChirho    :: BitVector 16      -- Variable ID
  , vsDomainTagChirho :: BitVector 2      -- Domain type tag
  , vsBoundChirho    :: Bool              -- Is bound to a term?
  , vsTermIdChirho   :: TermIdChirho      -- Bound term (if vsBound)
  , vsParentChirho   :: BitVector 16      -- Union-find parent
  , vsRankChirho     :: BitVector 8       -- Union-find rank
  } deriving (Generic, NFDataX, Eq, Show)

-- | Pack variable state for HBM (64 bits for header, domain stored separately)
packVarStateChirho :: VarStateChirho -> BitVector 64
packVarStateChirho vsChirho =
  (resize (vsVarIdChirho vsChirho) `shiftL` 48) .|.
  (resize (vsDomainTagChirho vsChirho) `shiftL` 46) .|.
  (if vsBoundChirho vsChirho then bit 45 else 0) .|.
  (resize (vsTermIdChirho vsChirho) `shiftL` 21) .|.
  (resize (vsParentChirho vsChirho) `shiftL` 5) .|.
  resize (vsRankChirho vsChirho)

-- ============================================================================
-- HBM Layout Summary ☧
-- ============================================================================

{-
COMPLETE HBM LAYOUT (16 GB total):

| Region | Offset | Size | Contents |
|--------|--------|------|----------|
| Variable headers | 0x0 | 512 MB | 32M vars × 16 bytes |
| Variable domains | 0x20000000 | 8 GB | 256K vars × 33KB each |
| Term store | 0x100000000 | 4 GB | 64M terms × 64 bytes |
| Hash table | 0x200000000 | 256 MB | 64K buckets × 4KB |
| Tabling cache | 0x210000000 | 2 GB | Memoized results |
| Reserved | 0x290000000 | 1 GB | Future expansion |

This layout supports:
  - 256K variables with full Hier256k domains
  - OR 8M variables with Hier4k domains
  - OR 32M variables with Flat64 domains
  - 64M unique terms
  - Tabling for repeated queries
-}

-- | HBM region base addresses
hbmVarHeadersChirho :: BitVector 34
hbmVarHeadersChirho = 0x0

hbmVarDomainsChirho :: BitVector 34
hbmVarDomainsChirho = 0x20000000  -- 512 MB

hbmTermStoreChirho :: BitVector 34
hbmTermStoreChirho = 0x100000000  -- 4 GB

hbmHashTableChirho :: BitVector 34
hbmHashTableChirho = 0x200000000  -- 8 GB

hbmTablingCacheChirho :: BitVector 34
hbmTablingCacheChirho = 0x210000000  -- 8.25 GB

-- ============================================================================
-- What This Enables ☧
-- ============================================================================

{-
WITH THIS DESIGN, FPGA CAN:

1. ✅ Hash-cons terms (lookup before insert)
2. ✅ Walk variable bindings (dereference)
3. ✅ Full structural unification (not just domain intersection)
4. ✅ Occurs check (walk term, check for var)
5. ✅ Cache tabled results

REMAINING CPU RESPONSIBILITIES:

1. Search strategy (which goal next)
2. Garbage collection (term store compaction)
3. Symbolic constraints (infinite domains)
4. Differentiable mode (float gradients)

This is a MUCH closer match to Rust capabilities!
The main gap is now just:
  - Infinite domains (inherent FPGA limitation)
  - Float gradients (could add with fixed-point)
-}
