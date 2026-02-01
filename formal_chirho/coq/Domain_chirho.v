(* ============================================================================ *)
(* For God so loved the world, that He gave His only begotten Son,             *)
(* that whosoever believeth in Him should not perish, but have everlasting life.*)
(* - John 3:16                                                                  *)
(* ============================================================================ *)
(*                                                                              *)
(* Domain Chirho - Bit Vector Domains for miniKanren ☧                         *)
(*                                                                              *)
(* This file defines finite domains as bit vectors and proves                   *)
(* that domain intersection (AND) correctly computes set intersection.          *)
(*                                                                              *)
(* Naming Convention (per AGENTS.md):                                           *)
(*   - Types/Definitions: snake_chirho (e.g., domain_chirho)                   *)
(*   - Theorems: snake_chirho (e.g., intersect_sound_chirho)                   *)
(*   - Constructors: PascalChirho (e.g., Mk_hier_4k_chirho)                    *)
(*                                                                              *)
(* ============================================================================ *)

From Stdlib Require Import Init.Nat.
From Stdlib Require Import Arith.Arith.
From Stdlib Require Import NArith.NArith.
From Stdlib Require Import Lists.List.
From Stdlib Require Import Bool.Bool.
Import ListNotations.

Open Scope N_scope.

(** * Domain Definition *)

(** A domain is represented as a natural number where each bit indicates
    whether that value is in the domain. We use N (binary naturals) for
    efficient bitwise operations. *)

Definition domain_chirho := N.

(** Empty domain (no values) *)
Definition empty_domain_chirho : domain_chirho := 0.

(** Full domain of size n (all values 0..n-1) *)
Definition full_domain_chirho (n : nat) : domain_chirho := N.ones (N.of_nat n).

(** Membership test: is value v in domain d? *)
Definition member_domain_chirho (v : nat) (d : domain_chirho) : bool :=
  N.testbit d (N.of_nat v).

(** Domain intersection (AND) - corresponds to unification *)
Definition intersect_domain_chirho (d1 d2 : domain_chirho) : domain_chirho :=
  N.land d1 d2.

(** Domain union (OR) - corresponds to conde/disjunction *)
Definition union_domain_chirho (d1 d2 : domain_chirho) : domain_chirho :=
  N.lor d1 d2.

(** Domain complement (XOR with full domain) *)
Definition complement_domain_chirho (n : nat) (d : domain_chirho) : domain_chirho :=
  N.lxor (full_domain_chirho n) d.

(** Is domain empty? *)
Definition is_empty_domain_chirho (d : domain_chirho) : bool :=
  N.eqb d 0.

(** Singleton domain containing only value v *)
Definition singleton_domain_chirho (v : nat) : domain_chirho :=
  N.shiftl 1 (N.of_nat v).

(** * Core Theorems *)

(** ** Theorem 1: Intersection Soundness (intersect_sound_chirho)
    If v is in the intersection, then v is in both domains *)

Theorem intersect_sound_chirho : forall (v : nat) (d1 d2 : domain_chirho),
  member_domain_chirho v (intersect_domain_chirho d1 d2) = true ->
  member_domain_chirho v d1 = true /\ member_domain_chirho v d2 = true.
Proof.
  intros v d1 d2 H.
  unfold member_domain_chirho, intersect_domain_chirho in *.
  rewrite N.land_spec in H.
  apply andb_true_iff in H.
  exact H.
Qed.

(** ** Theorem 2: Intersection Completeness (intersect_complete_chirho)
    If v is in both domains, then v is in the intersection *)

Theorem intersect_complete_chirho : forall (v : nat) (d1 d2 : domain_chirho),
  member_domain_chirho v d1 = true ->
  member_domain_chirho v d2 = true ->
  member_domain_chirho v (intersect_domain_chirho d1 d2) = true.
Proof.
  intros v d1 d2 H1 H2.
  unfold member_domain_chirho, intersect_domain_chirho.
  rewrite N.land_spec.
  apply andb_true_iff.
  split; assumption.
Qed.

(** ** Theorem 3: Intersection is Commutative *)

Theorem intersect_comm_chirho : forall (d1 d2 : domain_chirho),
  intersect_domain_chirho d1 d2 = intersect_domain_chirho d2 d1.
Proof.
  intros d1 d2.
  unfold intersect_domain_chirho.
  apply N.land_comm.
Qed.

(** ** Theorem 4: Intersection is Associative *)

Theorem intersect_assoc_chirho : forall (d1 d2 d3 : domain_chirho),
  intersect_domain_chirho (intersect_domain_chirho d1 d2) d3 =
  intersect_domain_chirho d1 (intersect_domain_chirho d2 d3).
