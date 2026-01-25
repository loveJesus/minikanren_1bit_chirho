#!/usr/bin/env python3
"""
Integration Audit for miniKanren 1-Bit Project

Validates that all components follow the framework rules and stay integrated.

Usage:
    python3 audit_integration_chirho.py [--fix] [--verbose]

Soli Deo Gloria
"""

import os
import re
import sys
import subprocess
from pathlib import Path
from typing import List, Tuple, Dict, Set
from dataclasses import dataclass

# Project root
PROJECT_ROOT_CHIRHO = Path(__file__).parent

# Component mapping: what each layer should contain
COMPONENT_MAP_CHIRHO = {
    "foundation_chirho": {
        "rust": ["terms_chirho.rs", "union_find_chirho.rs", "unify_chirho.rs"],
        "python": ["hashcons_chirho.py", "var_propagation_chirho.py", "unify_bits_chirho.py"],
        "hardware": ["hashcons_chirho.futil", "HashConsChirho.hs"],
    },
    "domains_chirho": {
        "rust": ["hardware_chirho.rs", "hierarchical_chirho.rs", "symbolic_chirho.rs",
                 "diff_hierarchical_chirho.rs", "adaptive_chirho.rs"],
        "python": ["differentiable_chirho.py"],
    },
    "goals_chirho": {
        "rust": ["goals_chirho.rs", "goal_ast_chirho.rs"],
        "python": ["minikanren_proper_chirho.py"],
    },
    "search_chirho": {
        "rust": ["stream_chirho.rs", "tabling_chirho.rs", "constraint_chirho.rs", "contraction_chirho.rs"],
        "python": ["tabling_chirho.py", "tabling_complete_chirho.py", "constraint_prop_chirho.py"],
    },
    "acceleration_chirho": {
        "rust": ["simd_chirho.rs", "gpu_chirho.rs", "optics_hw_chirho.rs"],
        "hardware": ["domain_chirho.futil", "cam_chirho.futil", "MiniKanrenChirho.hs"],
    },
    "semirings_chirho": {
        "rust": ["semiring_chirho.rs", "diff_semiring_chirho.rs", "contraction_semiring_chirho.rs"],
        "python": ["differentiable_chirho.py"],
    },
}

# Naming patterns (case-insensitive for _chirho/_CHIRHO/Chirho)
CHIRHO_PATTERN_CHIRHO = re.compile(r'_chirho|Chirho|_CHIRHO', re.IGNORECASE)
RUST_IDENT_PATTERN_CHIRHO = re.compile(r'(pub\s+)?(fn|struct|enum|type|const|static|trait|impl)\s+([A-Za-z_][A-Za-z0-9_]*)')
PYTHON_IDENT_PATTERN_CHIRHO = re.compile(r'(def|class)\s+([A-Za-z_][A-Za-z0-9_]*)')

# Exceptions - external names we don't control
NAMING_EXCEPTIONS_CHIRHO = {
    # Rust standard library / trait methods
    "new", "default", "from", "into", "clone", "fmt", "eq", "hash", "cmp", "partial_cmp",
    "deref", "drop", "index", "iter", "next", "len", "is_empty", "get", "set", "push", "pop",
    "main", "test", "bench", "run", "build", "parse", "read", "write", "flush", "close",
    "map", "filter", "fold", "collect", "take", "skip", "first", "last", "any", "all",
    # Associated types (Rust trait conventions)
    "Output", "Item", "Error", "Target", "Data", "Key", "Value", "Iter", "IntoIter",
    "Inner", "Outer", "Left", "Right", "Node", "Leaf", "Root",
    # Common short names in algorithms
    "bits", "mask", "size", "count", "depth", "width", "height", "offset", "base",
    "temp", "prev", "curr", "next", "head", "tail", "init", "fini", "start", "stop",
    # Names that might appear in docs/comments
    "definitions", "checks", "unifies", "instead", "returns", "takes", "uses",
    # Common descriptive names
    "values", "uniformity", "representation", "parameters", "arguments", "results",
    # SIMD intrinsic types
    "__m256i", "__m128i", "__m512i",
    # egg crate integration
    "make", "merge", "modify", "analysis", "aliases",
    # Python dunder methods
    "__init__", "__repr__", "__str__", "__eq__", "__hash__", "__len__", "__iter__", "__next__",
    # Traits/impls
    "Display", "Debug", "Clone", "Copy", "Default", "PartialEq", "Eq", "Hash", "Ord", "PartialOrd",
    "Iterator", "IntoIterator", "FromIterator", "Add", "Sub", "Mul", "Div", "BitAnd", "BitOr",
    # Test framework
    "should_panic", "ignore",
}


