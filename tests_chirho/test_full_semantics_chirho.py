#!/usr/bin/env python3
"""
Tests for Full miniKanren Semantics ☧

Tests the advanced control operators:
- not_goal_chirho (negation-as-failure)
- conda_goal_chirho (soft cut)
- condu_goal_chirho (committed choice)
- diseq_goal_chirho (disequality constraints)
- project_goal_chirho (substitution access)
"""

import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from minikanren_proper_chirho import (
    VarChirho,
    NilChirho,
    ConsChirho,
    list_chirho,
    StateChirho,
    empty_state_chirho,
    eq_goal_chirho,
    call_fresh_chirho,
    disj_chirho,
    conj_chirho,
    conj_all_chirho,
    disj_all_chirho,
    run_chirho,
    not_goal_chirho,
    conda_goal_chirho,
    condu_goal_chirho,
    diseq_goal_chirho,
    project_goal_chirho,
    succeed_goal_chirho,
    fail_goal_chirho,
    walk_deep_chirho,
)


def test_negation_basic_chirho():
    """not (x == 1) succeeds when x is bound to something other than 1."""
    x_chirho = VarChirho(0)

    # First bind x to 2, THEN check not(x == 1)
    # This is the proper order for negation-as-failure
    goal_chirho = conj_chirho(
        eq_goal_chirho(x_chirho, 2),
        not_goal_chirho(eq_goal_chirho(x_chirho, 1)),
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][x_chirho.id_chirho] == 2
    print("PASS: test_negation_basic_chirho")


