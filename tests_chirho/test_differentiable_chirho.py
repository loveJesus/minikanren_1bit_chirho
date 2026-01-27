#!/usr/bin/env python3
"""
Tests for Differentiable miniKanren ☧

Tests the differentiable relaxation features:
- Soft operations (AND, OR, NOT, EQ)
- Temperature annealing
- Gumbel-Softmax
- Straight-Through Estimator
- Learnable Relations
"""

import sys
import os
import math

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from differentiable_chirho import (
    soft_and_chirho,
    soft_or_chirho,
    soft_not_chirho,
    soft_eq_chirho,
    soft_eq_annealed_chirho,
    annealed_temp_chirho,
    softmax_chirho,
    gumbel_softmax_chirho,
    gumbel_softmax_hard_chirho,
    StraightThroughChirho,
    DiffValueChirho,
    soft_eq_with_grad_chirho,
    log_sum_exp_chirho,
    ProbPatternChirho,
    soft_unify_chirho,
    SemiringChirho,
)

from learn_relations_chirho import (
    LearnableRelationChirho,
    WeightedTupleChirho,
    DifferentiableGoalChirho,
    DifferentiableCondeChirho,
    DifferentiableConjChirho,
)


def test_soft_and_chirho():
    """Soft AND = probability multiplication."""
    result_chirho = soft_and_chirho(0.8, 0.9)
    expected_chirho = 0.72
    assert abs(result_chirho - expected_chirho) < 1e-10
    print("PASS: test_soft_and_chirho")


def test_soft_or_chirho():
    """Soft OR = inclusion-exclusion."""
    result_chirho = soft_or_chirho(0.3, 0.4)
    # 0.3 + 0.4 - 0.3*0.4 = 0.58
    expected_chirho = 0.58
    assert abs(result_chirho - expected_chirho) < 1e-10
    print("PASS: test_soft_or_chirho")


def test_soft_not_chirho():
    """Soft NOT = complement."""
    result_chirho = soft_not_chirho(0.8)
    expected_chirho = 0.2
    assert abs(result_chirho - expected_chirho) < 1e-10
    print("PASS: test_soft_not_chirho")


def test_soft_eq_same_chirho():
    """Soft EQ of same value = 1.0."""
    result_chirho = soft_eq_chirho(0.5, 0.5, temp_chirho=1.0)
    assert abs(result_chirho - 1.0) < 1e-10
    print("PASS: test_soft_eq_same_chirho")


def test_soft_eq_different_chirho():
    """Soft EQ of different values < 1.0."""
    result_chirho = soft_eq_chirho(0.0, 1.0, temp_chirho=1.0)
    # exp(-1/1) = exp(-1) ≈ 0.368
    expected_chirho = math.exp(-1)
    assert abs(result_chirho - expected_chirho) < 1e-10
    print("PASS: test_soft_eq_different_chirho")


def test_temperature_annealing_endpoints_chirho():
    """Temperature annealing reaches endpoints."""
    # At start
    temp_start_chirho = annealed_temp_chirho(1.0, step_chirho=0, total_steps_chirho=100, min_temp_chirho=0.01)
    assert abs(temp_start_chirho - 1.0) < 1e-6

    # At end
    temp_end_chirho = annealed_temp_chirho(1.0, step_chirho=100, total_steps_chirho=100, min_temp_chirho=0.01)
    assert abs(temp_end_chirho - 0.01) < 1e-6

    print("PASS: test_temperature_annealing_endpoints_chirho")


def test_temperature_annealing_monotonic_chirho():
    """Temperature annealing is monotonically decreasing."""
    temps_chirho = [
        annealed_temp_chirho(1.0, step_chirho=i, total_steps_chirho=100, min_temp_chirho=0.01)
        for i in range(101)
    ]

    for i_chirho in range(len(temps_chirho) - 1):
        assert temps_chirho[i_chirho] >= temps_chirho[i_chirho + 1]

    print("PASS: test_temperature_annealing_monotonic_chirho")


