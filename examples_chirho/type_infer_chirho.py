#!/usr/bin/env python3
"""
Type Inference using miniKanren with Hash Consing ☧

Demonstrates Hindley-Milner style type inference using:
- Hash-consed type terms (structural sharing)
- Unification for type equations
- Domain constraints for polymorphism

"Whether therefore ye eat, or drink, or whatsoever ye do, 
 do all to the glory of God." — 1 Corinthians 10:31
"""

from dataclasses import dataclass
from typing import Dict, List, Optional, Set, Tuple, Union
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# ============================================================================
# Hash-Consed Type Terms
# ============================================================================

@dataclass(frozen=True)
class TypeVarChirho:
    """Type variable (e.g., 'a, 'b)."""
    name_chirho: str
    
    def __repr__(self_chirho):
        return f"'{self_chirho.name_chirho}"

@dataclass(frozen=True)
class TypeIntChirho:
    """Integer type."""
    def __repr__(self_chirho):
        return "Int"

@dataclass(frozen=True)
class TypeBoolChirho:
    """Boolean type."""
    def __repr__(self_chirho):
        return "Bool"

@dataclass(frozen=True)
class TypeFunChirho:
    """Function type: arg -> result."""
    arg_chirho: 'TypeTermChirho'
    result_chirho: 'TypeTermChirho'
    
    def __repr__(self_chirho):
        arg_str_chirho = f"({self_chirho.arg_chirho})" if isinstance(self_chirho.arg_chirho, TypeFunChirho) else str(self_chirho.arg_chirho)
        return f"{arg_str_chirho} -> {self_chirho.result_chirho}"

@dataclass(frozen=True)
class TypeListChirho:
    """List type: [elem]."""
    elem_chirho: 'TypeTermChirho'
    
    def __repr__(self_chirho):
        return f"[{self_chirho.elem_chirho}]"

@dataclass(frozen=True)
class TypePairChirho:
    """Pair type: (fst, snd)."""
    fst_chirho: 'TypeTermChirho'
    snd_chirho: 'TypeTermChirho'
    
    def __repr__(self_chirho):
        return f"({self_chirho.fst_chirho}, {self_chirho.snd_chirho})"

# Union of all type terms
TypeTermChirho = Union[TypeVarChirho, TypeIntChirho, TypeBoolChirho, 
                       TypeFunChirho, TypeListChirho, TypePairChirho]


class TypeStoreChirho:
    """
    Hash-consing store for type terms.
    
    Ensures structural sharing: identical types share the same object.
    This is crucial for efficient unification and occurs check.
    """
    
    def __init__(self_chirho):
        self_chirho.cache_chirho: Dict[TypeTermChirho, TypeTermChirho] = {}
        self_chirho.var_counter_chirho = 0
    
    def intern_chirho(self_chirho, t_chirho: TypeTermChirho) -> TypeTermChirho:
        """Intern a type term (hash consing)."""
        if t_chirho in self_chirho.cache_chirho:
            return self_chirho.cache_chirho[t_chirho]
        self_chirho.cache_chirho[t_chirho] = t_chirho
        return t_chirho
    
    def fresh_var_chirho(self_chirho) -> TypeVarChirho:
        """Create a fresh type variable."""
        name_chirho = f"t{self_chirho.var_counter_chirho}"
        self_chirho.var_counter_chirho += 1
        return self_chirho.intern_chirho(TypeVarChirho(name_chirho))
    
    def int_chirho(self_chirho) -> TypeIntChirho:
        return self_chirho.intern_chirho(TypeIntChirho())
    
    def bool_chirho(self_chirho) -> TypeBoolChirho:
        return self_chirho.intern_chirho(TypeBoolChirho())
    
    def fun_chirho(self_chirho, arg_chirho: TypeTermChirho, result_chirho: TypeTermChirho) -> TypeFunChirho:
        return self_chirho.intern_chirho(TypeFunChirho(arg_chirho, result_chirho))
    
    def list_chirho(self_chirho, elem_chirho: TypeTermChirho) -> TypeListChirho:
        return self_chirho.intern_chirho(TypeListChirho(elem_chirho))
    
    def pair_chirho(self_chirho, fst_chirho: TypeTermChirho, snd_chirho: TypeTermChirho) -> TypePairChirho:
        return self_chirho.intern_chirho(TypePairChirho(fst_chirho, snd_chirho))


