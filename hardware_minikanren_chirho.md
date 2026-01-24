# Hardware miniKanren: Beyond Datalog ☧

## The Gap: Datalog vs Full miniKanren

| Feature | Datalog | miniKanren | Hardware Challenge |
|---------|---------|------------|-------------------|
| Terms | Constants only | Constructors (cons, nil) | Term graphs |
| Unification | Pattern match/join | Full structural unification | Graph matching |
| Occurs check | N/A | Required for soundness | Cycle detection |
| Domains | Finite, predefined | Infinite, lazy | Demand-driven expansion |
| Variables | Grounded by query | Fresh at runtime | Dynamic allocation |
| Search | Bottom-up (seminaive) | Top-down with interleaving | Control flow |
| Negation | Stratified | Via constraints | Requires completion |

**Datalog is easy**: Relations are just Boolean matrices, queries are sparse matmul.

**miniKanren is hard**: We need to handle *structure*, not just membership.

---

## What Makes miniKanren Different

### 1. Terms Have Structure

```scheme
;; Datalog: just symbols
parent(alice, bob).

;; miniKanren: nested structure
(== x (cons 1 (cons 2 (cons 3 nil))))
```

A term like `[1, 2, 3]` is really:
```
cons(1, cons(2, cons(3, nil)))
```

This is a **tree** (or DAG with sharing). Hardware must represent and traverse trees.

### 2. Unification is Graph Matching

```scheme
(== (cons x y) (cons 1 (cons 2 nil)))
;; Results in: x=1, y=[2]
```

This requires:
- Walking both terms in parallel
- Binding variables when encountered
- Propagating bindings through the substitution
- **Failing** if structures don't match

### 3. Occurs Check Prevents Infinite Terms

```scheme
(== x (cons x nil))  ;; x = [x] = [[x]] = [[[x]]] = ...
```

Without occurs check: infinite term, unsound logic.
With occurs check: must detect that `x` appears in `(cons x nil)`.

This is **cycle detection in a graph** — reachability!

### 4. Fresh Variables at Runtime

```scheme
(fresh (x y z)
  (appendo x y z))
```

Each `fresh` creates a NEW variable. We don't know ahead of time how many.

---

## Hardware Architecture for Full miniKanren

### Layer 1: Term Store (Hash Consing)

```
┌─────────────────────────────────────────────────────────┐
│                    TERM MEMORY                          │
├─────────────────────────────────────────────────────────┤
│ ID │ Tag    │ Field1   │ Field2   │ Hash              │
├────┼────────┼──────────┼──────────┼───────────────────┤
│ 0  │ NIL    │ -        │ -        │ 0x0000            │
│ 1  │ INT    │ 1        │ -        │ 0x0001            │
│ 2  │ INT    │ 2        │ -        │ 0x0002            │
│ 3  │ CONS   │ 1 (→INT) │ 0 (→NIL) │ 0xABCD            │
│ 4  │ CONS   │ 2        │ 3        │ 0xEF01            │
│ 5  │ VAR    │ 0        │ -        │ (unique per var)  │
└────┴────────┴──────────┴──────────┴───────────────────┘
```

**Hardware**: Content-addressable memory (CAM) for hash lookup.
- Insert: hash term → check if exists → return ID or allocate
- Lookup: ID → (tag, fields)

**1-bit optimization**: Tag can be 2-3 bits. For small domains, entire term fits in 64 bits.

### Layer 2: Substitution Store

```
┌─────────────────────────────────┐
│      SUBSTITUTION (per world)   │
├─────────────────────────────────┤
│ Var ID │ Bound To (Term ID)    │
├────────┼───────────────────────┤
│ 0      │ 3 (→ [1])             │
│ 1      │ UNBOUND               │
│ 2      │ 0 (→ [])              │
└────────┴───────────────────────┘
```

**Hardware**: Sparse map, or dense array if var count bounded.

**Key operation**: `walk(term_id, subst)` — follow variable chains.

```
walk(var_5, subst):
  if var_5 is bound to term_7:
    return walk(term_7, subst)  // recursive!
  else:
    return var_5
```

**1-bit optimization**: For bounded vars, substitution = bit matrix.
- Row = var ID
- Column = term ID
- Entry = 1 if var could be that term

### Layer 3: Unification Engine

```
UNIFY(t1, t2, subst) → subst' or FAIL

1. t1' = walk(t1, subst)
2. t2' = walk(t2, subst)
3. if t1' == t2': return subst
4. if t1' is VAR:
     if occurs(t1', t2', subst): return FAIL  // occurs check!
     return subst ∪ {t1' → t2'}
5. if t2' is VAR: (symmetric)
6. if t1' and t2' are CONS:
     subst = UNIFY(head(t1'), head(t2'), subst)
     if FAIL: return FAIL
     return UNIFY(tail(t1'), tail(t2'), subst)
7. return FAIL  // different constructors
```

**Hardware challenge**: This is recursive! Options:
- **Stack machine**: Push/pop unification frames
- **Iterative flattening**: Convert to worklist algorithm
- **Parallel tree traversal**: If terms are balanced

### Layer 4: Occurs Check (The Hard Part)

```
occurs(var_id, term, subst) → bool

"Does var_id appear anywhere in term (after walking)?"
```

This is **graph reachability**:
- Nodes = term IDs
- Edges = structural containment (cons → head, cons → tail)
- Query = is var_id reachable from term?

**1-bit approach**:
- Adjacency matrix A[i,j] = 1 if term i contains term j
- Transitive closure A* = A + A² + A³ + ...
- Check A*[term, var_id]

**Hardware**: Boolean matrix multiplication! This IS 1-bit friendly.