def test_soft_eq_annealed_gets_stricter_chirho():
    """Annealed soft EQ becomes stricter over time."""
    # High temp (start) - more forgiving
    prob_high_chirho = soft_eq_annealed_chirho(0.5, 0.6, temp_chirho=1.0, step_chirho=0, total_steps_chirho=100)

    # Low temp (end) - more strict
    prob_low_chirho = soft_eq_annealed_chirho(0.5, 0.6, temp_chirho=1.0, step_chirho=99, total_steps_chirho=100)

    # At low temp, probability should be lower (more selective)
    assert prob_low_chirho < prob_high_chirho
    print("PASS: test_soft_eq_annealed_gets_stricter_chirho")


def test_softmax_sums_to_one_chirho():
    """Softmax output sums to 1."""
    logits_chirho = [1.0, 2.0, 3.0]
    probs_chirho = softmax_chirho(logits_chirho)

    assert abs(sum(probs_chirho) - 1.0) < 1e-10
    print("PASS: test_softmax_sums_to_one_chirho")


def test_softmax_max_has_highest_prob_chirho():
    """Softmax gives highest probability to highest logit."""
    logits_chirho = [1.0, 3.0, 2.0]
    probs_chirho = softmax_chirho(logits_chirho)

    assert probs_chirho[1] > probs_chirho[0]
    assert probs_chirho[1] > probs_chirho[2]
    print("PASS: test_softmax_max_has_highest_prob_chirho")


def test_gumbel_softmax_sums_to_one_chirho():
    """Gumbel-Softmax output sums to 1."""
    logits_chirho = [0.0, 1.0, -1.0]
    probs_chirho = gumbel_softmax_chirho(logits_chirho, temp_chirho=1.0)

    assert abs(sum(probs_chirho) - 1.0) < 1e-10
    print("PASS: test_gumbel_softmax_sums_to_one_chirho")


def test_gumbel_softmax_low_temp_sparse_chirho():
    """Gumbel-Softmax at low temperature is near one-hot."""
    logits_chirho = [0.0, 2.0, -1.0]

    # Low temperature should give near one-hot
    probs_chirho = gumbel_softmax_chirho(logits_chirho, temp_chirho=0.01)

    max_prob_chirho = max(probs_chirho)
    assert max_prob_chirho > 0.95
    print("PASS: test_gumbel_softmax_low_temp_sparse_chirho")


def test_gumbel_softmax_hard_returns_index_chirho():
    """Gumbel-Softmax hard returns valid index."""
    logits_chirho = [0.0, 1.0, -1.0]

    idx_chirho, soft_chirho = gumbel_softmax_hard_chirho(logits_chirho, temp_chirho=0.1)

    assert 0 <= idx_chirho < len(logits_chirho)
    assert abs(sum(soft_chirho) - 1.0) < 1e-10
    print("PASS: test_gumbel_softmax_hard_returns_index_chirho")


def test_straight_through_forward_chirho():
    """Straight-through forward returns argmax."""
    probs_chirho = [0.1, 0.6, 0.3]
    idx_chirho = StraightThroughChirho.forward_chirho(probs_chirho)

    assert idx_chirho == 1
    print("PASS: test_straight_through_forward_chirho")


def test_straight_through_backward_chirho():
    """Straight-through backward distributes gradient."""
    probs_chirho = [0.1, 0.6, 0.3]
    grads_chirho = StraightThroughChirho.backward_chirho(probs_chirho, grad_out_chirho=1.0)

    assert len(grads_chirho) == 3
    assert abs(grads_chirho[0] - 0.1) < 1e-10
    assert abs(grads_chirho[1] - 0.6) < 1e-10
    assert abs(grads_chirho[2] - 0.3) < 1e-10
    print("PASS: test_straight_through_backward_chirho")


def test_diff_value_and_chirho():
    """DiffValue soft AND works."""
    a_chirho = DiffValueChirho(value_chirho=0.8)
    b_chirho = DiffValueChirho(value_chirho=0.9)
    result_chirho = a_chirho.soft_and_chirho(b_chirho)

    assert abs(result_chirho.value_chirho - 0.72) < 1e-10
    print("PASS: test_diff_value_and_chirho")


def test_diff_value_or_chirho():
    """DiffValue soft OR works."""
    a_chirho = DiffValueChirho(value_chirho=0.3)
    b_chirho = DiffValueChirho(value_chirho=0.4)
    result_chirho = a_chirho.soft_or_chirho(b_chirho)

    assert abs(result_chirho.value_chirho - 0.58) < 1e-10
    print("PASS: test_diff_value_or_chirho")


