#!/usr/bin/env python3
"""
Differentiable miniKanren ☧

Relaxes Boolean logic to probabilities, enabling gradient-based learning.

Key insight:
- Boolean AND → probability multiplication
- Boolean OR  → probability sum (clamped)
- Unification → soft matching with temperature

This connects miniKanren to:
- Scallop (differentiable Datalog)
- Neural-symbolic AI
- Probabilistic logic programming

The gradient flows through the logic!
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Set, Callable, Any
import math
import random


# === Soft Boolean Operations ===

def soft_and_chirho(a_chirho: float, b_chirho: float) -> float:
    """Soft AND: product of probabilities"""
    return a_chirho * b_chirho


def soft_or_chirho(a_chirho: float, b_chirho: float) -> float:
    """Soft OR: probabilistic sum (inclusion-exclusion)"""
    return a_chirho + b_chirho - a_chirho * b_chirho


def soft_not_chirho(a_chirho: float) -> float:
    """Soft NOT: complement"""
    return 1.0 - a_chirho


def soft_eq_chirho(a_chirho: float, b_chirho: float, temp_chirho: float = 1.0) -> float:
    """
    Soft equality with temperature.

    temp → 0: approaches hard equality (1 if equal, 0 otherwise)
    temp → ∞: everything is equally likely
    """
    diff_chirho = abs(a_chirho - b_chirho)
    return math.exp(-diff_chirho / temp_chirho)


# === Probabilistic Constraint ===

@dataclass
class ProbConstraintChirho:
    """
    A constraint with associated probability.

    Instead of "position 0 IS value 3" (Boolean),
    we have "position 0 is value 3 with probability 0.8"
    """
    position_chirho: int
    value_chirho: int
    prob_chirho: float  # In [0, 1]

    def __repr__(self_chirho):
        return f"P(pos{self_chirho.position_chirho}={self_chirho.value_chirho})={self_chirho.prob_chirho:.3f}"


# === Probabilistic Pattern ===

@dataclass
class ProbPatternChirho:
    """
    A pattern where each constraint has a probability.

    This is like a probability distribution over concrete patterns.
    """
    # Map from (position, value) to probability
    probs_chirho: Dict[Tuple[int, int], float] = field(default_factory=dict)
    length_chirho: int = 0

    def set_prob_chirho(self_chirho, pos_chirho: int, val_chirho: int, prob_chirho: float):
        """Set probability that position has value"""
        self_chirho.probs_chirho[(pos_chirho, val_chirho)] = prob_chirho
        self_chirho.length_chirho = max(self_chirho.length_chirho, pos_chirho + 1)

    def get_prob_chirho(self_chirho, pos_chirho: int, val_chirho: int) -> float:
        """Get probability (default 0.5 = unknown)"""
        return self_chirho.probs_chirho.get((pos_chirho, val_chirho), 0.5)

    def is_ground_chirho(self_chirho, threshold_chirho: float = 0.99) -> bool:
        """Check if all positions have high-confidence values"""
        for pos_chirho in range(self_chirho.length_chirho):
            max_prob_chirho = max(
                (self_chirho.get_prob_chirho(pos_chirho, v_chirho) for v_chirho in range(10)),
                default=0.0
            )
            if max_prob_chirho < threshold_chirho:
                return False
        return True

    def most_likely_chirho(self_chirho, max_val_chirho: int = 10) -> List[int]:
        """Get most likely concrete pattern"""
        result_chirho = []
        for pos_chirho in range(self_chirho.length_chirho):
            best_val_chirho = 0
            best_prob_chirho = 0.0
            for val_chirho in range(max_val_chirho):
                p_chirho = self_chirho.get_prob_chirho(pos_chirho, val_chirho)
                if p_chirho > best_prob_chirho:
                    best_prob_chirho = p_chirho
                    best_val_chirho = val_chirho
            result_chirho.append(best_val_chirho)
        return result_chirho

    def entropy_chirho(self_chirho, max_val_chirho: int = 10) -> float:
        """Compute entropy (uncertainty) of pattern"""
        total_chirho = 0.0
        for pos_chirho in range(self_chirho.length_chirho):
            for val_chirho in range(max_val_chirho):
                p_chirho = self_chirho.get_prob_chirho(pos_chirho, val_chirho)
                if 0 < p_chirho < 1:
                    total_chirho -= p_chirho * math.log(p_chirho)
        return total_chirho

    @classmethod
    def from_list_chirho(cls_chirho, lst_chirho: List[int], confidence_chirho: float = 0.99) -> 'ProbPatternChirho':
        """Create high-confidence pattern from concrete list"""
        p_chirho = ProbPatternChirho()
        p_chirho.length_chirho = len(lst_chirho)
        for pos_chirho, val_chirho in enumerate(lst_chirho):
            p_chirho.set_prob_chirho(pos_chirho, val_chirho, confidence_chirho)
        return p_chirho

    @classmethod
    def uniform_chirho(cls_chirho, length_chirho: int, max_val_chirho: int = 10) -> 'ProbPatternChirho':
        """Create uniform distribution (maximum uncertainty)"""
        p_chirho = ProbPatternChirho()
        p_chirho.length_chirho = length_chirho
        uniform_prob_chirho = 1.0 / max_val_chirho
        for pos_chirho in range(length_chirho):
            for val_chirho in range(max_val_chirho):
                p_chirho.set_prob_chirho(pos_chirho, val_chirho, uniform_prob_chirho)
        return p_chirho

    def __repr__(self_chirho):
        if self_chirho.is_ground_chirho():
            return f"Prob{self_chirho.most_likely_chirho()}"
        return f"ProbPattern(len={self_chirho.length_chirho}, entropy={self_chirho.entropy_chirho():.2f})"


# === Soft Unification ===

def soft_unify_chirho(p1_chirho: ProbPatternChirho,
                      p2_chirho: ProbPatternChirho,
                      temp_chirho: float = 0.1) -> Tuple[ProbPatternChirho, float]:
    """
    Soft unification of two probabilistic patterns.

    Returns:
    - Unified pattern (posterior distribution)
    - Overall match probability (in [0, 1])

    This is Bayesian inference!
    P(unified | p1, p2) ∝ P(p1 | unified) * P(p2 | unified) * P(unified)
    """
    result_chirho = ProbPatternChirho()
    result_chirho.length_chirho = max(p1_chirho.length_chirho, p2_chirho.length_chirho)

    total_prob_chirho = 1.0

    for pos_chirho in range(result_chirho.length_chirho):
        pos_prob_chirho = 0.0
        max_joint_chirho = 0.0

        for val_chirho in range(10):  # Assume max value 10
            # Probability both patterns agree on this value
            prob1_chirho = p1_chirho.get_prob_chirho(pos_chirho, val_chirho)
            prob2_chirho = p2_chirho.get_prob_chirho(pos_chirho, val_chirho)

            # Soft AND: both must have this value
            joint_chirho = soft_and_chirho(prob1_chirho, prob2_chirho)

            result_chirho.set_prob_chirho(pos_chirho, val_chirho, joint_chirho)
            pos_prob_chirho += joint_chirho
            max_joint_chirho = max(max_joint_chirho, joint_chirho)

        # Normalize probabilities at this position
        if pos_prob_chirho > 0:
            for val_chirho in range(10):
                old_chirho = result_chirho.get_prob_chirho(pos_chirho, val_chirho)
                result_chirho.set_prob_chirho(pos_chirho, val_chirho, old_chirho / pos_prob_chirho)

        # Use maximum joint probability at each position for total match score
        # This keeps the probability in [0, 1]
        total_prob_chirho *= max(max_joint_chirho, 1e-10)

    return result_chirho, total_prob_chirho


# === Probabilistic Relation ===

@dataclass
class ProbRelationChirho:
    """
    A relation where each tuple has a probability.

    This is like a weighted database.
    """
    name_chirho: str
    tuples_chirho: List[Tuple[ProbPatternChirho, ...]] = field(default_factory=list)
    weights_chirho: List[float] = field(default_factory=list)

    def add_chirho(self_chirho, *patterns_chirho: ProbPatternChirho, weight_chirho: float = 1.0):
        self_chirho.tuples_chirho.append(patterns_chirho)
        self_chirho.weights_chirho.append(weight_chirho)

    def query_chirho(self_chirho, *query_chirho: ProbPatternChirho) -> List[Tuple[Tuple[ProbPatternChirho, ...], float]]:
        """
        Query with soft matching.
        Returns list of (unified_tuple, probability).
        """
        results_chirho = []

        for tup_chirho, weight_chirho in zip(self_chirho.tuples_chirho, self_chirho.weights_chirho):
            unified_chirho = []
            total_prob_chirho = weight_chirho

            for q_chirho, t_chirho in zip(query_chirho, tup_chirho):
                u_chirho, prob_chirho = soft_unify_chirho(q_chirho, t_chirho)
                unified_chirho.append(u_chirho)
                total_prob_chirho *= prob_chirho

            if total_prob_chirho > 1e-6:  # Threshold
                results_chirho.append((tuple(unified_chirho), total_prob_chirho))

        # Sort by probability (descending)
        results_chirho.sort(key=lambda x_chirho: -x_chirho[1])
        return results_chirho


# === Gradient Computation (Manual) ===

def compute_gradient_chirho(relation_chirho: ProbRelationChirho,
                            query_chirho: Tuple[ProbPatternChirho, ...],
                            target_prob_chirho: float) -> Dict[Tuple[int, int, int], float]:
    """
    Compute gradient of loss w.r.t. relation probabilities.

    Loss = (actual_prob - target_prob)^2

    Returns: dict mapping (tuple_idx, pos, val) to gradient
    """
    results_chirho = relation_chirho.query_chirho(*query_chirho)
    actual_prob_chirho = sum(prob_chirho for _, prob_chirho in results_chirho)

    loss_chirho = (actual_prob_chirho - target_prob_chirho) ** 2
    dloss_dprob_chirho = 2 * (actual_prob_chirho - target_prob_chirho)

    gradients_chirho: Dict[Tuple[int, int, int], float] = {}

    # For each tuple that matched
    for tup_idx_chirho, (tup_chirho, weight_chirho) in enumerate(zip(relation_chirho.tuples_chirho, relation_chirho.weights_chirho)):
        for pat_idx_chirho, pattern_chirho in enumerate(tup_chirho):
            for (pos_chirho, val_chirho), prob_chirho in pattern_chirho.probs_chirho.items():
                # Simplified gradient (would need proper chain rule in practice)
                key_chirho = (tup_idx_chirho, pos_chirho, val_chirho)
                gradients_chirho[key_chirho] = dloss_dprob_chirho * prob_chirho * 0.01

    return gradients_chirho


# === Soft Appendo ===

class SoftAppendoChirho:
    """
    Soft/probabilistic version of appendo.

    Instead of Boolean success/failure, returns probability of success.
    """

    def __init__(self_chirho):
        self_chirho.cache_chirho = ProbRelationChirho("appendo")

    def call_chirho(self_chirho,
                    l_chirho: ProbPatternChirho,
                    s_chirho: ProbPatternChirho,
                    out_chirho: ProbPatternChirho) -> List[Tuple[ProbPatternChirho, ProbPatternChirho, ProbPatternChirho, float]]:
        """
        Soft appendo: returns list of (l, s, out, probability) results.
        """
        results_chirho = []

        # If l and s are ground, compute forward
        if l_chirho.is_ground_chirho() and s_chirho.is_ground_chirho():
            l_list_chirho = l_chirho.most_likely_chirho()
            s_list_chirho = s_chirho.most_likely_chirho()
            out_list_chirho = l_list_chirho + s_list_chirho

            out_result_chirho = ProbPatternChirho.from_list_chirho(out_list_chirho)
            _, prob_chirho = soft_unify_chirho(out_chirho, out_result_chirho)

            results_chirho.append((l_chirho, s_chirho, out_result_chirho, prob_chirho))

        # If out is ground, enumerate splits (backward)
        elif out_chirho.is_ground_chirho():
            out_list_chirho = out_chirho.most_likely_chirho()

            for i_chirho in range(len(out_list_chirho) + 1):
                l_list_chirho = out_list_chirho[:i_chirho]
                s_list_chirho = out_list_chirho[i_chirho:]

                l_result_chirho = ProbPatternChirho.from_list_chirho(l_list_chirho) if l_list_chirho else ProbPatternChirho()
                s_result_chirho = ProbPatternChirho.from_list_chirho(s_list_chirho) if s_list_chirho else ProbPatternChirho()

                _, prob_l_chirho = soft_unify_chirho(l_chirho, l_result_chirho)
                _, prob_s_chirho = soft_unify_chirho(s_chirho, s_result_chirho)

                total_prob_chirho = prob_l_chirho * prob_s_chirho
                if total_prob_chirho > 1e-6:
                    results_chirho.append((l_result_chirho, s_result_chirho, out_chirho, total_prob_chirho))

        return results_chirho


# === Semiring Abstraction ===

@dataclass
class SemiringChirho:
    """
    Abstract semiring for provenance tracking.

    Different semirings give different semantics:
    - Boolean: standard logic (AND=∧, OR=∨)
    - Probability: soft logic (AND=×, OR=+)
    - Tropical: shortest path (AND=+, OR=min)
    - Counting: count proofs (AND=×, OR=+)
    """
    zero_chirho: Any  # Identity for OR
    one_chirho: Any   # Identity for AND
    add_chirho: Callable[[Any, Any], Any]  # OR operation
    mul_chirho: Callable[[Any, Any], Any]  # AND operation

    @classmethod
    def boolean_chirho(cls_chirho) -> 'SemiringChirho':
        return SemiringChirho(
            zero_chirho=False,
            one_chirho=True,
            add_chirho=lambda a, b: a or b,
            mul_chirho=lambda a, b: a and b
        )

    @classmethod
    def probability_chirho(cls_chirho) -> 'SemiringChirho':
        return SemiringChirho(
            zero_chirho=0.0,
            one_chirho=1.0,
            add_chirho=soft_or_chirho,
            mul_chirho=soft_and_chirho
        )

    @classmethod
    def tropical_chirho(cls_chirho) -> 'SemiringChirho':
        """For shortest path / minimum cost"""
        return SemiringChirho(
            zero_chirho=float('inf'),
            one_chirho=0.0,
            add_chirho=min,
            mul_chirho=lambda a, b: a + b
        )

    @classmethod
    def counting_chirho(cls_chirho) -> 'SemiringChirho':
        """Count number of derivations"""
        return SemiringChirho(
            zero_chirho=0,
            one_chirho=1,
            add_chirho=lambda a, b: a + b,
            mul_chirho=lambda a, b: a * b
        )


# === Temperature Annealing ===

def soft_eq_annealed_chirho(a_chirho: float, b_chirho: float,
                             temp_chirho: float, step_chirho: int,
                             total_steps_chirho: int) -> float:
    """
    Temperature annealing schedule for soft equality.

    - Start with high temp (soft, forgiving)
    - End with low temp (hard, strict)

    This enables training that:
    1. Initially explores broadly (high temp)
    2. Gradually commits to best solutions (low temp)
    """
    # Exponential decay schedule
    progress_chirho = step_chirho / max(total_steps_chirho, 1)
    current_temp_chirho = temp_chirho * (0.1 ** progress_chirho)
    current_temp_chirho = max(current_temp_chirho, 1e-6)

    diff_chirho = abs(a_chirho - b_chirho)
    return math.exp(-diff_chirho / current_temp_chirho)


def annealed_temp_chirho(initial_temp_chirho: float, step_chirho: int,
                          total_steps_chirho: int, min_temp_chirho: float = 0.01) -> float:
    """
    Compute annealed temperature for a given step.

    Uses exponential decay: T(t) = T0 * (T_min/T0)^(t/T)
    """
    progress_chirho = step_chirho / max(total_steps_chirho, 1)
    ratio_chirho = max(min_temp_chirho / initial_temp_chirho, 1e-10)
    return initial_temp_chirho * (ratio_chirho ** progress_chirho)


# === Gumbel-Softmax ===

def gumbel_sample_chirho() -> float:
    """Sample from Gumbel(0, 1) distribution."""
    u_chirho = random.random()
    u_chirho = max(u_chirho, 1e-10)  # Avoid log(0)
    u_chirho = min(u_chirho, 1.0 - 1e-10)
    return -math.log(-math.log(u_chirho))


def softmax_chirho(logits_chirho: List[float]) -> List[float]:
    """Numerically stable softmax."""
    max_val_chirho = max(logits_chirho)
    exp_chirho = [math.exp(l - max_val_chirho) for l in logits_chirho]
    sum_exp_chirho = sum(exp_chirho)
    return [e / sum_exp_chirho for e in exp_chirho]


def gumbel_softmax_chirho(logits_chirho: List[float], temp_chirho: float = 1.0) -> List[float]:
    """
    Gumbel-Softmax: differentiable approximation to argmax.

    Enables gradient flow through branch selection in conde.

    At low temperature: approaches one-hot (hard selection)
    At high temperature: approaches uniform (soft exploration)

    Args:
        logits_chirho: Log-probabilities for each option
        temp_chirho: Temperature parameter (lower = harder selection)

    Returns:
        Soft one-hot vector (sums to 1, differentiable)
    """
    gumbels_chirho = [gumbel_sample_chirho() for _ in logits_chirho]

    scaled_chirho = [(l + g) / max(temp_chirho, 1e-10)
                     for l, g in zip(logits_chirho, gumbels_chirho)]

    return softmax_chirho(scaled_chirho)


def gumbel_softmax_hard_chirho(logits_chirho: List[float], temp_chirho: float = 1.0) -> Tuple[int, List[float]]:
    """
    Gumbel-Softmax with hard selection.

    Returns both:
    - Hard index (argmax of soft values)
    - Soft values (for gradient computation)

    This enables:
    - Forward: discrete selection
    - Backward: continuous gradient
    """
    soft_chirho = gumbel_softmax_chirho(logits_chirho, temp_chirho)
    hard_idx_chirho = max(range(len(soft_chirho)), key=lambda i: soft_chirho[i])
    return hard_idx_chirho, soft_chirho


# === Straight-Through Estimator ===

class StraightThroughChirho:
    """
    Straight-Through Estimator for hard constraints.

    Forward: uses hard decision (argmax)
    Backward: uses soft gradient (softmax)

    This allows learning with hard constraints while still
    propagating gradients through the selection.
    """

    @staticmethod
    def forward_chirho(probs_chirho: List[float]) -> int:
        """Return argmax (hard decision)."""
        return max(range(len(probs_chirho)), key=lambda i: probs_chirho[i])

    @staticmethod
    def backward_chirho(probs_chirho: List[float], grad_out_chirho: float) -> List[float]:
        """Distribute gradient via probability weights."""
        # Gradient flows proportionally to probability
        return [p * grad_out_chirho for p in probs_chirho]

    @staticmethod
    def forward_backward_chirho(probs_chirho: List[float]) -> Tuple[int, List[float]]:
        """Combined forward-backward for training."""
        idx_chirho = StraightThroughChirho.forward_chirho(probs_chirho)
        return idx_chirho, probs_chirho


# === Differentiable Value with Gradient ===

@dataclass
class DiffValueChirho:
    """
    A value that tracks its gradient for backpropagation.
    """
    value_chirho: float
    grad_chirho: float = 0.0

    def __add__(self_chirho, other_chirho: 'DiffValueChirho') -> 'DiffValueChirho':
        return DiffValueChirho(self_chirho.value_chirho + other_chirho.value_chirho)

    def __mul__(self_chirho, other_chirho: 'DiffValueChirho') -> 'DiffValueChirho':
        return DiffValueChirho(self_chirho.value_chirho * other_chirho.value_chirho)

    def soft_and_chirho(self_chirho, other_chirho: 'DiffValueChirho') -> 'DiffValueChirho':
        """Soft AND with gradient info preserved."""
        return DiffValueChirho(self_chirho.value_chirho * other_chirho.value_chirho)

    def soft_or_chirho(self_chirho, other_chirho: 'DiffValueChirho') -> 'DiffValueChirho':
        """Soft OR (inclusion-exclusion)."""
        a_chirho = self_chirho.value_chirho
        b_chirho = other_chirho.value_chirho
        return DiffValueChirho(a_chirho + b_chirho - a_chirho * b_chirho)

    def soft_not_chirho(self_chirho) -> 'DiffValueChirho':
        """Soft NOT (complement)."""
        return DiffValueChirho(1.0 - self_chirho.value_chirho)


# === Soft Equality with Gradient ===

def soft_eq_with_grad_chirho(a_chirho: float, b_chirho: float,
                              temp_chirho: float = 1.0) -> Tuple[float, float, float]:
    """
    Soft equality with gradients.

    Returns:
        (probability, grad_a, grad_b)

    gradient = ∂prob/∂a = prob * (-sign(a-b) / temp)
    """
    diff_chirho = a_chirho - b_chirho
    abs_diff_chirho = abs(diff_chirho)
    t_chirho = max(temp_chirho, 1e-10)
    prob_chirho = math.exp(-abs_diff_chirho / t_chirho)

    # Gradient computation
    if diff_chirho >= 0:
        sign_chirho = 1.0
    else:
        sign_chirho = -1.0

    grad_a_chirho = prob_chirho * (-sign_chirho / t_chirho)
    grad_b_chirho = -grad_a_chirho

    return prob_chirho, grad_a_chirho, grad_b_chirho


# === Log-Sum-Exp (Numerically Stable) ===

def log_sum_exp_chirho(values_chirho: List[float]) -> float:
    """
    Numerically stable log-sum-exp.

    log(sum(exp(values))) = max + log(sum(exp(values - max)))
    """
    if not values_chirho:
        return float('-inf')

    max_val_chirho = max(values_chirho)
    if math.isinf(max_val_chirho):
        return max_val_chirho

    sum_exp_chirho = sum(math.exp(v - max_val_chirho) for v in values_chirho)
    return max_val_chirho + math.log(sum_exp_chirho)


# === Demo ===

def main():
    print("=== Differentiable miniKanren ☧ ===\n")

    # === Soft operations ===
    print("=" * 60)
    print("SOFT BOOLEAN OPERATIONS")
    print("=" * 60)

    print(f"\nsoft_and(0.8, 0.9) = {soft_and_chirho(0.8, 0.9):.3f}")
    print(f"soft_or(0.8, 0.9)  = {soft_or_chirho(0.8, 0.9):.3f}")
    print(f"soft_not(0.8)      = {soft_not_chirho(0.8):.3f}")
    print(f"soft_eq(0.5, 0.5)  = {soft_eq_chirho(0.5, 0.5):.3f}")
    print(f"soft_eq(0.5, 0.9)  = {soft_eq_chirho(0.5, 0.9):.3f}")

    # === Probabilistic patterns ===
    print("\n" + "=" * 60)
    print("PROBABILISTIC PATTERNS")
    print("=" * 60)

    # Ground pattern
    p1_chirho = ProbPatternChirho.from_list_chirho([0, 1, 2])
    print(f"\nGround pattern: {p1_chirho}")
    print(f"  Most likely: {p1_chirho.most_likely_chirho()}")
    print(f"  Entropy: {p1_chirho.entropy_chirho():.3f}")

    # Uncertain pattern
    p2_chirho = ProbPatternChirho()
    p2_chirho.length_chirho = 3
    p2_chirho.set_prob_chirho(0, 0, 0.9)  # 90% sure pos 0 = 0
    p2_chirho.set_prob_chirho(0, 1, 0.1)  # 10% chance pos 0 = 1
    p2_chirho.set_prob_chirho(1, 1, 0.5)  # 50/50 on pos 1
    p2_chirho.set_prob_chirho(1, 2, 0.5)
    p2_chirho.set_prob_chirho(2, 2, 0.7)  # 70% sure pos 2 = 2
    p2_chirho.set_prob_chirho(2, 3, 0.3)

    print(f"\nUncertain pattern: {p2_chirho}")
    print(f"  Most likely: {p2_chirho.most_likely_chirho()}")
    print(f"  Entropy: {p2_chirho.entropy_chirho():.3f}")

    # === Soft unification ===
    print("\n" + "=" * 60)
    print("SOFT UNIFICATION")
    print("=" * 60)

    unified_chirho, prob_chirho = soft_unify_chirho(p1_chirho, p2_chirho)
    print(f"\nUnify {p1_chirho} with uncertain pattern:")
    print(f"  Result: {unified_chirho}")
    print(f"  Match probability: {prob_chirho:.4f}")

    # Conflict
    p3_chirho = ProbPatternChirho.from_list_chirho([9, 9, 9])
    unified2_chirho, prob2_chirho = soft_unify_chirho(p1_chirho, p3_chirho)
    print(f"\nUnify {p1_chirho} with {p3_chirho}:")
    print(f"  Match probability: {prob2_chirho:.6f} (very low - conflict!)")

    # === Soft appendo ===
    print("\n" + "=" * 60)
    print("SOFT APPENDO")
    print("=" * 60)

    appendo_chirho = SoftAppendoChirho()

    # Forward
    l_chirho = ProbPatternChirho.from_list_chirho([0, 1])
    s_chirho = ProbPatternChirho.from_list_chirho([2, 3])
    out_chirho = ProbPatternChirho.uniform_chirho(4)

    results_chirho = appendo_chirho.call_chirho(l_chirho, s_chirho, out_chirho)
    print(f"\nForward: appendo([0,1], [2,3], Out)")
    for l_r_chirho, s_r_chirho, out_r_chirho, prob_chirho in results_chirho:
        print(f"  {out_r_chirho.most_likely_chirho()} with prob {prob_chirho:.4f}")

    # Backward
    l2_chirho = ProbPatternChirho.uniform_chirho(2)
    s2_chirho = ProbPatternChirho.uniform_chirho(2)
    out2_chirho = ProbPatternChirho.from_list_chirho([0, 1, 2])

    results2_chirho = appendo_chirho.call_chirho(l2_chirho, s2_chirho, out2_chirho)
    print(f"\nBackward: appendo(L, S, [0,1,2])")
    for l_r_chirho, s_r_chirho, out_r_chirho, prob_chirho in results2_chirho[:5]:
        l_list_chirho = l_r_chirho.most_likely_chirho() if l_r_chirho.length_chirho > 0 else []
        s_list_chirho = s_r_chirho.most_likely_chirho() if s_r_chirho.length_chirho > 0 else []
        print(f"  L={l_list_chirho}, S={s_list_chirho} with prob {prob_chirho:.4f}")

    # === Semirings ===
    print("\n" + "=" * 60)
    print("SEMIRING ABSTRACTION")
    print("=" * 60)

    bool_sr_chirho = SemiringChirho.boolean_chirho()
    prob_sr_chirho = SemiringChirho.probability_chirho()
    trop_sr_chirho = SemiringChirho.tropical_chirho()
    count_sr_chirho = SemiringChirho.counting_chirho()

    print("\nSame query, different semirings:")
    print(f"  Boolean:     AND(T,T)={bool_sr_chirho.mul_chirho(True,True)}, OR(T,F)={bool_sr_chirho.add_chirho(True,False)}")
    print(f"  Probability: AND(0.8,0.9)={prob_sr_chirho.mul_chirho(0.8,0.9):.2f}, OR(0.3,0.4)={prob_sr_chirho.add_chirho(0.3,0.4):.2f}")
    print(f"  Tropical:    AND(2,3)={trop_sr_chirho.mul_chirho(2,3)}, OR(2,3)={trop_sr_chirho.add_chirho(2,3)}")
    print(f"  Counting:    AND(2,3)={count_sr_chirho.mul_chirho(2,3)}, OR(2,3)={count_sr_chirho.add_chirho(2,3)}")

    # === Summary ===
    print("\n" + "=" * 60)
    print("DIFFERENTIABLE LOGIC")
    print("=" * 60)
    print("""
    Key insight: Logic programming over semirings

    Boolean semiring (standard):
      AND = conjunction, OR = disjunction
      Result: yes/no

    Probability semiring (this file):
      AND = multiply, OR = probabilistic sum
      Result: probability of success
      GRADIENT FLOWS THROUGH!

    Tropical semiring:
      AND = add costs, OR = take minimum
      Result: shortest proof / minimum cost

    Counting semiring:
      AND = multiply, OR = add
      Result: number of distinct proofs

    Applications:
    - Neural-symbolic AI (learn from logic)
    - Probabilistic programming
    - Weighted model counting
    - Differentiable theorem proving

    Connection to 1-bit:
    - Boolean semiring = 1-bit operations
    - Probability semiring = floating point
    - Can quantize probabilities to k-bit!

    The miniKanren equation becomes:

      search(query) = Σ_{proof} Π_{step} weight(step)

    This is differentiable w.r.t. weights!
    """)


if __name__ == "__main__":
    main()
