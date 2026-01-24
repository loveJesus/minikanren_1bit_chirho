#!/usr/bin/env python3
"""
Learnable Relations ☧

Relations with learnable tuple weights, enabling gradient-based
learning from examples.

Key insight: each tuple in a relation has a weight that can be
learned via gradient descent. This connects miniKanren to
neural-symbolic AI.

Example use cases:
- Learn which rules are most important
- Weight training examples by relevance
- Differentiable program synthesis
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Callable, Any
import math
import random

from differentiable_chirho import (
    ProbPatternChirho,
    soft_unify_chirho,
    soft_and_chirho,
    soft_or_chirho,
    softmax_chirho,
    gumbel_softmax_chirho,
    log_sum_exp_chirho,
)


@dataclass
class WeightedTupleChirho:
    """
    A tuple with a learnable weight.

    The weight determines how much this tuple contributes
    to query results. Gradient descent can adjust weights
    to match training data.
    """
    data_chirho: Tuple[Any, ...]
    log_weight_chirho: float = 0.0  # Log-space for numerical stability
    grad_log_weight_chirho: float = 0.0

    @property
    def weight_chirho(self_chirho) -> float:
        """Get the actual weight (exp of log-weight)."""
        return math.exp(self_chirho.log_weight_chirho)

    def update_chirho(self_chirho, lr_chirho: float):
        """Apply gradient descent step."""
        self_chirho.log_weight_chirho -= lr_chirho * self_chirho.grad_log_weight_chirho
        self_chirho.grad_log_weight_chirho = 0.0


class LearnableRelationChirho:
    """
    A relation with learnable tuple weights.

    Each tuple (a, b, c) in the relation has a weight w ∈ (0, 1).
    Gradients flow through soft unification to update weights.
    """

    def __init__(self_chirho, name_chirho: str, tuples_chirho: List[Tuple] = None,
                 lr_chirho: float = 0.01):
        self_chirho.name_chirho = name_chirho
        self_chirho.tuples_chirho: List[WeightedTupleChirho] = []
        self_chirho.lr_chirho = lr_chirho

        if tuples_chirho:
            for tup_chirho in tuples_chirho:
                self_chirho.add_tuple_chirho(tup_chirho)

    def add_tuple_chirho(self_chirho, data_chirho: Tuple, weight_chirho: float = 1.0):
        """Add a tuple with initial weight."""
        log_w_chirho = math.log(max(weight_chirho, 1e-10))
        self_chirho.tuples_chirho.append(
            WeightedTupleChirho(data_chirho=data_chirho, log_weight_chirho=log_w_chirho)
        )

    def weights_chirho(self_chirho) -> List[float]:
        """Get normalized weights (softmax over log-weights)."""
        log_weights_chirho = [t.log_weight_chirho for t in self_chirho.tuples_chirho]
        return softmax_chirho(log_weights_chirho)

    def raw_weights_chirho(self_chirho) -> List[float]:
        """Get unnormalized weights (for debugging)."""
        return [t.weight_chirho for t in self_chirho.tuples_chirho]

    def query_soft_chirho(self_chirho, pattern_chirho: ProbPatternChirho) -> float:
        """
        Soft query: returns weighted sum of match probabilities.

        probability = Σ_i (weight_i * match_prob_i)
        """
        total_prob_chirho = 0.0
        weights_chirho = self_chirho.weights_chirho()

        for tup_chirho, weight_chirho in zip(self_chirho.tuples_chirho, weights_chirho):
            # Convert tuple to pattern
            if hasattr(tup_chirho.data_chirho, '__iter__') and not isinstance(tup_chirho.data_chirho, str):
                tup_pattern_chirho = ProbPatternChirho.from_list_chirho(list(tup_chirho.data_chirho))
            else:
                tup_pattern_chirho = ProbPatternChirho.from_list_chirho([tup_chirho.data_chirho])

            # Soft unification
            _, match_prob_chirho = soft_unify_chirho(pattern_chirho, tup_pattern_chirho)

            total_prob_chirho += match_prob_chirho * weight_chirho

        return total_prob_chirho

    def query_top_k_chirho(self_chirho, pattern_chirho: ProbPatternChirho,
                           k_chirho: int = 5) -> List[Tuple[Tuple, float, float]]:
        """
        Query and return top-k matches.

        Returns: List of (tuple_data, weight, match_prob)
        """
        results_chirho = []
        weights_chirho = self_chirho.weights_chirho()

        for tup_chirho, weight_chirho in zip(self_chirho.tuples_chirho, weights_chirho):
            if hasattr(tup_chirho.data_chirho, '__iter__') and not isinstance(tup_chirho.data_chirho, str):
                tup_pattern_chirho = ProbPatternChirho.from_list_chirho(list(tup_chirho.data_chirho))
            else:
                tup_pattern_chirho = ProbPatternChirho.from_list_chirho([tup_chirho.data_chirho])

            _, match_prob_chirho = soft_unify_chirho(pattern_chirho, tup_pattern_chirho)

            score_chirho = match_prob_chirho * weight_chirho
            results_chirho.append((tup_chirho.data_chirho, weight_chirho, match_prob_chirho))

        # Sort by score descending
        results_chirho.sort(key=lambda x: x[1] * x[2], reverse=True)
        return results_chirho[:k_chirho]

    def update_chirho(self_chirho, pattern_chirho: ProbPatternChirho, target_chirho: float) -> float:
        """
        Update weights via gradient descent.

        Loss = (query_soft(pattern) - target)^2

        Returns the loss value.
        """
        # Forward pass
        pred_chirho = self_chirho.query_soft_chirho(pattern_chirho)
        loss_chirho = (pred_chirho - target_chirho) ** 2

        # Backward pass
        grad_pred_chirho = 2 * (pred_chirho - target_chirho)
        weights_chirho = self_chirho.weights_chirho()

        for i, (tup_chirho, weight_chirho) in enumerate(zip(self_chirho.tuples_chirho, weights_chirho)):
            # Compute match probability
            if hasattr(tup_chirho.data_chirho, '__iter__') and not isinstance(tup_chirho.data_chirho, str):
                tup_pattern_chirho = ProbPatternChirho.from_list_chirho(list(tup_chirho.data_chirho))
            else:
                tup_pattern_chirho = ProbPatternChirho.from_list_chirho([tup_chirho.data_chirho])

            _, match_prob_chirho = soft_unify_chirho(pattern_chirho, tup_pattern_chirho)

            # Gradient of softmax output w.r.t. log-weight
            # ∂softmax_i/∂log_w_i = softmax_i * (1 - softmax_i)
            grad_softmax_chirho = weight_chirho * (1 - weight_chirho)

            # Chain rule: ∂L/∂log_w_i = ∂L/∂pred * match_prob * ∂softmax_i/∂log_w_i
            grad_i_chirho = grad_pred_chirho * match_prob_chirho * grad_softmax_chirho

            tup_chirho.grad_log_weight_chirho += grad_i_chirho

        # Apply updates
        for tup_chirho in self_chirho.tuples_chirho:
            tup_chirho.update_chirho(self_chirho.lr_chirho)

        return loss_chirho

    def train_batch_chirho(self_chirho, examples_chirho: List[Tuple[ProbPatternChirho, float]],
                           epochs_chirho: int = 100) -> List[float]:
        """
        Train on a batch of examples.

        Args:
            examples_chirho: List of (pattern, target_probability)
            epochs_chirho: Number of training epochs

        Returns:
            List of loss values per epoch
        """
        losses_chirho = []

        for epoch_chirho in range(epochs_chirho):
            epoch_loss_chirho = 0.0

            for pattern_chirho, target_chirho in examples_chirho:
                loss_chirho = self_chirho.update_chirho(pattern_chirho, target_chirho)
                epoch_loss_chirho += loss_chirho

            avg_loss_chirho = epoch_loss_chirho / len(examples_chirho)
            losses_chirho.append(avg_loss_chirho)

        return losses_chirho

    def zero_grad_chirho(self_chirho):
        """Zero all gradients."""
        for tup_chirho in self_chirho.tuples_chirho:
            tup_chirho.grad_log_weight_chirho = 0.0

    def __repr__(self_chirho):
        weights_chirho = self_chirho.weights_chirho()
        tuples_str_chirho = ", ".join(
            f"{t.data_chirho}:{w:.3f}"
            for t, w in zip(self_chirho.tuples_chirho, weights_chirho)
        )
        return f"LearnableRelation({self_chirho.name_chirho}: [{tuples_str_chirho}])"


class DifferentiableGoalChirho:
    """
    A goal that returns soft probabilities instead of Boolean success/failure.

    This enables gradient-based optimization of goal parameters.
    """

    def __init__(self_chirho, relation_chirho: LearnableRelationChirho):
        self_chirho.relation_chirho = relation_chirho

    def query_chirho(self_chirho, *args_chirho) -> float:
        """Query the relation with given arguments."""
        pattern_chirho = ProbPatternChirho.from_list_chirho(list(args_chirho))
        return self_chirho.relation_chirho.query_soft_chirho(pattern_chirho)


class DifferentiableCondeChirho:
    """
    Differentiable conde (disjunction) with learnable branch weights.

    Each branch has a weight that can be learned to prefer
    certain branches over others.
    """

    def __init__(self_chirho, branches_chirho: List[Callable[[], float]],
                 lr_chirho: float = 0.01):
        self_chirho.branches_chirho = branches_chirho
        self_chirho.log_weights_chirho = [0.0] * len(branches_chirho)
        self_chirho.lr_chirho = lr_chirho

    def weights_chirho(self_chirho) -> List[float]:
        """Get normalized branch weights."""
        return softmax_chirho(self_chirho.log_weights_chirho)

    def query_soft_chirho(self_chirho) -> float:
        """
        Soft disjunction: weighted sum of branch probabilities.
        """
        weights_chirho = self_chirho.weights_chirho()
        total_chirho = 0.0

        for branch_chirho, weight_chirho in zip(self_chirho.branches_chirho, weights_chirho):
            branch_prob_chirho = branch_chirho()
            total_chirho = soft_or_chirho(total_chirho, branch_prob_chirho * weight_chirho)

        return total_chirho

    def query_hard_chirho(self_chirho) -> Tuple[int, float]:
        """
        Hard selection using Gumbel-Softmax.

        Returns (selected_branch_index, probability)
        """
        soft_chirho = gumbel_softmax_chirho(self_chirho.log_weights_chirho, temp_chirho=0.1)
        idx_chirho = max(range(len(soft_chirho)), key=lambda i: soft_chirho[i])
        return idx_chirho, self_chirho.branches_chirho[idx_chirho]()


class DifferentiableConjChirho:
    """
    Differentiable conj (conjunction) as probability multiplication.
    """

    def __init__(self_chirho, goals_chirho: List[Callable[[], float]]):
        self_chirho.goals_chirho = goals_chirho

    def query_soft_chirho(self_chirho) -> float:
        """Soft conjunction: product of goal probabilities."""
        result_chirho = 1.0
        for goal_chirho in self_chirho.goals_chirho:
            result_chirho = soft_and_chirho(result_chirho, goal_chirho())
        return result_chirho


def main():
    print("=== Learnable Relations ☧ ===\n")

    # === Basic learnable relation ===
    print("=" * 60)
    print("LEARNABLE RELATION")
    print("=" * 60)

    rel_chirho = LearnableRelationChirho("numbers", [
        (0, 0),  # Tuple 0
        (1, 1),  # Tuple 1
        (2, 2),  # Tuple 2
        (3, 3),  # Tuple 3
    ], lr_chirho=0.1)

    print(f"\nInitial relation: {rel_chirho}")
    print(f"Weights: {[f'{w:.3f}' for w in rel_chirho.weights_chirho()]}")

    # === Train to prefer (1, 1) ===
    print("\n" + "=" * 60)
    print("TRAINING TO PREFER (1, 1)")
    print("=" * 60)

    # Create training example: (1, 1) should have high probability
    pattern_chirho = ProbPatternChirho.from_list_chirho([1, 1], confidence_chirho=0.9)

    losses_chirho = []
    for epoch_chirho in range(100):
        loss_chirho = rel_chirho.update_chirho(pattern_chirho, target_chirho=1.0)
        if epoch_chirho % 20 == 0:
            losses_chirho.append(loss_chirho)
            print(f"Epoch {epoch_chirho}: loss = {loss_chirho:.6f}")

    print(f"\nFinal relation: {rel_chirho}")
    print(f"Weights: {[f'{w:.3f}' for w in rel_chirho.weights_chirho()]}")

    # Verify weight for (1, 1) is highest
    weights_chirho = rel_chirho.weights_chirho()
    assert weights_chirho[1] > weights_chirho[0], "Weight for (1,1) should be > (0,0)"
    assert weights_chirho[1] > weights_chirho[2], "Weight for (1,1) should be > (2,2)"
    print("PASSED: (1, 1) has highest weight!")

    # === Query the learned relation ===
    print("\n" + "=" * 60)
    print("QUERYING LEARNED RELATION")
    print("=" * 60)

    query_patterns_chirho = [
        [0, 0],
        [1, 1],
        [2, 2],
        [1, 2],  # No match
    ]

    for query_chirho in query_patterns_chirho:
        pattern_chirho = ProbPatternChirho.from_list_chirho(query_chirho, confidence_chirho=0.9)
        prob_chirho = rel_chirho.query_soft_chirho(pattern_chirho)
        print(f"Query {query_chirho}: probability = {prob_chirho:.4f}")

    # === Differentiable conde ===
    print("\n" + "=" * 60)
    print("DIFFERENTIABLE CONDE")
    print("=" * 60)

    # Three branches with different probabilities
    branches_chirho = [
        lambda: 0.3,  # Branch 0: always returns 0.3
        lambda: 0.8,  # Branch 1: always returns 0.8
        lambda: 0.5,  # Branch 2: always returns 0.5
    ]

    conde_chirho = DifferentiableCondeChirho(branches_chirho)

    soft_result_chirho = conde_chirho.query_soft_chirho()
    print(f"\nSoft conde result: {soft_result_chirho:.4f}")
    print(f"Branch weights: {[f'{w:.3f}' for w in conde_chirho.weights_chirho()]}")

    hard_idx_chirho, hard_prob_chirho = conde_chirho.query_hard_chirho()
    print(f"Hard selection: branch {hard_idx_chirho} with prob {hard_prob_chirho:.4f}")

    # === Summary ===
    print("\n" + "=" * 60)
    print("LEARNABLE RELATIONS SUMMARY")
    print("=" * 60)
    print("""
    Key features:

    1. LearnableRelationChirho: Relations with weighted tuples
       - Weights are learnable via gradient descent
       - Soft queries return probability of match
       - Can train on (pattern, target_probability) pairs

    2. DifferentiableCondeChirho: Learnable branch selection
       - Each branch has a weight
       - Soft: weighted sum of branch probabilities
       - Hard: Gumbel-Softmax selection

    3. DifferentiableConjChirho: Soft conjunction
       - Probability product across goals

    Applications:
    - Learn which facts are most relevant
    - Weight evidence in probabilistic inference
    - Differentiable program synthesis
    - Neural-symbolic integration

    Connection to 1-bit:
    - Boolean logic = limit as temperature → 0
    - Can quantize weights to k-bit precision
    - Enables hardware-friendly soft logic
    """)


if __name__ == "__main__":
    main()
