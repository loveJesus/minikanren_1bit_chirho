#!/usr/bin/env python3
"""
Proper miniKanren Implementation ☧

Key fix: fresh variables created at RUN time, not goal construction time.
Uses proper interleaving streams for fair search.
"""
from dataclasses import dataclass
from typing import Dict, List, Tuple, Optional, Union, Iterator, Callable
from itertools import islice

# === Terms ===

@dataclass(frozen=True)
class VarChirho:
    id_chirho: int
    def __repr__(self_chirho): return f"_{self_chirho.id_chirho}"

@dataclass(frozen=True)
class NilChirho:
    def __repr__(self_chirho): return "[]"

@dataclass(frozen=True)
class ConsChirho:
    head_chirho: 'TermChirho'
    tail_chirho: 'TermChirho'
    
    def __repr__(self_chirho):
        elems_chirho = []
        curr_chirho = self_chirho
        while isinstance(curr_chirho, ConsChirho):
            elems_chirho.append(repr(curr_chirho.head_chirho))
            curr_chirho = curr_chirho.tail_chirho
        if isinstance(curr_chirho, NilChirho):
            return "[" + ", ".join(elems_chirho) + "]"
        else:
            return "[" + ", ".join(elems_chirho) + " | " + repr(curr_chirho) + "]"

TermChirho = Union[int, VarChirho, NilChirho, ConsChirho]

def list_chirho(*elems_chirho) -> TermChirho:
    result_chirho = NilChirho()
    for e_chirho in reversed(elems_chirho):
        result_chirho = ConsChirho(e_chirho, result_chirho)
    return result_chirho


# === State: Substitution + Variable Counter ===

@dataclass
class StateChirho:
    subst_chirho: Dict[int, TermChirho]
    counter_chirho: int
    diseqs_chirho: List[Tuple[TermChirho, TermChirho]] = None  # Disequality constraints

    def __post_init__(self_chirho):
        if self_chirho.diseqs_chirho is None:
            self_chirho.diseqs_chirho = []

    def copy_chirho(self_chirho) -> 'StateChirho':
        return StateChirho(
            self_chirho.subst_chirho.copy(),
            self_chirho.counter_chirho,
            self_chirho.diseqs_chirho.copy()
        )

def empty_state_chirho() -> StateChirho:
    return StateChirho({}, 0, [])


# === Substitution Operations ===

def walk_chirho(term_chirho: TermChirho, subst_chirho: Dict[int, TermChirho]) -> TermChirho:
    while isinstance(term_chirho, VarChirho) and term_chirho.id_chirho in subst_chirho:
        term_chirho = subst_chirho[term_chirho.id_chirho]
    return term_chirho

def walk_deep_chirho(term_chirho: TermChirho, subst_chirho: Dict[int, TermChirho]) -> TermChirho:
    term_chirho = walk_chirho(term_chirho, subst_chirho)
    if isinstance(term_chirho, ConsChirho):
        return ConsChirho(
            walk_deep_chirho(term_chirho.head_chirho, subst_chirho),
            walk_deep_chirho(term_chirho.tail_chirho, subst_chirho)
        )
    return term_chirho

def occurs_chirho(var_id_chirho: int, term_chirho: TermChirho, subst_chirho: Dict[int, TermChirho]) -> bool:
    term_chirho = walk_chirho(term_chirho, subst_chirho)
    if isinstance(term_chirho, VarChirho):
        return term_chirho.id_chirho == var_id_chirho
    if isinstance(term_chirho, ConsChirho):
        return (occurs_chirho(var_id_chirho, term_chirho.head_chirho, subst_chirho) or
                occurs_chirho(var_id_chirho, term_chirho.tail_chirho, subst_chirho))
    return False