def test_negation_fails_when_succeeds_chirho():
    """not (x == 1) fails when x is 1."""
    x_chirho = VarChirho(0)

    # Goal: x == 1 AND not(x == 1) - should fail
    goal_chirho = conj_chirho(
        eq_goal_chirho(x_chirho, 1),
        not_goal_chirho(eq_goal_chirho(x_chirho, 1))
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 0
    print("PASS: test_negation_fails_when_succeeds_chirho")


def test_conda_basic_chirho():
    """conda commits to first successful branch."""
    x_chirho = VarChirho(0)

    # If x can be 1, make it 1; else make it 2
    goal_chirho = conda_goal_chirho(
        eq_goal_chirho(x_chirho, 1),  # Condition (succeeds)
        succeed_goal_chirho(),         # Then
        eq_goal_chirho(x_chirho, 2),  # Else (never reached)
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][x_chirho.id_chirho] == 1
    print("PASS: test_conda_basic_chirho")


def test_conda_else_branch_chirho():
    """conda takes else branch when condition fails."""
    x_chirho = VarChirho(0)

    # Condition fails (x already bound to 3, can't be 1)
    goal_chirho = conj_chirho(
        eq_goal_chirho(x_chirho, 3),
        conda_goal_chirho(
            eq_goal_chirho(x_chirho, 1),  # Condition fails (x is 3)
            eq_goal_chirho(x_chirho, 1),  # Then (not reached)
            succeed_goal_chirho(),         # Else (succeeds with x=3)
        )
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][x_chirho.id_chirho] == 3
    print("PASS: test_conda_else_branch_chirho")


def test_condu_committed_choice_chirho():
    """condu takes only first solution from first successful clause."""
    x_chirho = VarChirho(0)

    # Normal conde would give 3 solutions; condu gives 1
    goal_chirho = condu_goal_chirho(
        eq_goal_chirho(x_chirho, 1),
        eq_goal_chirho(x_chirho, 2),
        eq_goal_chirho(x_chirho, 3),
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][x_chirho.id_chirho] == 1
    print("PASS: test_condu_committed_choice_chirho")


def test_condu_skips_failed_chirho():
    """condu skips failing clauses."""
    x_chirho = VarChirho(0)

    # First clause fails (x can't be both 1 and not-1)
    goal_chirho = conj_chirho(
        eq_goal_chirho(x_chirho, 5),
        condu_goal_chirho(
            eq_goal_chirho(x_chirho, 1),  # Fails (x is 5)
            eq_goal_chirho(x_chirho, 5),  # Succeeds
            eq_goal_chirho(x_chirho, 9),  # Not reached
        )
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][x_chirho.id_chirho] == 5
    print("PASS: test_condu_skips_failed_chirho")


def test_diseq_basic_chirho():
    """x != 1 excludes 1 from solutions."""
    x_chirho = VarChirho(0)

    # x != 1 AND (x == 1 OR x == 2)
    goal_chirho = conj_chirho(
        diseq_goal_chirho(x_chirho, 1),
        disj_chirho(
            eq_goal_chirho(x_chirho, 1),
            eq_goal_chirho(x_chirho, 2),
        )
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][x_chirho.id_chirho] == 2
    print("PASS: test_diseq_basic_chirho")


def test_diseq_ground_equal_fails_chirho():
    """Disequality on already-equal ground terms fails."""
    x_chirho = VarChirho(0)

    # x == 1, then x != 1 (should fail)
    goal_chirho = conj_chirho(
        eq_goal_chirho(x_chirho, 1),
        diseq_goal_chirho(x_chirho, 1),
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 0
    print("PASS: test_diseq_ground_equal_fails_chirho")


def test_diseq_different_succeeds_chirho():
    """Disequality on different ground terms succeeds."""
    x_chirho = VarChirho(0)

    # x == 1, then x != 2 (should succeed)
    goal_chirho = conj_chirho(
        eq_goal_chirho(x_chirho, 1),
        diseq_goal_chirho(x_chirho, 2),
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][x_chirho.id_chirho] == 1
    print("PASS: test_diseq_different_succeeds_chirho")


def test_diseq_deferred_chirho():
    """Disequality constraint is checked when variables become ground."""
    x_chirho = VarChirho(0)
    y_chirho = VarChirho(1)

    # x != y, then x == 1, then y == 1 (should fail)
    goal_chirho = conj_all_chirho(
        diseq_goal_chirho(x_chirho, y_chirho),
        eq_goal_chirho(x_chirho, 1),
        eq_goal_chirho(y_chirho, 1),
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho, y_chirho])
    assert len(results_chirho) == 0
    print("PASS: test_diseq_deferred_chirho")


def test_diseq_different_vars_succeed_chirho():
    """Variables with different values satisfy disequality."""
    x_chirho = VarChirho(0)
    y_chirho = VarChirho(1)

    # x != y, then x == 1, then y == 2 (should succeed)
    goal_chirho = conj_all_chirho(
        diseq_goal_chirho(x_chirho, y_chirho),
        eq_goal_chirho(x_chirho, 1),
        eq_goal_chirho(y_chirho, 2),
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho, y_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][x_chirho.id_chirho] == 1
    assert results_chirho[0][y_chirho.id_chirho] == 2
    print("PASS: test_diseq_different_vars_succeed_chirho")


def test_project_basic_chirho():
    """project accesses current variable values."""
    x_chirho = VarChirho(0)
    y_chirho = VarChirho(1)

    # x == 3, then y = x (via project)
    goal_chirho = conj_chirho(
        eq_goal_chirho(x_chirho, 3),
        project_goal_chirho(
            [x_chirho],
            lambda vals: eq_goal_chirho(y_chirho, vals[0])
        )
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho, y_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][y_chirho.id_chirho] == 3
    print("PASS: test_project_basic_chirho")


def test_project_arithmetic_chirho():
    """project enables arithmetic operations."""
    x_chirho = VarChirho(0)
    y_chirho = VarChirho(1)
    z_chirho = VarChirho(2)

    # x == 2, y == 3, z == x + y
    def add_goal_chirho(vals_chirho):
        x_val_chirho = vals_chirho[0]
        y_val_chirho = vals_chirho[1]
        if isinstance(x_val_chirho, int) and isinstance(y_val_chirho, int):
            return eq_goal_chirho(z_chirho, x_val_chirho + y_val_chirho)
        return fail_goal_chirho()

    goal_chirho = conj_all_chirho(
        eq_goal_chirho(x_chirho, 2),
        eq_goal_chirho(y_chirho, 3),
        project_goal_chirho([x_chirho, y_chirho], add_goal_chirho)
    )

    results_chirho = run_chirho(5, goal_chirho, [z_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][z_chirho.id_chirho] == 5
    print("PASS: test_project_arithmetic_chirho")


def test_succeed_goal_chirho():
    """succeed always succeeds."""
    x_chirho = VarChirho(0)

    goal_chirho = conj_chirho(
        eq_goal_chirho(x_chirho, 42),
        succeed_goal_chirho()
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 1
    assert results_chirho[0][x_chirho.id_chirho] == 42
    print("PASS: test_succeed_goal_chirho")


def test_fail_goal_chirho():
    """fail always fails."""
    x_chirho = VarChirho(0)

    goal_chirho = conj_chirho(
        eq_goal_chirho(x_chirho, 42),
        fail_goal_chirho()
    )

    results_chirho = run_chirho(5, goal_chirho, [x_chirho])
    assert len(results_chirho) == 0
    print("PASS: test_fail_goal_chirho")


def run_all_tests_chirho():
    """Run all semantic tests."""
    print("=" * 60)
    print("Full miniKanren Semantics Tests ☧")
    print("=" * 60)
    print()

    test_negation_basic_chirho()
    test_negation_fails_when_succeeds_chirho()
    test_conda_basic_chirho()
    test_conda_else_branch_chirho()
    test_condu_committed_choice_chirho()
    test_condu_skips_failed_chirho()
    test_diseq_basic_chirho()
    test_diseq_ground_equal_fails_chirho()
    test_diseq_different_succeeds_chirho()
    test_diseq_deferred_chirho()
    test_diseq_different_vars_succeed_chirho()
    test_project_basic_chirho()
    test_project_arithmetic_chirho()
    test_succeed_goal_chirho()
    test_fail_goal_chirho()

    print()
    print("=" * 60)
    print("ALL TESTS PASSED!")
    print("=" * 60)


if __name__ == "__main__":
    run_all_tests_chirho()
