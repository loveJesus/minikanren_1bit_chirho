# PoC 4: Differentiable Logic / Neural-Symbolic ☧

## Goal

Train relation weights from data by flowing gradients through logic programs, connecting to the hot neural-symbolic AI research area.

---

## Background: Why This Matters

Neural-symbolic AI is one of the hottest research areas:
- **Scallop** (PLDI 2023): Differentiable Datalog
- **DeepProbLog**: Neural predicates in ProbLog
- **NeurASP**: Neural network + answer set programming
- **Logic Tensor Networks**: Differentiable first-order logic

**Our contribution:** Differentiable miniKanren with hardware acceleration.

---

## Core Idea: Soft Logic + Gradients

### Boolean → Probabilistic Relaxation

| Boolean | Soft (Probabilistic) |
|---------|---------------------|
| `true` | `1.0` |
| `false` | `0.0` |
| `AND(a, b)` | `a * b` |
| `OR(a, b)` | `a + b - a*b` |
| `NOT(a)` | `1 - a` |
| `x == y` | `exp(-|x - y| / τ)` (soft equality) |

### Gradient Flow

```
Loss = (predicted_prob - target)²

∂Loss/∂weight = ∂Loss/∂pred × ∂pred/∂weight
              = 2(pred - target) × (soft_logic_gradient)
```

---

## Application 1: Learn Relation Weights

**Problem:** Given I/O examples, learn which tuples belong in a relation.

```python
# Learnable parent relation
parent_chirho = LearnableRelationChirho([
    ("alice", "bob"),    # weight[0]
    ("bob", "charlie"),  # weight[1]
    ("alice", "charlie"),# weight[2] (wrong - should be grandparent)
])

# Training data: grandparent(alice, charlie) = true
# This should increase weights for paths through parent

for epoch in range(100):
    # Query: grandparent(alice, charlie)?
    # = exists X. parent(alice, X) AND parent(X, charlie)

    prob = soft_query_chirho("grandparent", ("alice", "charlie"))
    loss = (prob - 1.0) ** 2

    # Backprop updates weights
    parent_chirho.backward(loss)
```

### Expected Outcome

After training:
- `weight["alice", "bob"]` ↑ (used in valid path)
- `weight["bob", "charlie"]` ↑ (used in valid path)
- `weight["alice", "charlie"]` ↓ (not a parent relation)

---

## Application 2: Neural Predicate Integration

**Problem:** Combine neural network outputs with logic reasoning.

```rust
// Neural network outputs probability that image contains a cat
let cat_prob_chirho = neural_net_chirho.forward(image_chirho); // e.g., 0.95

// Logic rule: cute(X) :- cat(X), small(X)
let cute_prob_chirho = soft_and_chirho(
    cat_prob_chirho,
    small_prob_chirho,  // From another network or predicate
);

// Backprop through entire system
let loss_chirho = (cute_prob_chirho - label_chirho).powi(2);
loss_chirho.backward(); // Gradients flow to neural net!
```

---

## Application 3: Program Synthesis with Soft Constraints

**Problem:** Synthesize programs that approximately satisfy examples.

```rust
// Soft example satisfaction
fn soft_satisfies_chirho(
    program_chirho: &ProgramChirho,
    examples_chirho: &[(Input, Output)],
    temp_chirho: f64,
) -> f64 {
    let mut total_prob_chirho = 1.0;

    for (input_chirho, expected_chirho) in examples_chirho {
        let actual_chirho = eval_chirho(program_chirho, input_chirho);
        let match_prob_chirho = soft_eq_chirho(actual_chirho, expected_chirho, temp_chirho);
        total_prob_chirho *= match_prob_chirho;
    }

    total_prob_chirho
}

// Gradient-guided search
// Instead of enumerate-and-test, follow gradient toward better programs
```

---

## Implementation Plan

### Step 1: Differentiable Semiring (Already Exists)

We already have `DiffProbChirho` in `rust_chirho/src/diff_semiring_chirho.rs`:

```rust
pub struct DiffProbChirho {
    pub value_chirho: f64,
    pub grad_chirho: f64,
}
```

### Step 2: Learnable Relations

```rust
// rust_chirho/src/learn_chirho.rs

pub struct LearnableRelationChirho {
    tuples_chirho: Vec<(TermIdChirho, TermIdChirho)>,
    log_weights_chirho: Vec<f64>,  // Learnable parameters
    lr_chirho: f64,
}

impl LearnableRelationChirho {
    pub fn soft_query_chirho(
        &self,
        pattern_chirho: (TermIdChirho, TermIdChirho),
        store_chirho: &TermStoreChirho,
    ) -> DiffProbChirho {
        let mut total_chirho = DiffProbChirho::zero_chirho();

        for (i_chirho, tuple_chirho) in self.tuples_chirho.iter().enumerate() {
            let match_prob_chirho = soft_unify_chirho(pattern_chirho, *tuple_chirho, store_chirho);
            let weight_chirho = self.weight_chirho(i_chirho);
            total_chirho = total_chirho.or_chirho(&match_prob_chirho.and_chirho(&weight_chirho));
        }

        total_chirho
    }

    pub fn update_chirho(&mut self, grad_chirho: f64) {
        // SGD update
        for w_chirho in &mut self.log_weights_chirho {
            *w_chirho -= self.lr_chirho * grad_chirho;
        }
    }
}
```