@dataclass
class AuditResultChirho:
    """Result of an audit check."""
    category_chirho: str
    severity_chirho: str  # "error", "warning", "info"
    message_chirho: str
    file_chirho: str = ""
    line_chirho: int = 0


def find_naming_violations_chirho(verbose_chirho: bool = False) -> List[AuditResultChirho]:
    """Find identifiers missing _chirho suffix."""
    results_chirho: List[AuditResultChirho] = []

    # Check Rust files
    rust_dir_chirho = PROJECT_ROOT_CHIRHO / "rust_chirho" / "src"
    if rust_dir_chirho.exists():
        for rs_file_chirho in rust_dir_chirho.rglob("*.rs"):
            # Skip target directory
            if "target" in str(rs_file_chirho):
                continue

            with open(rs_file_chirho, 'r', encoding='utf-8', errors='ignore') as f_chirho:
                for line_num_chirho, line_chirho in enumerate(f_chirho, 1):
                    for match_chirho in RUST_IDENT_PATTERN_CHIRHO.finditer(line_chirho):
                        ident_chirho = match_chirho.group(3)
                        # Skip exceptions (case-insensitive check)
                        if ident_chirho in NAMING_EXCEPTIONS_CHIRHO or ident_chirho.lower() in NAMING_EXCEPTIONS_CHIRHO:
                            continue
                        # Skip if already has chirho
                        if CHIRHO_PATTERN_CHIRHO.search(ident_chirho):
                            continue
                        # Skip single-letter or very short names
                        if len(ident_chirho) <= 4:
                            continue
                        # Skip all-caps constants (likely already have _CHIRHO)
                        if ident_chirho.isupper():
                            continue
                        # Skip impl blocks for external traits
                        if match_chirho.group(2) == "impl":
                            continue

                        results_chirho.append(AuditResultChirho(
                            category_chirho="naming",
                            severity_chirho="warning",
                            message_chirho=f"Identifier '{ident_chirho}' missing _chirho suffix",
                            file_chirho=str(rs_file_chirho.relative_to(PROJECT_ROOT_CHIRHO)),
                            line_chirho=line_num_chirho,
                        ))

    # Check Python files (excluding venv)
    for py_file_chirho in PROJECT_ROOT_CHIRHO.glob("*.py"):
        with open(py_file_chirho, 'r', encoding='utf-8', errors='ignore') as f_chirho:
            for line_num_chirho, line_chirho in enumerate(f_chirho, 1):
                for match_chirho in PYTHON_IDENT_PATTERN_CHIRHO.finditer(line_chirho):
                    ident_chirho = match_chirho.group(2)
                    if ident_chirho.startswith("__"):
                        continue
                    if CHIRHO_PATTERN_CHIRHO.search(ident_chirho):
                        continue
                    if len(ident_chirho) <= 3:
                        continue

                    results_chirho.append(AuditResultChirho(
                        category_chirho="naming",
                        severity_chirho="warning",
                        message_chirho=f"Identifier '{ident_chirho}' missing _chirho suffix",
                        file_chirho=py_file_chirho.name,
                        line_chirho=line_num_chirho,
                    ))

    return results_chirho