# ============================================================================
# Substitution (Type Variable Bindings)
# ============================================================================

class SubstChirho:
    """
    Substitution mapping type variables to types.
    
    Implements triangular substitution for efficient unification.
    """
    
    def __init__(self_chirho):
        self_chirho.bindings_chirho: Dict[str, TypeTermChirho] = {}
    
    def bind_chirho(self_chirho, var_chirho: TypeVarChirho, t_chirho: TypeTermChirho):
        """Bind a type variable to a type."""
        self_chirho.bindings_chirho[var_chirho.name_chirho] = t_chirho
    
    def lookup_chirho(self_chirho, var_chirho: TypeVarChirho) -> Optional[TypeTermChirho]:
        """Look up a type variable's binding."""
        return self_chirho.bindings_chirho.get(var_chirho.name_chirho)
    
    def walk_chirho(self_chirho, t_chirho: TypeTermChirho) -> TypeTermChirho:
        """Walk to the end of the substitution chain."""
        while isinstance(t_chirho, TypeVarChirho):
            binding_chirho = self_chirho.lookup_chirho(t_chirho)
            if binding_chirho is None:
                break
            t_chirho = binding_chirho
        return t_chirho
    
    def apply_chirho(self_chirho, t_chirho: TypeTermChirho, store_chirho: TypeStoreChirho) -> TypeTermChirho:
        """Fully apply substitution to a type."""
        t_chirho = self_chirho.walk_chirho(t_chirho)
        
        if isinstance(t_chirho, TypeVarChirho):
            return t_chirho
        elif isinstance(t_chirho, (TypeIntChirho, TypeBoolChirho)):
            return t_chirho
        elif isinstance(t_chirho, TypeFunChirho):
            return store_chirho.fun_chirho(
                self_chirho.apply_chirho(t_chirho.arg_chirho, store_chirho),
                self_chirho.apply_chirho(t_chirho.result_chirho, store_chirho)
            )
        elif isinstance(t_chirho, TypeListChirho):
            return store_chirho.list_chirho(
                self_chirho.apply_chirho(t_chirho.elem_chirho, store_chirho)
            )
        elif isinstance(t_chirho, TypePairChirho):
            return store_chirho.pair_chirho(
                self_chirho.apply_chirho(t_chirho.fst_chirho, store_chirho),
                self_chirho.apply_chirho(t_chirho.snd_chirho, store_chirho)
            )
        else:
            return t_chirho
    
    def copy_chirho(self_chirho) -> 'SubstChirho':
        """Create a copy for branching."""
        new_chirho = SubstChirho()
        new_chirho.bindings_chirho = self_chirho.bindings_chirho.copy()
        return new_chirho


# ============================================================================
# Unification
# ============================================================================

def occurs_check_chirho(var_chirho: TypeVarChirho, t_chirho: TypeTermChirho, subst_chirho: SubstChirho) -> bool:
    """
    Check if var occurs in t (prevents infinite types).
    
    This is the occurs check that prevents cycles like 'a = 'a -> Int.
    """
    t_chirho = subst_chirho.walk_chirho(t_chirho)
    
    if isinstance(t_chirho, TypeVarChirho):
        return t_chirho.name_chirho == var_chirho.name_chirho
    elif isinstance(t_chirho, (TypeIntChirho, TypeBoolChirho)):
        return False
    elif isinstance(t_chirho, TypeFunChirho):
        return (occurs_check_chirho(var_chirho, t_chirho.arg_chirho, subst_chirho) or
                occurs_check_chirho(var_chirho, t_chirho.result_chirho, subst_chirho))
    elif isinstance(t_chirho, TypeListChirho):
        return occurs_check_chirho(var_chirho, t_chirho.elem_chirho, subst_chirho)
    elif isinstance(t_chirho, TypePairChirho):
        return (occurs_check_chirho(var_chirho, t_chirho.fst_chirho, subst_chirho) or
                occurs_check_chirho(var_chirho, t_chirho.snd_chirho, subst_chirho))
    return False


