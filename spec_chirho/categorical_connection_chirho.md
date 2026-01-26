# Categorical Logic Connection ☧

## Tensor Networks as String Diagrams

Our tensor contraction approach has deep connections to category theory:

### The Mapping

| Our System | Categorical Concept |
|------------|---------------------|
| Term ID domain | Object in category |
| Relation tensor | Morphism |
| Tensor contraction | Composition of morphisms |
| Variable sharing | Wiring in string diagram |
| Domain intersection (AND) | Product/pullback |
| Domain union (OR) | Coproduct/pushout |

### String Diagrams

In monoidal categories, morphisms can be drawn as boxes with wires:
- Inputs = wires on left
- Outputs = wires on right
- Composition = connecting wires

Our relations are exactly this:
```
appendo(L, S, Out) corresponds to:

  L ──┐
      ├──[appendo]──── Out
  S ──┘
```

### Contraction Order as Natural Transformation

The choice of contraction order corresponds to choosing a particular
factorization of a composite morphism. Different orders give the same
semantic result but different computational costs.

This connects to:
- **Coherence theorems**: Different diagrams representing same morphism
- **Mac Lane's pentagon/triangle**: Associativity constraints
- **String diagram rewriting**: Optimization = diagram simplification

### Relevance to Topos Institute Work

The Topos Institute's work on Applied Category Theory includes:
- **Polynomial functors**: Model state machines and dependent types
- **Optics/lenses**: Our hardware optics are categorical lenses
- **Wiring diagrams**: Exactly what we compute with tensor contraction

### Future Work

1. **Formalize the functor**: miniKanren → Tensor Networks
2. **Prove coherence**: Contraction order doesn't affect semantics
3. **Connect to AlgebraicJulia**: Compositional systems modeling

## References

- Selinger, "A Survey of Graphical Languages for Monoidal Categories"
- Fong & Spivak, "An Invitation to Applied Category Theory"
- Topos Institute blog on compositional systems

*Soli Deo Gloria* ☧