def check_component_presence_chirho() -> List[AuditResultChirho]:
    """Check that expected components exist."""
    results_chirho: List[AuditResultChirho] = []

    for layer_chirho, components_chirho in COMPONENT_MAP_CHIRHO.items():
        for lang_chirho, files_chirho in components_chirho.items():
            for file_chirho in files_chirho:
                if lang_chirho == "rust":
                    # Search in rust_chirho/src
                    found_chirho = list((PROJECT_ROOT_CHIRHO / "rust_chirho" / "src").rglob(file_chirho))
                elif lang_chirho == "python":
                    found_chirho = list(PROJECT_ROOT_CHIRHO.glob(file_chirho))
                    found_chirho.extend(PROJECT_ROOT_CHIRHO.glob(f"*/{file_chirho}"))
                elif lang_chirho == "hardware":
                    found_chirho = list(PROJECT_ROOT_CHIRHO.glob(f"*_chirho/{file_chirho}"))
                else:
                    found_chirho = []

                if not found_chirho:
                    results_chirho.append(AuditResultChirho(
                        category_chirho="component",
                        severity_chirho="info",
                        message_chirho=f"Component '{file_chirho}' not found for layer '{layer_chirho}'",
                    ))

    return results_chirho


def check_docs_sync_chirho() -> List[AuditResultChirho]:
    """Check that documentation is in sync with code."""
    results_chirho: List[AuditResultChirho] = []

    # Check AGENTS.md exists and mentions key modules
    agents_path_chirho = PROJECT_ROOT_CHIRHO / "AGENTS.md"
    if agents_path_chirho.exists():
        with open(agents_path_chirho, 'r') as f_chirho:
            content_chirho = f_chirho.read()

        # Check for key modules
        key_modules_chirho = ["adaptive_chirho", "diff_hierarchical_chirho", "hierarchical_chirho"]
        for module_chirho in key_modules_chirho:
            if module_chirho not in content_chirho:
                results_chirho.append(AuditResultChirho(
                    category_chirho="docs",
                    severity_chirho="warning",
                    message_chirho=f"AGENTS.md may need update for module '{module_chirho}'",
                    file_chirho="AGENTS.md",
                ))

    # Check README.md
    readme_path_chirho = PROJECT_ROOT_CHIRHO / "README.md"
    if readme_path_chirho.exists():
        with open(readme_path_chirho, 'r') as f_chirho:
            content_chirho = f_chirho.read()

        # Should mention domain types
        if "BitVec64" not in content_chirho and "Hierarchical" not in content_chirho:
            results_chirho.append(AuditResultChirho(
                category_chirho="docs",
                severity_chirho="info",
                message_chirho="README.md may need domain type documentation",
                file_chirho="README.md",
            ))

    return results_chirho


def check_rust_compiles_chirho() -> List[AuditResultChirho]:
    """Check that Rust code compiles without errors."""
    results_chirho: List[AuditResultChirho] = []

    rust_dir_chirho = PROJECT_ROOT_CHIRHO / "rust_chirho"
    if rust_dir_chirho.exists():
        try:
            result_chirho = subprocess.run(
                ["cargo", "check", "--all-features"],
                cwd=rust_dir_chirho,
                capture_output=True,
                text=True,
                timeout=120,
            )
            if result_chirho.returncode != 0:
                results_chirho.append(AuditResultChirho(
                    category_chirho="build",
                    severity_chirho="error",
                    message_chirho=f"Rust compilation failed: {result_chirho.stderr[:500]}",
                    file_chirho="rust_chirho/",
                ))
        except subprocess.TimeoutExpired:
            results_chirho.append(AuditResultChirho(
                category_chirho="build",
                severity_chirho="warning",
                message_chirho="Rust compilation timed out",
                file_chirho="rust_chirho/",
            ))
        except FileNotFoundError:
            results_chirho.append(AuditResultChirho(
                category_chirho="build",
                severity_chirho="info",
                message_chirho="Cargo not found, skipping Rust check",
            ))

    return results_chirho