### Step 3: Temperature Annealing

```rust
// rust_chirho/src/anneal_chirho.rs

pub struct AnnealingScheduleChirho {
    initial_temp_chirho: f64,
    final_temp_chirho: f64,
    total_steps_chirho: usize,
}

impl AnnealingScheduleChirho {
    pub fn temp_at_chirho(&self, step_chirho: usize) -> f64 {
        let progress_chirho = step_chirho as f64 / self.total_steps_chirho as f64;
        self.initial_temp_chirho * (self.final_temp_chirho / self.initial_temp_chirho).powf(progress_chirho)
    }
}

// Start soft (high temp) → end hard (low temp)
// Allows gradients to flow early, converges to discrete at end
```

### Step 4: Gumbel-Softmax for Discrete Choices

```rust
// rust_chirho/src/gumbel_chirho.rs

pub fn gumbel_softmax_chirho(
    logits_chirho: &[f64],
    temp_chirho: f64,
    rng_chirho: &mut impl Rng,
) -> Vec<f64> {
    let gumbels_chirho: Vec<f64> = logits_chirho.iter()
        .map(|_| -(-rng_chirho.gen::<f64>().ln()).ln())
        .collect();

    let scaled_chirho: Vec<f64> = logits_chirho.iter()
        .zip(&gumbels_chirho)
        .map(|(l_chirho, g_chirho)| (l_chirho + g_chirho) / temp_chirho)
        .collect();

    softmax_chirho(&scaled_chirho)
}

// Use for differentiable branch selection in conde
```

---

## Demo Application: Learn Family Relations

```rust
// demo_learn_family_chirho.rs

fn main() {
    // Raw facts (some may be noisy/wrong)
    let mut parent_chirho = LearnableRelationChirho::new_chirho(vec![
        ("alice", "bob"),
        ("bob", "charlie"),
        ("alice", "charlie"),  // WRONG - will be downweighted
        ("eve", "bob"),        // WRONG - will be downweighted
    ]);

    // Training: grandparent(alice, charlie) = true
    // grandparent(X, Z) :- parent(X, Y), parent(Y, Z)

    let examples_chirho = vec![
        (("alice", "charlie"), 1.0),  // alice IS grandparent of charlie
        (("eve", "charlie"), 0.0),    // eve is NOT grandparent of charlie
    ];

    for epoch_chirho in 0..1000 {
        let mut total_loss_chirho = 0.0;

        for ((x_chirho, z_chirho), target_chirho) in &examples_chirho {
            // Soft query: grandparent(x, z)?
            let prob_chirho = soft_grandparent_chirho(&parent_chirho, x_chirho, z_chirho);
            let loss_chirho = (prob_chirho.value_chirho - target_chirho).powi(2);
            total_loss_chirho += loss_chirho;

            // Backprop
            let grad_chirho = 2.0 * (prob_chirho.value_chirho - target_chirho);
            parent_chirho.backward_chirho(grad_chirho);
        }

        if epoch_chirho % 100 == 0 {
            println!("Epoch {}: loss = {:.4}", epoch_chirho, total_loss_chirho);
            parent_chirho.print_weights_chirho();
        }
    }

    // Expected: alice→bob and bob→charlie have high weights
    // alice→charlie and eve→bob have low weights
}
```

---

## Files to Create

| File | Purpose |
|------|---------|
| `rust_chirho/src/learn_chirho.rs` | Learnable relations |
| `rust_chirho/src/anneal_chirho.rs` | Temperature annealing |
| `rust_chirho/src/gumbel_chirho.rs` | Gumbel-softmax |
| `rust_chirho/examples/learn_family_chirho.rs` | Family relations demo |
| `rust_chirho/examples/neural_predicate_chirho.rs` | Neural integration demo |
| `rust_chirho/benches/differentiable_bench_chirho.rs` | Performance benchmarks |

---

## Expected Results

| Metric | Target |
|--------|--------|
| Training convergence | <1000 iterations |
| Final accuracy | >95% on held-out examples |
| Gradient computation | <1ms per example |
| Comparison with Scallop | Competitive or faster |

---

## Paper Positioning

This PoC positions our work in the neural-symbolic AI conversation:

> "We show that the 1-bit matrix representation of miniKanren naturally extends to differentiable logic, enabling gradient-based learning of relation weights. Combined with hardware acceleration, this provides a practical path to scalable neural-symbolic systems."

---

*Soli Deo Gloria* ☧