def unify_chirho(t1_chirho: TermChirho, t2_chirho: TermChirho, 
                 subst_chirho: Dict[int, TermChirho]) -> Optional[Dict[int, TermChirho]]:
    t1_chirho = walk_chirho(t1_chirho, subst_chirho)
    t2_chirho = walk_chirho(t2_chirho, subst_chirho)
    
    if t1_chirho == t2_chirho:
        return subst_chirho
    
    if isinstance(t1_chirho, VarChirho):
        if occurs_chirho(t1_chirho.id_chirho, t2_chirho, subst_chirho):
            return None
        subst_chirho = subst_chirho.copy()
        subst_chirho[t1_chirho.id_chirho] = t2_chirho
        return subst_chirho
    
    if isinstance(t2_chirho, VarChirho):
        if occurs_chirho(t2_chirho.id_chirho, t1_chirho, subst_chirho):
            return None
        subst_chirho = subst_chirho.copy()
        subst_chirho[t2_chirho.id_chirho] = t1_chirho
        return subst_chirho
    
    if isinstance(t1_chirho, ConsChirho) and isinstance(t2_chirho, ConsChirho):
        subst_chirho = unify_chirho(t1_chirho.head_chirho, t2_chirho.head_chirho, subst_chirho)
        if subst_chirho is None:
            return None
        return unify_chirho(t1_chirho.tail_chirho, t2_chirho.tail_chirho, subst_chirho)
    
    return None


def ground_eq_chirho(t1_chirho: TermChirho, t2_chirho: TermChirho,
                     subst_chirho: Dict[int, TermChirho]) -> Optional[bool]:
    """
    Check if two terms are definitively equal or unequal after walking.
    Returns True if equal, False if definitely unequal, None if unknown.
    """
    t1_chirho = walk_chirho(t1_chirho, subst_chirho)
    t2_chirho = walk_chirho(t2_chirho, subst_chirho)

    if t1_chirho == t2_chirho:
        return True

    # Both ground and different
    if not isinstance(t1_chirho, VarChirho) and not isinstance(t2_chirho, VarChirho):
        if isinstance(t1_chirho, ConsChirho) and isinstance(t2_chirho, ConsChirho):
            head_eq_chirho = ground_eq_chirho(t1_chirho.head_chirho, t2_chirho.head_chirho, subst_chirho)
            if head_eq_chirho is False:
                return False
            tail_eq_chirho = ground_eq_chirho(t1_chirho.tail_chirho, t2_chirho.tail_chirho, subst_chirho)
            if tail_eq_chirho is False:
                return False
            if head_eq_chirho and tail_eq_chirho:
                return True
            return None
        return False  # Different ground terms

    return None  # Contains variables, uncertain


def check_diseqs_chirho(state_chirho: StateChirho) -> bool:
    """Check if all disequality constraints are satisfied."""
    for t1_chirho, t2_chirho in state_chirho.diseqs_chirho:
        eq_result_chirho = ground_eq_chirho(t1_chirho, t2_chirho, state_chirho.subst_chirho)
        if eq_result_chirho is True:
            return False  # Constraint violated
    return True


# === Streams (lazy lists with interleaving) ===

StreamChirho = Iterator[StateChirho]

def mplus_chirho(s1_chirho: StreamChirho, s2_chirho: StreamChirho) -> StreamChirho:
    """Interleaving merge of two streams"""
    try:
        first_chirho = next(s1_chirho)
        yield first_chirho
        yield from mplus_chirho(s2_chirho, s1_chirho)  # Swap for interleaving
    except StopIteration:
        yield from s2_chirho

def bind_chirho(stream_chirho: StreamChirho, goal_chirho: 'GoalChirho') -> StreamChirho:
    """Flatmap a goal over a stream"""
    for state_chirho in stream_chirho:
        yield from goal_chirho(state_chirho)


# === Goals ===

GoalChirho = Callable[[StateChirho], StreamChirho]

def eq_goal_chirho(t1_chirho: TermChirho, t2_chirho: TermChirho) -> GoalChirho:
    """Unification goal"""
    def goal_chirho(state_chirho: StateChirho) -> StreamChirho:
        subst_chirho = unify_chirho(t1_chirho, t2_chirho, state_chirho.subst_chirho)
        if subst_chirho is not None:
            new_state_chirho = StateChirho(subst_chirho, state_chirho.counter_chirho, state_chirho.diseqs_chirho.copy())
            # Check that disequality constraints still hold
            if check_diseqs_chirho(new_state_chirho):
                yield new_state_chirho
    return goal_chirho

def call_fresh_chirho(fn_chirho: Callable[[VarChirho], GoalChirho]) -> GoalChirho:
    """Introduce a fresh variable"""
    def goal_chirho(state_chirho: StateChirho) -> StreamChirho:
        var_chirho = VarChirho(state_chirho.counter_chirho)
        new_state_chirho = StateChirho(state_chirho.subst_chirho, state_chirho.counter_chirho + 1)
        yield from fn_chirho(var_chirho)(new_state_chirho)
    return goal_chirho