def test_soft_eq_with_grad_chirho():
    """Soft EQ with gradient computation."""
    prob_chirho, grad_a_chirho, grad_b_chirho = soft_eq_with_grad_chirho(0.5, 0.5, temp_chirho=1.0)

    # Same values should give probability 1.0
    assert abs(prob_chirho - 1.0) < 1e-10

    # Gradients should be opposite (moving a up decreases match, moving b up increases match relative to a)
    assert abs(grad_a_chirho + grad_b_chirho) < 1e-10

    # Test with different values
    prob2_chirho, grad_a2_chirho, grad_b2_chirho = soft_eq_with_grad_chirho(0.0, 1.0, temp_chirho=1.0)
    # Moving a towards 1 (up) should increase probability
    assert grad_a2_chirho > 0 or grad_b2_chirho < 0  # At least one gradient should push towards equality

    print("PASS: test_soft_eq_with_grad_chirho")


def test_log_sum_exp_chirho():
    """Log-sum-exp is numerically stable."""
    values_chirho = [1000.0, 1001.0, 1002.0]
    result_chirho = log_sum_exp_chirho(values_chirho)

    # Should not overflow
    assert not math.isinf(result_chirho)
    assert not math.isnan(result_chirho)

    # Check correctness
    expected_chirho = 1002.0 + math.log(math.exp(-2) + math.exp(-1) + 1)
    assert abs(result_chirho - expected_chirho) < 1e-10
    print("PASS: test_log_sum_exp_chirho")


def test_learnable_relation_initial_weights_chirho():
    """Learnable relation starts with equal weights."""
    rel_chirho = LearnableRelationChirho("test", [(0, 0), (1, 1), (2, 2)])

    weights_chirho = rel_chirho.weights_chirho()
    assert len(weights_chirho) == 3
    for w_chirho in weights_chirho:
        assert abs(w_chirho - 1/3) < 1e-10

    print("PASS: test_learnable_relation_initial_weights_chirho")


def test_learnable_relation_query_chirho():
    """Learnable relation query returns probability."""
    rel_chirho = LearnableRelationChirho("test", [(0, 0), (1, 1), (2, 2)])

    pattern_chirho = ProbPatternChirho.from_list_chirho([1, 1], confidence_chirho=0.9)
    prob_chirho = rel_chirho.query_soft_chirho(pattern_chirho)

    assert 0.0 <= prob_chirho <= 1.0
    print("PASS: test_learnable_relation_query_chirho")


def test_learnable_relation_training_chirho():
    """Learnable relation weights can be trained."""
    rel_chirho = LearnableRelationChirho("test", [(0, 0), (1, 1), (2, 2)], lr_chirho=0.1)

    # Train to prefer (1, 1)
    pattern_chirho = ProbPatternChirho.from_list_chirho([1, 1], confidence_chirho=0.9)

    initial_weights_chirho = rel_chirho.weights_chirho()

    for _ in range(50):
        rel_chirho.update_chirho(pattern_chirho, target_chirho=1.0)

    final_weights_chirho = rel_chirho.weights_chirho()

    # Weight for (1, 1) should increase
    assert final_weights_chirho[1] > initial_weights_chirho[1]
    print("PASS: test_learnable_relation_training_chirho")


def test_learnable_relation_loss_decreases_chirho():
    """Training reduces loss over time."""
    rel_chirho = LearnableRelationChirho("test", [(0, 0), (1, 1), (2, 2)], lr_chirho=0.1)

    pattern_chirho = ProbPatternChirho.from_list_chirho([1, 1], confidence_chirho=0.9)

    losses_chirho = []
    for _ in range(100):
        loss_chirho = rel_chirho.update_chirho(pattern_chirho, target_chirho=1.0)
        losses_chirho.append(loss_chirho)

    # Loss should generally decrease
    assert losses_chirho[-1] < losses_chirho[0]
    print("PASS: test_learnable_relation_loss_decreases_chirho")


def test_differentiable_conde_chirho():
    """Differentiable conde returns weighted probability."""
    branches_chirho = [
        lambda: 0.3,  # Branch probability functions (no parameters)
        lambda: 0.8,
        lambda: 0.5,
    ]

    conde_chirho = DifferentiableCondeChirho(branches_chirho)

    result_chirho = conde_chirho.query_soft_chirho()
    assert 0.0 <= result_chirho <= 1.0
    print("PASS: test_differentiable_conde_chirho")