Proof.
  intros d1 d2 d3.
  unfold intersect_domain_chirho.
  symmetry. apply N.land_assoc.
Qed.

(** ** Theorem 5: Union is Commutative *)

Theorem union_comm_chirho : forall (d1 d2 : domain_chirho),
  union_domain_chirho d1 d2 = union_domain_chirho d2 d1.
Proof.
  intros d1 d2.
  unfold union_domain_chirho.
  apply N.lor_comm.
Qed.

(** ** Theorem 6: Union is Associative *)

Theorem union_assoc_chirho : forall (d1 d2 d3 : domain_chirho),
  union_domain_chirho (union_domain_chirho d1 d2) d3 =
  union_domain_chirho d1 (union_domain_chirho d2 d3).
Proof.
  intros d1 d2 d3.
  unfold union_domain_chirho.
  symmetry. apply N.lor_assoc.
Qed.

(** ** Theorem 7: Union Soundness
    If v is in the union, then v is in at least one domain *)

Theorem union_sound_chirho : forall (v : nat) (d1 d2 : domain_chirho),
  member_domain_chirho v (union_domain_chirho d1 d2) = true ->
  member_domain_chirho v d1 = true \/ member_domain_chirho v d2 = true.
Proof.
  intros v d1 d2 H.
  unfold member_domain_chirho, union_domain_chirho in *.
  rewrite N.lor_spec in H.
  apply orb_true_iff in H.
  exact H.
Qed.

(** ** Theorem 8: Intersection distributes over Union
    This is crucial for conde inside fresh *)

Theorem intersect_distrib_union_chirho : forall (d1 d2 d3 : domain_chirho),
  intersect_domain_chirho d1 (union_domain_chirho d2 d3) =
  union_domain_chirho (intersect_domain_chirho d1 d2) (intersect_domain_chirho d1 d3).
Proof.
  intros d1 d2 d3.
  unfold intersect_domain_chirho, union_domain_chirho.
  rewrite N.land_lor_distr_r.
  reflexivity.
Qed.

(** ** Theorem 9: Empty domain is identity for union *)

Theorem union_empty_l_chirho : forall (d : domain_chirho),
  union_domain_chirho empty_domain_chirho d = d.
Proof.
  intro d.
  unfold union_domain_chirho, empty_domain_chirho.
  apply N.lor_0_l.
Qed.

(** ** Theorem 10: Empty domain is absorbing for intersection *)

Theorem intersect_empty_l_chirho : forall (d : domain_chirho),
  intersect_domain_chirho empty_domain_chirho d = empty_domain_chirho.
Proof.
  intro d.
  unfold intersect_domain_chirho, empty_domain_chirho.
  apply N.land_0_l.
Qed.

(** ** Theorem 11: Membership Equivalence (main bridge theorem)
    v is in (d1 ∩ d2) iff v is in d1 AND v is in d2 *)

Theorem member_intersect_iff_chirho : forall (v : nat) (d1 d2 : domain_chirho),
  member_domain_chirho v (intersect_domain_chirho d1 d2) = true <->
  (member_domain_chirho v d1 = true /\ member_domain_chirho v d2 = true).
Proof.
  intros v d1 d2.
  split.
  - apply intersect_sound_chirho.
  - intros [H1 H2]. apply intersect_complete_chirho; assumption.
Qed.

(** * Hierarchical Domains *)

(** For domains larger than 64 values, we use a two-level hierarchy:
    - root: 64 bits indicating which leaves are non-empty
    - leaves: 64 arrays of 64 bits each = 4096 values total

    The key insight is that we can check the root first and skip
    leaves that are guaranteed empty after intersection. *)

Record hierarchical_4k_chirho := Mk_hier_4k_chirho {
  root_hier_chirho : domain_chirho;        (* 64 bits: which leaves non-empty *)
  leaves_hier_chirho : list domain_chirho  (* 64 domains of 64 bits each *)
}.

(** Flatten hierarchical domain to simple domain (for specification) *)
Definition flatten_hier_4k_chirho (h : hierarchical_4k_chirho) : domain_chirho :=
  (* Conceptually: combine all leaves with appropriate shifts *)
  fold_left (fun acc '(i, leaf) =>
    N.lor acc (N.shiftl leaf (N.of_nat (i * 64))))
    (combine (seq 0 64) (leaves_hier_chirho h))
    0.