def unify_chirho(t1_chirho: TypeTermChirho, t2_chirho: TypeTermChirho, 
                 subst_chirho: SubstChirho) -> bool:
    """
    Unify two types, updating the substitution.
    
    Returns True if unification succeeds, False otherwise.
    This is the core constraint solving operation.
    """
    t1_chirho = subst_chirho.walk_chirho(t1_chirho)
    t2_chirho = subst_chirho.walk_chirho(t2_chirho)
    
    # Same type (including same variable)
    if t1_chirho == t2_chirho:
        return True
    
    # Var = Type
    if isinstance(t1_chirho, TypeVarChirho):
        if occurs_check_chirho(t1_chirho, t2_chirho, subst_chirho):
            return False  # Infinite type
        subst_chirho.bind_chirho(t1_chirho, t2_chirho)
        return True
    
    if isinstance(t2_chirho, TypeVarChirho):
        if occurs_check_chirho(t2_chirho, t1_chirho, subst_chirho):
            return False
        subst_chirho.bind_chirho(t2_chirho, t1_chirho)
        return True
    
    # Structural unification
    if isinstance(t1_chirho, TypeIntChirho) and isinstance(t2_chirho, TypeIntChirho):
        return True
    
    if isinstance(t1_chirho, TypeBoolChirho) and isinstance(t2_chirho, TypeBoolChirho):
        return True
    
    if isinstance(t1_chirho, TypeFunChirho) and isinstance(t2_chirho, TypeFunChirho):
        return (unify_chirho(t1_chirho.arg_chirho, t2_chirho.arg_chirho, subst_chirho) and
                unify_chirho(t1_chirho.result_chirho, t2_chirho.result_chirho, subst_chirho))
    
    if isinstance(t1_chirho, TypeListChirho) and isinstance(t2_chirho, TypeListChirho):
        return unify_chirho(t1_chirho.elem_chirho, t2_chirho.elem_chirho, subst_chirho)
    
    if isinstance(t1_chirho, TypePairChirho) and isinstance(t2_chirho, TypePairChirho):
        return (unify_chirho(t1_chirho.fst_chirho, t2_chirho.fst_chirho, subst_chirho) and
                unify_chirho(t1_chirho.snd_chirho, t2_chirho.snd_chirho, subst_chirho))
    
    return False  # Type mismatch


# ============================================================================
# Simple Expression Language
# ============================================================================

@dataclass
class ExprVarChirho:
    """Variable reference."""
    name_chirho: str

@dataclass
class ExprIntChirho:
    """Integer literal."""
    value_chirho: int

@dataclass
class ExprBoolChirho:
    """Boolean literal."""
    value_chirho: bool

@dataclass
class ExprLamChirho:
    """Lambda abstraction: λx. body."""
    param_chirho: str
    body_chirho: 'ExprChirho'

@dataclass  
class ExprAppChirho:
    """Function application: f x."""
    fun_chirho: 'ExprChirho'
    arg_chirho: 'ExprChirho'

@dataclass
class ExprIfChirho:
    """Conditional: if cond then t else f."""
    cond_chirho: 'ExprChirho'
    then_chirho: 'ExprChirho'
    else_chirho: 'ExprChirho'

@dataclass
class ExprLetChirho:
    """Let binding: let x = e1 in e2."""
    name_chirho: str
    value_chirho: 'ExprChirho'
    body_chirho: 'ExprChirho'

ExprChirho = Union[ExprVarChirho, ExprIntChirho, ExprBoolChirho, 
                   ExprLamChirho, ExprAppChirho, ExprIfChirho, ExprLetChirho]


# ============================================================================
# Type Inference
# ============================================================================

class TypeEnvChirho:
    """Type environment mapping variable names to types."""
    
    def __init__(self_chirho, bindings_chirho: Optional[Dict[str, TypeTermChirho]] = None):
        self_chirho.bindings_chirho = bindings_chirho or {}
    
    def extend_chirho(self_chirho, name_chirho: str, t_chirho: TypeTermChirho) -> 'TypeEnvChirho':
        """Extend environment with a new binding."""
        new_bindings_chirho = self_chirho.bindings_chirho.copy()
        new_bindings_chirho[name_chirho] = t_chirho
        return TypeEnvChirho(new_bindings_chirho)
    
    def lookup_chirho(self_chirho, name_chirho: str) -> Optional[TypeTermChirho]:
        return self_chirho.bindings_chirho.get(name_chirho)