def check_test_coverage_chirho() -> List[AuditResultChirho]:
    """Check that tests exist for key modules."""
    results_chirho: List[AuditResultChirho] = []

    # Key modules that MUST have tests
    required_tests_chirho = [
        ("rust_chirho/src/approaches_chirho/adaptive_chirho.rs", "adaptive"),
        ("rust_chirho/src/approaches_chirho/diff_hierarchical_chirho.rs", "diff_hierarchical"),
        ("rust_chirho/src/approaches_chirho/hierarchical_chirho.rs", "hierarchical"),
    ]

    for module_path_chirho, test_name_chirho in required_tests_chirho:
        module_file_chirho = PROJECT_ROOT_CHIRHO / module_path_chirho
        if module_file_chirho.exists():
            with open(module_file_chirho, 'r') as f_chirho:
                content_chirho = f_chirho.read()

            if "#[cfg(test)]" not in content_chirho and "#[test]" not in content_chirho:
                results_chirho.append(AuditResultChirho(
                    category_chirho="testing",
                    severity_chirho="warning",
                    message_chirho=f"Module '{test_name_chirho}' has no inline tests",
                    file_chirho=module_path_chirho,
                ))

    return results_chirho


def run_audit_chirho(verbose_chirho: bool = False) -> Tuple[int, int, int]:
    """Run full audit and return (errors, warnings, infos)."""
    all_results_chirho: List[AuditResultChirho] = []

    print("=" * 60)
    print("miniKanren 1-Bit Integration Audit")
    print("=" * 60)
    print()

    # Run all checks
    print("Checking naming conventions...")
    all_results_chirho.extend(find_naming_violations_chirho(verbose_chirho))

    print("Checking component presence...")
    all_results_chirho.extend(check_component_presence_chirho())

    print("Checking documentation sync...")
    all_results_chirho.extend(check_docs_sync_chirho())

    print("Checking Rust compilation...")
    all_results_chirho.extend(check_rust_compiles_chirho())

    print("Checking test coverage...")
    all_results_chirho.extend(check_test_coverage_chirho())

    print()
    print("=" * 60)
    print("RESULTS")
    print("=" * 60)

    # Count by severity
    errors_chirho = [r for r in all_results_chirho if r.severity_chirho == "error"]
    warnings_chirho = [r for r in all_results_chirho if r.severity_chirho == "warning"]
    infos_chirho = [r for r in all_results_chirho if r.severity_chirho == "info"]

    # Print errors
    if errors_chirho:
        print(f"\nERRORS ({len(errors_chirho)}):")
        for r_chirho in errors_chirho:
            loc_chirho = f"{r_chirho.file_chirho}:{r_chirho.line_chirho}" if r_chirho.line_chirho else r_chirho.file_chirho
            print(f"  [ERROR] {r_chirho.message_chirho}")
            if loc_chirho:
                print(f"          at {loc_chirho}")

    # Print warnings (limited to first 20 if many)
    if warnings_chirho:
        print(f"\nWARNINGS ({len(warnings_chirho)}):")
        for r_chirho in warnings_chirho[:20]:
            loc_chirho = f"{r_chirho.file_chirho}:{r_chirho.line_chirho}" if r_chirho.line_chirho else r_chirho.file_chirho
            print(f"  [WARN] {r_chirho.message_chirho}")
            if loc_chirho and verbose_chirho:
                print(f"         at {loc_chirho}")
        if len(warnings_chirho) > 20:
            print(f"  ... and {len(warnings_chirho) - 20} more")

    # Print info if verbose
    if infos_chirho and verbose_chirho:
        print(f"\nINFO ({len(infos_chirho)}):")
        for r_chirho in infos_chirho:
            print(f"  [INFO] {r_chirho.message_chirho}")

    # Summary
    print()
    print("=" * 60)
    print(f"Summary: {len(errors_chirho)} errors, {len(warnings_chirho)} warnings, {len(infos_chirho)} infos")
    print("=" * 60)

    if errors_chirho:
        print("\nAudit FAILED - fix errors before proceeding")
    elif warnings_chirho:
        print("\nAudit PASSED with warnings - consider addressing them")
    else:
        print("\nAudit PASSED")

    return len(errors_chirho), len(warnings_chirho), len(infos_chirho)


def main_chirho():
    """Main entry point."""
    verbose_chirho = "--verbose" in sys.argv or "-v" in sys.argv

    errors_chirho, warnings_chirho, _ = run_audit_chirho(verbose_chirho)

    # Exit with error code if there are errors
    sys.exit(1 if errors_chirho > 0 else 0)


if __name__ == "__main__":
    main_chirho()
