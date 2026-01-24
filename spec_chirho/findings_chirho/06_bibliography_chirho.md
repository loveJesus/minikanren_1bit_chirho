# Bibliography ☧

## Core miniKanren References

### Primary Sources

1. **Friedman, D. P., Byrd, W. E., & Kiselyov, O. (2018).**
   *The Reasoned Schemer* (2nd ed.). MIT Press.
   - Foundational text for miniKanren
   - Introduces streams, fresh, conde, run*

2. **Byrd, W. E. (2009).**
   *Relational Programming in miniKanren: Techniques, Applications, and Implementations.*
   PhD Thesis, Indiana University.
   - Comprehensive treatment of miniKanren internals
   - Tabling and optimization techniques

3. **Hemann, J., & Friedman, D. P. (2013).**
   "μKanren: A Minimal Functional Core for Relational Programming."
   *Workshop on Scheme and Functional Programming*.
   - Minimal 40-line implementation
   - Core semantics distilled

### Extensions and Variants

4. **Rosette (Torlak, E., & Bodik, R., 2014).**
   "A Lightweight Symbolic Virtual Machine for Solver-Aided Host Languages."
   *PLDI*.
   - Symbolic execution meets relational programming
   - Solver-backed constraints

5. **cKanren (Alvis, C. E., et al., 2011).**
   "cKanren: miniKanren with Constraints."
   *Workshop on Scheme and Functional Programming*.
   - Constraint Logic Programming in miniKanren
   - Finite domain constraints

## E-Graphs and Equality Saturation

6. **Willsey, M., et al. (2021).**
   "egg: Fast and Extensible Equality Saturation."
   *POPL*.
   - State-of-the-art e-graph implementation
   - Rebuilding algorithm
   - https://egraphs-good.github.io/

7. **Nelson, G., & Oppen, D. C. (1980).**
   "Fast Decision Procedures Based on Congruence Closure."
   *JACM*.
   - Original congruence closure algorithm
   - Foundation for e-graphs

## Tensor Networks and Quantum Computing

8. **Orús, R. (2014).**
   "A Practical Introduction to Tensor Networks."
   *Annals of Physics*.
   - Accessible introduction to tensor networks
   - Contraction and decomposition

9. **Markov, I. L., & Shi, Y. (2008).**
   "Simulating Quantum Computation by Contracting Tensor Networks."
   *SIAM Journal on Computing*.
   - Tensor network contraction complexity
   - Connection to quantum simulation

10. **Gray, J. (2021).**
    "Hyper-optimized tensor network contraction."
    *Quantum*.
    - State-of-the-art contraction ordering
    - https://github.com/jcmgray/cotengra

## Hardware Acceleration

11. **Calyx: A Language for Hardware Compilers (2021).**
    Nigam, R., et al. *PLDI*.
    - Intermediate representation for hardware
    - https://calyxir.org/

12. **Clash: Functional Hardware Description (2015).**
    Baaij, C. *PhD Thesis, University of Twente*.
    - Haskell to hardware compilation
    - https://clash-lang.org/

13. **BitNet (2023).**
    Wang, H., et al. "BitNet: Scaling 1-bit Transformers."
    - 1-bit neural networks
    - Hardware efficiency of binary operations

## Constraint Satisfaction

14. **Mackworth, A. K. (1977).**
    "Consistency in Networks of Relations."
    *Artificial Intelligence*.
    - Arc consistency algorithm
    - Foundation for constraint propagation

15. **Bessière, C. (2006).**
    "Constraint Propagation."
    *Handbook of Constraint Programming*.
    - Comprehensive treatment of propagation algorithms

## Differentiable Logic Programming

16. **Scallop (Huang, J., et al., 2021).**
    "Scallop: From Probabilistic Deductive Databases to Scalable
    Differentiable Reasoning."
    *NeurIPS*.
    - Differentiable Datalog
    - Provenance semirings
    - https://www.scallop-lang.org/

17. **DeepProbLog (Manhaeve, R., et al., 2018).**
    "DeepProbLog: Neural Probabilistic Logic Programming."
    *NeurIPS*.
    - Integration of neural networks and logic
    - Probabilistic inference

## Hash Consing

18. **Filliâtre, J.-C., & Conchon, S. (2006).**
    "Type-Safe Modular Hash-Consing."
    *ML Workshop*.
    - Efficient hash consing in ML
    - Weak hash tables for GC

19. **Goto, E. (1974).**
    "Monocopy and Associative Algorithms in Extended Lisp."
    University of Tokyo.
    - Original hash consing technique

## SIMD and Parallel Processing

20. **Intel Intrinsics Guide.**
    - AVX2 instruction reference
    - https://www.intel.com/content/www/us/en/docs/intrinsics-guide/

21. **ARM NEON Intrinsics Reference.**
    - ARM SIMD instructions
    - https://developer.arm.com/documentation/

## Related Logic Programming Systems

22. **Warren, D. H. D. (1983).**
    *An Abstract Prolog Instruction Set.*
    SRI Technical Note 309.
    - WAM: Warren Abstract Machine
    - Foundation for Prolog implementation

23. **Mercury (Somogyi, Z., et al., 1996).**
    "The Execution Algorithm of Mercury."
    *Journal of Logic Programming*.
    - Mode and determinism analysis
    - Compilation to efficient code

24. **Datalog (Ceri, S., et al., 1989).**
    "What You Always Wanted to Know About Datalog."
    *IEEE TKDE*.
    - Bottom-up evaluation
    - Materialization and magic sets

## GPU Computing

25. **WebGPU Specification.**
    W3C Working Draft.
    - GPU compute shaders in browsers
    - https://www.w3.org/TR/webgpu/

26. **CUDA C Programming Guide.**
    NVIDIA.
    - GPU programming model
    - https://docs.nvidia.com/cuda/

## Type Inference

27. **Milner, R. (1978).**
    "A Theory of Type Polymorphism in Programming."
    *Journal of Computer and System Sciences*.
    - Algorithm W
    - Hindley-Milner type inference

28. **Pierce, B. C. (2002).**
    *Types and Programming Languages.*
    MIT Press.
    - Comprehensive treatment of type systems
    - Unification and constraint solving

---

## Online Resources

- **miniKanren.org**: http://minikanren.org/
- **egg documentation**: https://docs.rs/egg/
- **Tensor Network**: https://tensornetwork.org/
- **Calyx documentation**: https://docs.calyxir.org/
- **Clash documentation**: https://clash-lang.org/documentation/

---

*Soli Deo Gloria* ☧