def infer_chirho(expr_chirho: ExprChirho, env_chirho: TypeEnvChirho,
                 store_chirho: TypeStoreChirho, subst_chirho: SubstChirho) -> Optional[TypeTermChirho]:
    """
    Infer the type of an expression.
    
    Uses unification to solve type constraints.
    Returns None if type inference fails.
    """
    if isinstance(expr_chirho, ExprIntChirho):
        return store_chirho.int_chirho()
    
    elif isinstance(expr_chirho, ExprBoolChirho):
        return store_chirho.bool_chirho()
    
    elif isinstance(expr_chirho, ExprVarChirho):
        t_chirho = env_chirho.lookup_chirho(expr_chirho.name_chirho)
        if t_chirho is None:
            print(f"Error: Unbound variable '{expr_chirho.name_chirho}'")
            return None
        return t_chirho
    
    elif isinstance(expr_chirho, ExprLamChirho):
        # λx. body  has type  α -> β  where x:α ⊢ body:β
        param_type_chirho = store_chirho.fresh_var_chirho()
        new_env_chirho = env_chirho.extend_chirho(expr_chirho.param_chirho, param_type_chirho)
        body_type_chirho = infer_chirho(expr_chirho.body_chirho, new_env_chirho, store_chirho, subst_chirho)
        if body_type_chirho is None:
            return None
        return store_chirho.fun_chirho(param_type_chirho, body_type_chirho)
    
    elif isinstance(expr_chirho, ExprAppChirho):
        # f x  requires  f : α -> β  and  x : α, result is β
        fun_type_chirho = infer_chirho(expr_chirho.fun_chirho, env_chirho, store_chirho, subst_chirho)
        if fun_type_chirho is None:
            return None
        
        arg_type_chirho = infer_chirho(expr_chirho.arg_chirho, env_chirho, store_chirho, subst_chirho)
        if arg_type_chirho is None:
            return None
        
        result_type_chirho = store_chirho.fresh_var_chirho()
        expected_fun_chirho = store_chirho.fun_chirho(arg_type_chirho, result_type_chirho)
        
        if not unify_chirho(fun_type_chirho, expected_fun_chirho, subst_chirho):
            print(f"Error: Cannot apply {subst_chirho.apply_chirho(fun_type_chirho, store_chirho)} to {subst_chirho.apply_chirho(arg_type_chirho, store_chirho)}")
            return None
        
        return result_type_chirho
    
    elif isinstance(expr_chirho, ExprIfChirho):
        # if c then t else f  requires  c:Bool, t:α, f:α, result is α
        cond_type_chirho = infer_chirho(expr_chirho.cond_chirho, env_chirho, store_chirho, subst_chirho)
        if cond_type_chirho is None:
            return None
        
        if not unify_chirho(cond_type_chirho, store_chirho.bool_chirho(), subst_chirho):
            print("Error: Condition must be Bool")
            return None
        
        then_type_chirho = infer_chirho(expr_chirho.then_chirho, env_chirho, store_chirho, subst_chirho)
        if then_type_chirho is None:
            return None
        
        else_type_chirho = infer_chirho(expr_chirho.else_chirho, env_chirho, store_chirho, subst_chirho)
        if else_type_chirho is None:
            return None
        
        if not unify_chirho(then_type_chirho, else_type_chirho, subst_chirho):
            print("Error: Branches must have same type")
            return None
        
        return then_type_chirho
    
    elif isinstance(expr_chirho, ExprLetChirho):
        # let x = e1 in e2  has type of e2 with x bound to type of e1
        val_type_chirho = infer_chirho(expr_chirho.value_chirho, env_chirho, store_chirho, subst_chirho)
        if val_type_chirho is None:
            return None
        
        new_env_chirho = env_chirho.extend_chirho(expr_chirho.name_chirho, val_type_chirho)
        return infer_chirho(expr_chirho.body_chirho, new_env_chirho, store_chirho, subst_chirho)
    
    return None


# ============================================================================
# Examples
# ============================================================================