```
A* = I ∨ A ∨ (A ∧ A) ∨ (A ∧ A ∧ A) ∨ ...
   = fixpoint of (λX. I ∨ A ∨ (A ∧ X))
```

With n terms, converges in O(log n) iterations of matrix multiply.

### Layer 5: Search State (Parallel Worlds)

Each "world" is a substitution + variable counter. Multiple worlds = multiple rows.

```
┌─────────────────────────────────────────────────────────┐
│                    SEARCH STATE                         │
├─────────────────────────────────────────────────────────┤
│ World │ Var0 │ Var1 │ Var2 │ Counter │ Status         │
├───────┼──────┼──────┼──────┼─────────┼────────────────┤
│ 0     │ [1]  │ ?    │ []   │ 3       │ ACTIVE         │
│ 1     │ []   │ [1]  │ ?    │ 3       │ ACTIVE         │
│ 2     │ -    │ -    │ -    │ -       │ FAILED         │
└───────┴──────┴──────┴──────┴─────────┴────────────────┘
```

**1-bit**: Each cell is a bitmask of possible term IDs.
- Unification = AND of bitmasks
- Disjunction = duplicate row
- Failure = row is all zeros → prune

### Layer 6: Goal Execution

Goals form a tree/DAG:
```
conj(
  appendo(a, b, x),
  conj(
    appendo(x, c, y),
    eq(y, [1,2,3])
  )
)
```

**Execution model options**:

1. **Instruction stream**: Compile goals to opcodes
   ```
   FRESH a
   FRESH b
   FRESH x
   CALL appendo, [a, b, x]
   FRESH c
   FRESH y
   CALL appendo, [x, c, y]
   UNIFY y, [1,2,3]
   ```

2. **Dataflow**: Goals as nodes, data dependencies as edges
   - Fire goal when inputs ready
   - Natural parallelism

3. **Tensor contraction**: Pre-compile relations to tensors
   - Query = contract network
   - Our tabling approach!

---

## The 1-Bit Path: What's Tractable

### ✅ Definitely 1-bit friendly:

1. **Finite domain unification**: Variable domains as bitmasks, unify = AND
2. **Relation lookup**: Sparse tensor slice
3. **Occurs check**: Boolean matrix reachability
4. **World management**: Bitmask of active worlds
5. **Failure pruning**: Check for all-zero rows

### ⚠️ Challenging but possible:

1. **Substitution walk**: Chain following, but bounded depth
2. **Term comparison**: Bitwise if hash-consed
3. **Disjunction**: Row duplication, but exponential blowup

### ❌ Fundamentally hard:

1. **Unbounded fresh variables**: Need dynamic allocation
2. **Infinite term generation**: Must use tabling/modes
3. **Arbitrary recursion depth**: Stack or continuation needed
4. **Fair interleaving**: Control flow, not data flow

---

## Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      miniKanren ASIC/FPGA                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│  │ Term Memory  │    │ Relation     │    │ Search State │     │
│  │ (CAM + RAM)  │◄──►│ Tensors      │◄──►│ (Bit Matrix) │     │
│  │              │    │ (Sparse COO) │    │              │     │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘     │
│         │                   │                   │              │
│         ▼                   ▼                   ▼              │
│  ┌─────────────────────────────────────────────────────┐      │
│  │              UNIFICATION ENGINE                      │      │
│  │  • Parallel walk units (pipelined)                  │      │
│  │  • Occurs check via matrix multiply                  │      │
│  │  • Binding broadcast                                 │      │
│  └─────────────────────────────────────────────────────┘      │
│         │                                                      │
│         ▼                                                      │
│  ┌─────────────────────────────────────────────────────┐      │
│  │              GOAL SCHEDULER                          │      │
│  │  • Mode analysis (static)                           │      │
│  │  • Tabling cache                                    │      │
│  │  • World fork/join                                  │      │
│  └─────────────────────────────────────────────────────┘      │
│         │                                                      │
│         ▼                                                      │
│  ┌─────────────────────────────────────────────────────┐      │
│  │              RESULT EXTRACTION                       │      │
│  │  • Walk substitution to ground terms                │      │
│  │  • Stream out solutions                             │      │
│  └─────────────────────────────────────────────────────┘      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation Roadmap

### Phase 1: Bounded miniKanren (Current)
- [x] Finite domains
- [x] Sparse relation tensors
- [x] Tabling for termination
- [x] Mode-driven goal reordering
- [ ] Proper casing convention

### Phase 2: Occurs Check as Matrix Op
- [ ] Build term containment graph
- [ ] Implement Boolean transitive closure
- [ ] Integrate with unification

### Phase 3: Hardware Prototype
- [ ] Term memory in BRAM
- [ ] Unification state machine
- [ ] Parallel world processing
- [ ] Target: Lattice iCE40 or Xilinx Artix

### Phase 4: Full miniKanren
- [ ] Dynamic variable allocation
- [ ] Continuation-based interleaving
- [ ] Constraint extensions (disequality, absento)

---

## Key Insight

**The tabling work we did is essential for hardware.**

Without tabling/modes, queries can diverge. Hardware needs bounded computation.

```
Tabling + Mode Analysis = Bounded miniKanren ⊂ Full miniKanren
```

This bounded subset is EXACTLY what maps to tensor operations!

---

## References

- Warren's Abstract Machine (WAM) for Prolog compilation
- JANUS: Hardware Prolog machine (1980s)
- Mercury: Mode system for logic programming
- XSB: Tabling implementation
- Tensor networks for quantum simulation (same math!)

---

*Soli Deo Gloria* ☧
