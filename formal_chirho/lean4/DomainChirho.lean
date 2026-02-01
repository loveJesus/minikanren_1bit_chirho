/-
  For God so loved the world, that He gave His only begotten Son,
  that whosoever believeth in Him should not perish, but have everlasting life.
  - John 3:16

  Domain Chirho - Bit Vector Domains for miniKanren ☧

  This file defines finite domains as bit vectors and proves
  that domain intersection (AND) correctly computes set intersection.
-/

import Mathlib.Data.Finset.Basic
import Mathlib.Data.Nat.Bitwise
import Mathlib.Tactic

namespace MiniKanrenChirho

/-! ## Domain Definition -/

/-- A domain of size n represented as an n-bit vector.
    Each bit indicates whether that value is in the domain. -/
structure DomainChirho (n : ℕ) where
  bits : ℕ
  bound_chirho : bits < 2^n
  deriving Repr

/-- The empty domain (no values possible) -/
def emptyDomainChirho (n : ℕ) : DomainChirho n :=
  ⟨0, Nat.pow_pos (by decide : 0 < 2)⟩

/-- The full domain (all values possible) -/
def fullDomainChirho (n : ℕ) : DomainChirho n :=
  ⟨2^n - 1, by
    have h : 0 < 2^n := Nat.pow_pos (by decide : 0 < 2)
    omega⟩

/-- Membership: is value v in domain d? -/
def memberChirho {n : ℕ} (v : Fin n) (d : DomainChirho n) : Bool :=
  d.bits.testBit v.val

/-- Domain intersection (AND) -/
def intersectChirho {n : ℕ} (d1 d2 : DomainChirho n) : DomainChirho n :=
  ⟨d1.bits &&& d2.bits, by
    have h1 := d1.bound_chirho
    calc d1.bits &&& d2.bits ≤ d1.bits := Nat.and_le_left
      _ < 2^n := h1⟩

/-- Domain union (OR) -/
def unionChirho {n : ℕ} (d1 d2 : DomainChirho n) : DomainChirho n :=
  ⟨d1.bits ||| d2.bits, by
    have h1 := d1.bound_chirho
    have h2 := d2.bound_chirho
    exact Nat.or_lt_two_pow h1 h2⟩

/-- Is domain empty? -/
def isEmptyChirho {n : ℕ} (d : DomainChirho n) : Bool :=
  d.bits == 0

/-! ## Core Theorems -/

/-- Theorem 1: Intersection Soundness (UnifySoundChirho)
    If v is in the intersection, then v is in both domains -/
theorem intersect_sound_chirho {n : ℕ} (d1 d2 : DomainChirho n) (v : Fin n) :
    memberChirho v (intersectChirho d1 d2) = true →
    memberChirho v d1 = true ∧ memberChirho v d2 = true := by
  intro h
  simp only [memberChirho, intersectChirho] at h ⊢
  rw [Nat.testBit_and] at h
  rw [Bool.and_eq_true] at h
  exact h

/-- Theorem 2: Intersection Completeness (UnifyCompleteChirho)
    If v is in both domains, then v is in the intersection -/
theorem intersect_complete_chirho {n : ℕ} (d1 d2 : DomainChirho n) (v : Fin n) :
    memberChirho v d1 = true → memberChirho v d2 = true →
    memberChirho v (intersectChirho d1 d2) = true := by
  intro h1 h2
  simp only [memberChirho, intersectChirho]
  rw [Nat.testBit_and]
  rw [Bool.and_eq_true]
  exact ⟨h1, h2⟩

/-- Intersection is commutative -/
theorem intersect_comm_chirho {n : ℕ} (d1 d2 : DomainChirho n) :
    intersectChirho d1 d2 = intersectChirho d2 d1 := by
  simp only [intersectChirho, DomainChirho.mk.injEq]
  exact Nat.and_comm d1.bits d2.bits

/-- Intersection is associative -/
theorem intersect_assoc_chirho {n : ℕ} (d1 d2 d3 : DomainChirho n) :
    intersectChirho (intersectChirho d1 d2) d3 =
    intersectChirho d1 (intersectChirho d2 d3) := by
  simp only [intersectChirho, DomainChirho.mk.injEq]
  exact Nat.and_assoc d1.bits d2.bits d3.bits

/-- Union is commutative -/
theorem union_comm_chirho {n : ℕ} (d1 d2 : DomainChirho n) :
    unionChirho d1 d2 = unionChirho d2 d1 := by
  simp only [unionChirho, DomainChirho.mk.injEq]
  exact Nat.or_comm d1.bits d2.bits

/-- Union is associative -/
theorem union_assoc_chirho {n : ℕ} (d1 d2 d3 : DomainChirho n) :
    unionChirho (unionChirho d1 d2) d3 =
    unionChirho d1 (unionChirho d2 d3) := by
  simp only [unionChirho, DomainChirho.mk.injEq]
  exact Nat.or_assoc d1.bits d2.bits d3.bits

/-- Intersection distributes over union -/
theorem intersect_distrib_union_chirho {n : ℕ} (d1 d2 d3 : DomainChirho n) :
    intersectChirho d1 (unionChirho d2 d3) =
    unionChirho (intersectChirho d1 d2) (intersectChirho d1 d3) := by
  simp only [intersectChirho, unionChirho, DomainChirho.mk.injEq]
  exact Nat.and_or_distrib_left d1.bits d2.bits d3.bits

/-- Main bridge theorem: v in (d1 ∩ d2) iff v in d1 and v in d2 -/
theorem member_intersect_iff_chirho {n : ℕ} (d1 d2 : DomainChirho n) (v : Fin n) :
    memberChirho v (intersectChirho d1 d2) = true ↔
    (memberChirho v d1 = true ∧ memberChirho v d2 = true) := by
  constructor
  · exact intersect_sound_chirho d1 d2 v
  · intro ⟨h1, h2⟩
    exact intersect_complete_chirho d1 d2 v h1 h2

/-! ## Instances -/

instance {n : ℕ} : BEq (DomainChirho n) where
  beq d1 d2 := d1.bits == d2.bits

instance {n : ℕ} : Inhabited (DomainChirho n) where
  default := emptyDomainChirho n

instance {n : ℕ} : EmptyCollection (DomainChirho n) where
  emptyCollection := emptyDomainChirho n

/-! ## Singleton and Enumeration -/

/-- Create a singleton domain containing only value v -/
def singletonChirho {n : ℕ} (v : Fin n) : DomainChirho n :=
  ⟨1 <<< v.val, by
    have hv := v.isLt
    rw [Nat.one_shiftLeft]
    exact Nat.pow_lt_pow_right (by decide : 1 < 2) hv⟩

/-- Convert domain to list of values -/
def toListChirho {n : ℕ} (d : DomainChirho n) : List (Fin n) :=
  List.filter (fun v => memberChirho v d) (List.finRange n)

/-- Convert list of values to domain -/
def ofListChirho {n : ℕ} (vs : List (Fin n)) : DomainChirho n :=
  vs.foldl (fun d v => unionChirho d (singletonChirho v)) (emptyDomainChirho n)

end MiniKanrenChirho