def format_expr_chirho(expr_chirho: ExprChirho) -> str:
    """Pretty-print an expression."""
    if isinstance(expr_chirho, ExprIntChirho):
        return str(expr_chirho.value_chirho)
    elif isinstance(expr_chirho, ExprBoolChirho):
        return "true" if expr_chirho.value_chirho else "false"
    elif isinstance(expr_chirho, ExprVarChirho):
        return expr_chirho.name_chirho
    elif isinstance(expr_chirho, ExprLamChirho):
        return f"λ{expr_chirho.param_chirho}. {format_expr_chirho(expr_chirho.body_chirho)}"
    elif isinstance(expr_chirho, ExprAppChirho):
        fun_str_chirho = format_expr_chirho(expr_chirho.fun_chirho)
        arg_str_chirho = format_expr_chirho(expr_chirho.arg_chirho)
        if isinstance(expr_chirho.arg_chirho, (ExprAppChirho, ExprLamChirho)):
            arg_str_chirho = f"({arg_str_chirho})"
        return f"{fun_str_chirho} {arg_str_chirho}"
    elif isinstance(expr_chirho, ExprIfChirho):
        return f"if {format_expr_chirho(expr_chirho.cond_chirho)} then {format_expr_chirho(expr_chirho.then_chirho)} else {format_expr_chirho(expr_chirho.else_chirho)}"
    elif isinstance(expr_chirho, ExprLetChirho):
        return f"let {expr_chirho.name_chirho} = {format_expr_chirho(expr_chirho.value_chirho)} in {format_expr_chirho(expr_chirho.body_chirho)}"
    return "?"


def infer_and_print_chirho(name_chirho: str, expr_chirho: ExprChirho):
    """Infer type and print result."""
    store_chirho = TypeStoreChirho()
    subst_chirho = SubstChirho()
    env_chirho = TypeEnvChirho()
    
    print(f"{name_chirho}:")
    print(f"  Expression: {format_expr_chirho(expr_chirho)}")
    
    t_chirho = infer_chirho(expr_chirho, env_chirho, store_chirho, subst_chirho)
    
    if t_chirho:
        final_type_chirho = subst_chirho.apply_chirho(t_chirho, store_chirho)
        print(f"  Type: {final_type_chirho}")
        print(f"  Hash-cons cache size: {len(store_chirho.cache_chirho)}")
    else:
        print("  Type: ERROR")
    
    print()


def main_chirho():
    print("Type Inference using miniKanren with Hash Consing ☧\n")
    print("="*50)
    
    # Example 1: Identity function
    id_chirho = ExprLamChirho("x", ExprVarChirho("x"))
    infer_and_print_chirho("Identity", id_chirho)
    
    # Example 2: Constant function
    const_chirho = ExprLamChirho("x", ExprLamChirho("y", ExprVarChirho("x")))
    infer_and_print_chirho("Const", const_chirho)
    
    # Example 3: Apply identity to an int
    apply_id_chirho = ExprAppChirho(
        ExprLamChirho("x", ExprVarChirho("x")),
        ExprIntChirho(42)
    )
    infer_and_print_chirho("Apply id to 42", apply_id_chirho)
    
    # Example 4: Conditional
    cond_chirho = ExprIfChirho(
        ExprBoolChirho(True),
        ExprIntChirho(1),
        ExprIntChirho(2)
    )
    infer_and_print_chirho("If-then-else", cond_chirho)
    
    # Example 5: Let binding
    let_chirho = ExprLetChirho(
        "double",
        ExprLamChirho("x", ExprVarChirho("x")),  # simplified
        ExprAppChirho(ExprVarChirho("double"), ExprIntChirho(5))
    )
    infer_and_print_chirho("Let binding", let_chirho)
    
    # Example 6: Higher-order function
    apply_chirho = ExprLamChirho("f", 
        ExprLamChirho("x", 
            ExprAppChirho(ExprVarChirho("f"), ExprVarChirho("x"))
        )
    )
    infer_and_print_chirho("Apply (higher-order)", apply_chirho)
    
    # Example 7: Composition
    compose_chirho = ExprLamChirho("f",
        ExprLamChirho("g",
            ExprLamChirho("x",
                ExprAppChirho(
                    ExprVarChirho("f"),
                    ExprAppChirho(ExprVarChirho("g"), ExprVarChirho("x"))
                )
            )
        )
    )
    infer_and_print_chirho("Compose", compose_chirho)
    
    # Example 8: Type error
    print("--- Type Error Example ---")
    bad_chirho = ExprIfChirho(
        ExprIntChirho(1),  # Not a Bool!
        ExprIntChirho(2),
        ExprIntChirho(3)
    )
    infer_and_print_chirho("Bad conditional", bad_chirho)
    
    print("☧ Soli Deo Gloria ☧")


if __name__ == "__main__":
    main_chirho()