def test_differentiable_conj_chirho():
    """Differentiable conj multiplies probabilities."""
    goals_chirho = [
        lambda: 0.8,  # Goal probability functions (no parameters)
        lambda: 0.9,
    ]

    conj_chirho = DifferentiableConjChirho(goals_chirho)

    result_chirho = conj_chirho.query_soft_chirho()
    # 0.8 * 0.9 = 0.72
    assert abs(result_chirho - 0.72) < 1e-10
    print("PASS: test_differentiable_conj_chirho")


def test_semiring_boolean_chirho():
    """Boolean semiring works correctly."""
    sr_chirho = SemiringChirho.boolean_chirho()

    assert sr_chirho.mul_chirho(True, True) == True
    assert sr_chirho.mul_chirho(True, False) == False
    assert sr_chirho.add_chirho(True, False) == True
    assert sr_chirho.add_chirho(False, False) == False
    print("PASS: test_semiring_boolean_chirho")


def test_semiring_probability_chirho():
    """Probability semiring works correctly."""
    sr_chirho = SemiringChirho.probability_chirho()

    # AND = multiply
    assert abs(sr_chirho.mul_chirho(0.8, 0.9) - 0.72) < 1e-10

    # OR = inclusion-exclusion
    assert abs(sr_chirho.add_chirho(0.3, 0.4) - 0.58) < 1e-10
    print("PASS: test_semiring_probability_chirho")


def test_semiring_tropical_chirho():
    """Tropical semiring works correctly."""
    sr_chirho = SemiringChirho.tropical_chirho()

    # AND = add costs
    assert sr_chirho.mul_chirho(2, 3) == 5

    # OR = minimum
    assert sr_chirho.add_chirho(2, 3) == 2
    print("PASS: test_semiring_tropical_chirho")


def test_semiring_counting_chirho():
    """Counting semiring works correctly."""
    sr_chirho = SemiringChirho.counting_chirho()

    # AND = multiply derivation counts
    assert sr_chirho.mul_chirho(2, 3) == 6

    # OR = add derivation counts
    assert sr_chirho.add_chirho(2, 3) == 5
    print("PASS: test_semiring_counting_chirho")


def run_all_tests_chirho():
    """Run all differentiable tests."""
    print("=" * 60)
    print("Differentiable miniKanren Tests ☧")
    print("=" * 60)
    print()

    # Basic soft operations
    test_soft_and_chirho()
    test_soft_or_chirho()
    test_soft_not_chirho()
    test_soft_eq_same_chirho()
    test_soft_eq_different_chirho()

    # Temperature annealing
    test_temperature_annealing_endpoints_chirho()
    test_temperature_annealing_monotonic_chirho()
    test_soft_eq_annealed_gets_stricter_chirho()

    # Softmax and Gumbel-Softmax
    test_softmax_sums_to_one_chirho()
    test_softmax_max_has_highest_prob_chirho()
    test_gumbel_softmax_sums_to_one_chirho()
    test_gumbel_softmax_low_temp_sparse_chirho()
    test_gumbel_softmax_hard_returns_index_chirho()

    # Straight-Through Estimator
    test_straight_through_forward_chirho()
    test_straight_through_backward_chirho()

    # DiffValue
    test_diff_value_and_chirho()
    test_diff_value_or_chirho()
    test_soft_eq_with_grad_chirho()

    # Utilities
    test_log_sum_exp_chirho()

    # Learnable Relations
    test_learnable_relation_initial_weights_chirho()
    test_learnable_relation_query_chirho()
    test_learnable_relation_training_chirho()
    test_learnable_relation_loss_decreases_chirho()

    # Differentiable Goals
    test_differentiable_conde_chirho()
    test_differentiable_conj_chirho()

    # Semirings
    test_semiring_boolean_chirho()
    test_semiring_probability_chirho()
    test_semiring_tropical_chirho()
    test_semiring_counting_chirho()

    print()
    print("=" * 60)
    print("ALL TESTS PASSED!")
    print("=" * 60)


if __name__ == "__main__":
    run_all_tests_chirho()