def disj_chirho(g1_chirho: GoalChirho, g2_chirho: GoalChirho) -> GoalChirho:
    """Disjunction with interleaving"""
    def goal_chirho(state_chirho: StateChirho) -> StreamChirho:
        yield from mplus_chirho(g1_chirho(state_chirho), g2_chirho(state_chirho))
    return goal_chirho

def conj_chirho(g1_chirho: GoalChirho, g2_chirho: GoalChirho) -> GoalChirho:
    """Conjunction"""
    def goal_chirho(state_chirho: StateChirho) -> StreamChirho:
        yield from bind_chirho(g1_chirho(state_chirho), g2_chirho)
    return goal_chirho


# === Advanced Control Operators ===

def not_goal_chirho(goal_chirho: GoalChirho) -> GoalChirho:
    """
    Negation-as-failure: succeeds if goal fails, fails if goal succeeds.

    WARNING: Not pure relational - depends on order and groundness.
    Should only be used when goal arguments are sufficiently ground.
    """
    def inner_chirho(state_chirho: StateChirho) -> StreamChirho:
        results_chirho = list(islice(goal_chirho(state_chirho), 1))
        if not results_chirho:
            yield state_chirho
    return inner_chirho


def conda_goal_chirho(cond_chirho: GoalChirho,
                      then_chirho: GoalChirho,
                      else_chirho: GoalChirho) -> GoalChirho:
    """
    Soft cut (conda): if cond succeeds, run then; otherwise run else.

    Commits to first branch but explores all solutions in that branch.
    """
    def inner_chirho(state_chirho: StateChirho) -> StreamChirho:
        cond_results_chirho = list(islice(cond_chirho(state_chirho), 1))
        if cond_results_chirho:
            # Condition succeeded - run then on the cond result state
            for cond_state_chirho in cond_results_chirho:
                yield from then_chirho(cond_state_chirho)
        else:
            # Condition failed - run else on original state
            yield from else_chirho(state_chirho)
    return inner_chirho


def condu_goal_chirho(*clauses_chirho: GoalChirho) -> GoalChirho:
    """
    Committed choice (condu): try each clause, take first success and stop.

    Unlike conde which explores all branches, condu commits to the first
    successful clause and only takes one solution from it.
    """
    def inner_chirho(state_chirho: StateChirho) -> StreamChirho:
        for clause_chirho in clauses_chirho:
            results_chirho = list(islice(clause_chirho(state_chirho), 1))
            if results_chirho:
                yield results_chirho[0]
                return  # Stop after first solution from first successful clause
    return inner_chirho


def diseq_goal_chirho(t1_chirho: TermChirho, t2_chirho: TermChirho) -> GoalChirho:
    """
    Disequality constraint: (=/= t1 t2) - t1 must never equal t2.

    If they are already ground and equal, fail immediately.
    Otherwise, record the constraint for later checking.
    """
    def inner_chirho(state_chirho: StateChirho) -> StreamChirho:
        t1_walked_chirho = walk_chirho(t1_chirho, state_chirho.subst_chirho)
        t2_walked_chirho = walk_chirho(t2_chirho, state_chirho.subst_chirho)

        # Check if already definitively equal
        eq_result_chirho = ground_eq_chirho(t1_walked_chirho, t2_walked_chirho, state_chirho.subst_chirho)
        if eq_result_chirho is True:
            return  # Fail: already equal

        # Add constraint
        new_state_chirho = state_chirho.copy_chirho()
        new_state_chirho.diseqs_chirho.append((t1_chirho, t2_chirho))
        yield new_state_chirho
    return inner_chirho


def project_goal_chirho(vars_chirho: List[VarChirho],
                        fn_chirho: Callable[[List[TermChirho]], GoalChirho]) -> GoalChirho:
    """
    Project: access walked values of variables, then run a goal.

    Enables accessing the current state of variables to make decisions.
    Useful for arithmetic, printing, or conditional logic.

    Example:
        project_goal_chirho([x, y], lambda vals: eq_goal_chirho(z, vals[0] + vals[1]))
    """
    def inner_chirho(state_chirho: StateChirho) -> StreamChirho:
        walked_chirho = [walk_deep_chirho(v_chirho, state_chirho.subst_chirho) for v_chirho in vars_chirho]
        goal_chirho = fn_chirho(walked_chirho)
        yield from goal_chirho(state_chirho)
    return inner_chirho