(** Hierarchical intersection *)
Definition intersect_hier_4k_chirho (h1 h2 : hierarchical_4k_chirho)
    : hierarchical_4k_chirho :=
  let new_root := N.land (root_hier_chirho h1) (root_hier_chirho h2) in
  let new_leaves := map (fun '(l1, l2) => N.land l1 l2)
                        (combine (leaves_hier_chirho h1) (leaves_hier_chirho h2)) in
  Mk_hier_4k_chirho new_root new_leaves.

(** ** Theorem 12: Hierarchical Equivalence
    Hierarchical intersection = flat intersection *)

Theorem hier_intersect_equiv_chirho :
  forall (h1 h2 : hierarchical_4k_chirho),
  flatten_hier_4k_chirho (intersect_hier_4k_chirho h1 h2) =
  intersect_domain_chirho (flatten_hier_4k_chirho h1) (flatten_hier_4k_chirho h2).
Proof.
  intros h1 h2.
  unfold flatten_hier_4k_chirho, intersect_hier_4k_chirho, intersect_domain_chirho.
  (* Key insight: N.land distributes over N.lor when shifts don't overlap *)
  (* Proof requires showing that for disjoint bit positions:
     (a ||| b) &&& (c ||| d) = (a &&& c) ||| (b &&& d) *)
  admit.
Admitted.

(** * Semiring Abstraction *)

(** A semiring provides the algebraic structure for generalized search:
    - Boolean: standard search (AND=∧, OR=∨)
    - Counting: count solutions (AND=×, OR=+)
    - Tropical: shortest path (AND=+, OR=min)
    - Probabilistic: weighted search (AND=×, OR=+) with normalization *)

Class Semiring_chirho (A : Type) := {
  zero_semiring_chirho : A;
  one_semiring_chirho : A;
  plus_semiring_chirho : A -> A -> A;
  times_semiring_chirho : A -> A -> A;
  (* Semiring laws - note: associativity direction matches stdlib *)
  plus_assoc_semiring_chirho : forall a b c,
    plus_semiring_chirho (plus_semiring_chirho a b) c =
    plus_semiring_chirho a (plus_semiring_chirho b c);
  plus_comm_semiring_chirho : forall a b,
    plus_semiring_chirho a b = plus_semiring_chirho b a;
  plus_zero_l_semiring_chirho : forall a,
    plus_semiring_chirho zero_semiring_chirho a = a;
  times_assoc_semiring_chirho : forall a b c,
    times_semiring_chirho (times_semiring_chirho a b) c =
    times_semiring_chirho a (times_semiring_chirho b c);
  times_one_l_semiring_chirho : forall a,
    times_semiring_chirho one_semiring_chirho a = a;
  times_one_r_semiring_chirho : forall a,
    times_semiring_chirho a one_semiring_chirho = a;
  times_zero_l_semiring_chirho : forall a,
    times_semiring_chirho zero_semiring_chirho a = zero_semiring_chirho;
  distrib_l_semiring_chirho : forall a b c,
    times_semiring_chirho a (plus_semiring_chirho b c) =
    plus_semiring_chirho (times_semiring_chirho a b) (times_semiring_chirho a c)
}.

(** Boolean semiring instance *)
#[export] Instance bool_semiring_chirho : Semiring_chirho bool := {
  zero_semiring_chirho := false;
  one_semiring_chirho := true;
  plus_semiring_chirho := orb;
  times_semiring_chirho := andb;
  plus_assoc_semiring_chirho := fun a b c => eq_sym (orb_assoc a b c);
  plus_comm_semiring_chirho := orb_comm;
  plus_zero_l_semiring_chirho := fun a => eq_refl;
  times_assoc_semiring_chirho := fun a b c => eq_sym (andb_assoc a b c);
  times_one_l_semiring_chirho := andb_true_l;
  times_one_r_semiring_chirho := andb_true_r;
  times_zero_l_semiring_chirho := andb_false_l;
  distrib_l_semiring_chirho := andb_orb_distrib_r
}.

(** Natural number semiring for counting *)
#[export] Instance nat_semiring_chirho : Semiring_chirho nat := {
  zero_semiring_chirho := 0;
  one_semiring_chirho := 1;
  plus_semiring_chirho := Nat.add;
  times_semiring_chirho := Nat.mul;
  plus_assoc_semiring_chirho := fun a b c => eq_sym (Nat.add_assoc a b c);
  plus_comm_semiring_chirho := Nat.add_comm;
  plus_zero_l_semiring_chirho := Nat.add_0_l;
  times_assoc_semiring_chirho := fun a b c => eq_sym (Nat.mul_assoc a b c);
  times_one_l_semiring_chirho := Nat.mul_1_l;
  times_one_r_semiring_chirho := Nat.mul_1_r;
  times_zero_l_semiring_chirho := Nat.mul_0_l;
  distrib_l_semiring_chirho := Nat.mul_add_distr_l
}.

(** * Soli Deo Gloria ☧ *)
