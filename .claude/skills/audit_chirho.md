# Skill: /audit-chirho

Audit the miniKanren 1-bit codebase for integration and framework compliance.

## When to Use

Use this skill to verify the codebase follows the 1-bit logic solver framework:
- After adding new features
- Before commits to ensure quality
- When exploring unfamiliar parts of the codebase

## Instructions

When this skill is invoked, perform these checks:

### 1. Run the Python Audit Script
```bash
python3 audit_integration_chirho.py
```

Report the summary: errors, warnings, and overall status.

### 2. Verify Rust Compiles
```bash
cd rust_chirho && cargo check --all-features
```

### 3. Run Tests
```bash
cd rust_chirho && cargo test
```

### 4. Check Framework Compliance

Verify that code follows the core principles:

**NO POINTER CHASING** - All operations should be:
- Bit-parallel (AND, OR, NOT on bitmasks)
- Fixed-size structures (arrays, not heap allocations in hot paths)
- Cache-friendly (contiguous memory)

**DOMAIN HIERARCHY** - Use the correct domain type:
| Domain | Values | When to Use |
|--------|--------|-------------|
| BitVec64Chirho | ≤64 | Default, fastest |
| Hierarchical4kChirho | ≤4096 | Medium scale |
| SymbolicChirho | ∞ | Infinite domains |
| DiffHierarchical4kChirho | ≤4096 | Learning mode |

**INTEGRATION OVER EXTENSION** - New features should:
- Extend AdaptiveDomainChirho, not create parallel systems
- Use existing semiring abstraction
- Integrate with the core, not sit beside it

**NAMING CONVENTION** - All custom identifiers must end with `_chirho` or `Chirho`:
- Functions: `snake_chirho()`
- Types: `PascalChirho`
- Constants: `UPPER_CHIRHO`

### 5. Report Summary

Output a summary table showing:
- Component status (rust/python/hardware/docs)
- Test count and pass rate
- Any violations of framework rules

## Output Format

```
=== miniKanren 1-Bit Audit ===

Compilation: ✓ PASS
Tests: 215 passing
Naming: 44 warnings (acceptable)
Integration: ✓ All components aligned

Framework Compliance:
- No pointer chasing: ✓
- Domain hierarchy: ✓
- _chirho convention: ✓

Soli Deo Gloria ☧
```