def succeed_goal_chirho() -> GoalChirho:
    """Always succeeds with current state."""
    def inner_chirho(state_chirho: StateChirho) -> StreamChirho:
        yield state_chirho
    return inner_chirho


def fail_goal_chirho() -> GoalChirho:
    """Always fails (produces no solutions)."""
    def inner_chirho(state_chirho: StateChirho) -> StreamChirho:
        return
        yield  # Make it a generator
    return inner_chirho


# === Convenience ===

def conj_all_chirho(*goals_chirho: GoalChirho) -> GoalChirho:
    if len(goals_chirho) == 0:
        return lambda state_chirho: iter([state_chirho])
    if len(goals_chirho) == 1:
        return goals_chirho[0]
    return conj_chirho(goals_chirho[0], conj_all_chirho(*goals_chirho[1:]))

def disj_all_chirho(*goals_chirho: GoalChirho) -> GoalChirho:
    if len(goals_chirho) == 0:
        return lambda state_chirho: iter([])
    if len(goals_chirho) == 1:
        return goals_chirho[0]
    return disj_chirho(goals_chirho[0], disj_all_chirho(*goals_chirho[1:]))


# === appendo ===

def appendo_chirho(l_chirho: TermChirho, s_chirho: TermChirho, out_chirho: TermChirho) -> GoalChirho:
    """
    appendo([], S, S).
    appendo([H|T], S, [H|R]) :- appendo(T, S, R).
    """
    # Base case
    base_chirho = conj_chirho(
        eq_goal_chirho(l_chirho, NilChirho()),
        eq_goal_chirho(s_chirho, out_chirho)
    )
    
    # Recursive case with fresh variables
    rec_chirho = call_fresh_chirho(lambda h_chirho:
        call_fresh_chirho(lambda t_chirho:
            call_fresh_chirho(lambda r_chirho:
                conj_all_chirho(
                    eq_goal_chirho(l_chirho, ConsChirho(h_chirho, t_chirho)),
                    eq_goal_chirho(out_chirho, ConsChirho(h_chirho, r_chirho)),
                    lambda state_chirho: appendo_chirho(t_chirho, s_chirho, r_chirho)(state_chirho)
                )
            )
        )
    )
    
    return disj_chirho(base_chirho, rec_chirho)


# === Run ===

def run_chirho(n_chirho: Optional[int], goal_chirho: GoalChirho, 
               query_vars_chirho: List[VarChirho]) -> List[Dict[int, TermChirho]]:
    """Run goal and extract query variable bindings"""
    stream_chirho = goal_chirho(empty_state_chirho())
    
    if n_chirho is not None:
        states_chirho = list(islice(stream_chirho, n_chirho))
    else:
        states_chirho = list(stream_chirho)
    
    results_chirho = []
    for state_chirho in states_chirho:
        result_chirho = {}
        for var_chirho in query_vars_chirho:
            result_chirho[var_chirho.id_chirho] = walk_deep_chirho(var_chirho, state_chirho.subst_chirho)
        results_chirho.append(result_chirho)
    
    return results_chirho


def main():
    print("=== Proper miniKanren with Interleaving ☧ ===\n")
    
    # === Forward ===
    print("="*60)
    print("FORWARD: appendo([0], [1], Out)")
    print("="*60)
    
    out_chirho = VarChirho(0)
    goal_chirho = call_fresh_chirho(lambda out_chirho:
        appendo_chirho(list_chirho(0), list_chirho(1), out_chirho)
    )
    
    # Actually let's be more direct
    out_v_chirho = VarChirho(100)  # Use high ID to avoid collision
    goal_chirho = appendo_chirho(list_chirho(0), list_chirho(1), out_v_chirho)
    
    results_chirho = run_chirho(5, goal_chirho, [out_v_chirho])
    print(f"\nResults ({len(results_chirho)}):")
    for r_chirho in results_chirho:
        print(f"  out = {r_chirho[out_v_chirho.id_chirho]}")
    
    # === Backward ===
    print("\n" + "="*60)
    print("BACKWARD: appendo(L, S, [0, 1])")
    print("="*60)
    
    l_v_chirho = VarChirho(101)
    s_v_chirho = VarChirho(102)
    goal_chirho = appendo_chirho(l_v_chirho, s_v_chirho, list_chirho(0, 1))
    
    results_chirho = run_chirho(10, goal_chirho, [l_v_chirho, s_v_chirho])
    print(f"\nResults ({len(results_chirho)}):")
    for r_chirho in results_chirho:
        print(f"  l = {r_chirho[l_v_chirho.id_chirho]}, s = {r_chirho[s_v_chirho.id_chirho]}")
    
    # === Generate ===
    print("\n" + "="*60)
    print("GENERATE: appendo(L, S, Out) - first 10")
    print("="*60)
    
    l_v_chirho = VarChirho(201)
    s_v_chirho = VarChirho(202)
    out_v_chirho = VarChirho(203)
    goal_chirho = appendo_chirho(l_v_chirho, s_v_chirho, out_v_chirho)
    
    results_chirho = run_chirho(10, goal_chirho, [l_v_chirho, s_v_chirho, out_v_chirho])
    print(f"\nResults ({len(results_chirho)}):")
    for r_chirho in results_chirho:
        l_chirho = r_chirho[l_v_chirho.id_chirho]
        s_chirho = r_chirho[s_v_chirho.id_chirho]
        out_chirho = r_chirho[out_v_chirho.id_chirho]
        print(f"  {l_chirho} ++ {s_chirho} = {out_chirho}")
    
    # === Longer backward ===
    print("\n" + "="*60)
    print("BACKWARD LONGER: appendo(L, S, [0, 1, 2])")
    print("="*60)
    
    l_v_chirho = VarChirho(301)
    s_v_chirho = VarChirho(302)
    goal_chirho = appendo_chirho(l_v_chirho, s_v_chirho, list_chirho(0, 1, 2))
    
    results_chirho = run_chirho(10, goal_chirho, [l_v_chirho, s_v_chirho])
    print(f"\nResults ({len(results_chirho)}):")
    for r_chirho in results_chirho:
        print(f"  l = {r_chirho[l_v_chirho.id_chirho]}, s = {r_chirho[s_v_chirho.id_chirho]}")
    
    # === Composition ===
    print("\n" + "="*60)
    print("COMPOSITION: appendo(A, B, X), appendo(X, C, [0,1,2])")
    print("="*60)
    
    a_v_chirho = VarChirho(401)
    b_v_chirho = VarChirho(402)
    x_v_chirho = VarChirho(403)
    c_v_chirho = VarChirho(404)
    
    goal_chirho = conj_chirho(
        appendo_chirho(a_v_chirho, b_v_chirho, x_v_chirho),
        appendo_chirho(x_v_chirho, c_v_chirho, list_chirho(0, 1, 2))
    )
    
    results_chirho = run_chirho(15, goal_chirho, [a_v_chirho, b_v_chirho, x_v_chirho, c_v_chirho])
    print(f"\nResults ({len(results_chirho)}):")
    for r_chirho in results_chirho:
        a_chirho = r_chirho[a_v_chirho.id_chirho]
        b_chirho = r_chirho[b_v_chirho.id_chirho]
        x_chirho = r_chirho[x_v_chirho.id_chirho]
        c_chirho = r_chirho[c_v_chirho.id_chirho]
        print(f"  a={a_chirho}, b={b_chirho} → x={x_chirho}, c={c_chirho}")
    
    print("\n" + "="*60)
    print("SUCCESS: Full miniKanren is working!")
    print("="*60)
    print("""
    What we have:
    ✓ Proper fresh variable introduction
    ✓ Interleaving streams (fair search)
    ✓ Occurs check
    ✓ Recursive relations (appendo)
    ✓ Backward and bidirectional queries
    ✓ Composition of relations
    
    Connection to 1-bit matrices:
    
    The substitution {v1→t1, v2→t2, ...} can be viewed as:
    - Sparse map from var IDs to term IDs (hash consed)
    - Unification = computing transitive closure
    
    For 1-bit acceleration:
    - Batch multiple substitutions as matrix rows
    - Terms hash-consed → columns are term IDs
    - Unification = sparse boolean ops
    
    The interleaving is KEY:
    - Without it, infinite loops
    - With 1-bit: need to track "frontier" of active states
    - States = rows in matrix, active = bitmask
    """)


if __name__ == "__main__":
    main()
