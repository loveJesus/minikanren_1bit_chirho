//! SMT Solver Integration ☧
//!
//! Bridge between miniKanren's relational search and SMT solvers.
//! Uses SMT-LIB2 format for solver communication.
//!
//! Key mapping:
//! - miniKanren variables → SMT variables
//! - Unification constraints → SMT equality
//! - Finite domains → SMT bitvector or enumeration
//! - conde (disjunction) → SMT (or ...)
//!
//! This enables offloading constraint-heavy queries to specialized SMT solvers
//! like Z3, CVC5, or Bitwuzla.

use std::collections::HashMap;
use std::fmt::Write;

/// SMT variable type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtSortChirho {
    /// Boolean
    BoolChirho,
    /// Bitvector with width
    BitVecChirho(u32),
    /// Integer
    IntChirho,
    /// Enumeration type (finite domain)
    EnumChirho(u32), // ID into enum definitions
}

/// SMT expression
#[derive(Debug, Clone)]
pub enum SmtExprChirho {
    /// Variable reference
    VarChirho(String),
    /// Boolean constant
    BoolConstChirho(bool),
    /// Integer constant
    IntConstChirho(i64),
    /// Bitvector constant
    BvConstChirho(u64, u32), // value, width
    /// Equality
    EqChirho(Box<SmtExprChirho>, Box<SmtExprChirho>),
    /// Conjunction (and)
    AndChirho(Vec<SmtExprChirho>),
    /// Disjunction (or)
    OrChirho(Vec<SmtExprChirho>),
    /// Negation (not)
    NotChirho(Box<SmtExprChirho>),
    /// Bitvector AND
    BvAndChirho(Box<SmtExprChirho>, Box<SmtExprChirho>),
    /// Bitvector OR
    BvOrChirho(Box<SmtExprChirho>, Box<SmtExprChirho>),
    /// If-then-else
    IteChirho(Box<SmtExprChirho>, Box<SmtExprChirho>, Box<SmtExprChirho>),
}

impl SmtExprChirho {
    /// Create equality constraint
    pub fn eq_chirho(a_chirho: Self, b_chirho: Self) -> Self {
        SmtExprChirho::EqChirho(Box::new(a_chirho), Box::new(b_chirho))
    }

    /// Create conjunction
    pub fn and_chirho(exprs_chirho: Vec<Self>) -> Self {
        if exprs_chirho.is_empty() {
            SmtExprChirho::BoolConstChirho(true)
        } else if exprs_chirho.len() == 1 {
            exprs_chirho.into_iter().next().unwrap()
        } else {
            SmtExprChirho::AndChirho(exprs_chirho)
        }
    }

    /// Create disjunction
    pub fn or_chirho(exprs_chirho: Vec<Self>) -> Self {
        if exprs_chirho.is_empty() {
            SmtExprChirho::BoolConstChirho(false)
        } else if exprs_chirho.len() == 1 {
            exprs_chirho.into_iter().next().unwrap()
        } else {
            SmtExprChirho::OrChirho(exprs_chirho)
        }
    }

    /// Convert to SMT-LIB2 string
    pub fn to_smtlib_chirho(&self) -> String {
        match self {
            SmtExprChirho::VarChirho(name_chirho) => name_chirho.clone(),
            SmtExprChirho::BoolConstChirho(b_chirho) => {
                if *b_chirho { "true".to_string() } else { "false".to_string() }
            }
            SmtExprChirho::IntConstChirho(i_chirho) => {
                if *i_chirho >= 0 {
                    i_chirho.to_string()
                } else {
                    format!("(- {})", -i_chirho)
                }
            }
            SmtExprChirho::BvConstChirho(v_chirho, w_chirho) => {
                format!("(_ bv{} {})", v_chirho, w_chirho)
            }
            SmtExprChirho::EqChirho(a_chirho, b_chirho) => {
                format!("(= {} {})", a_chirho.to_smtlib_chirho(), b_chirho.to_smtlib_chirho())
            }
            SmtExprChirho::AndChirho(exprs_chirho) => {
                let inner_chirho: Vec<_> = exprs_chirho.iter().map(|e| e.to_smtlib_chirho()).collect();
                format!("(and {})", inner_chirho.join(" "))
            }
            SmtExprChirho::OrChirho(exprs_chirho) => {
                let inner_chirho: Vec<_> = exprs_chirho.iter().map(|e| e.to_smtlib_chirho()).collect();
                format!("(or {})", inner_chirho.join(" "))
            }
            SmtExprChirho::NotChirho(e_chirho) => {
                format!("(not {})", e_chirho.to_smtlib_chirho())
            }
            SmtExprChirho::BvAndChirho(a_chirho, b_chirho) => {
                format!("(bvand {} {})", a_chirho.to_smtlib_chirho(), b_chirho.to_smtlib_chirho())
            }
            SmtExprChirho::BvOrChirho(a_chirho, b_chirho) => {
                format!("(bvor {} {})", a_chirho.to_smtlib_chirho(), b_chirho.to_smtlib_chirho())
            }
            SmtExprChirho::IteChirho(c_chirho, t_chirho, e_chirho) => {
                format!(
                    "(ite {} {} {})",
                    c_chirho.to_smtlib_chirho(),
                    t_chirho.to_smtlib_chirho(),
                    e_chirho.to_smtlib_chirho()
                )
            }
        }
    }
}

/// SMT problem builder
#[derive(Debug, Clone)]
pub struct SmtProblemChirho {
    /// Variable declarations: name → sort
    vars_chirho: HashMap<String, SmtSortChirho>,
    /// Constraints to assert
    constraints_chirho: Vec<SmtExprChirho>,
    /// Logic to use
    logic_chirho: String,
}

impl SmtProblemChirho {
    /// Create new problem with given logic
    pub fn new_chirho(logic_chirho: &str) -> Self {
        Self {
            vars_chirho: HashMap::new(),
            constraints_chirho: Vec::new(),
            logic_chirho: logic_chirho.to_string(),
        }
    }

    /// Create problem for quantifier-free bitvectors
    pub fn new_qfbv_chirho() -> Self {
        Self::new_chirho("QF_BV")
    }

    /// Create problem for quantifier-free linear integer arithmetic
    pub fn new_qflia_chirho() -> Self {
        Self::new_chirho("QF_LIA")
    }

    /// Declare a variable
    pub fn declare_var_chirho(&mut self, name_chirho: &str, sort_chirho: SmtSortChirho) {
        self.vars_chirho.insert(name_chirho.to_string(), sort_chirho);
    }

    /// Add constraint
    pub fn assert_chirho(&mut self, expr_chirho: SmtExprChirho) {
        self.constraints_chirho.push(expr_chirho);
    }

    /// Add equality constraint between two variables
    pub fn assert_eq_chirho(&mut self, var1_chirho: &str, var2_chirho: &str) {
        self.assert_chirho(SmtExprChirho::eq_chirho(
            SmtExprChirho::VarChirho(var1_chirho.to_string()),
            SmtExprChirho::VarChirho(var2_chirho.to_string()),
        ));
    }

    /// Convert sort to SMT-LIB2 string
    fn sort_to_smtlib_chirho(sort_chirho: &SmtSortChirho) -> String {
        match sort_chirho {
            SmtSortChirho::BoolChirho => "Bool".to_string(),
            SmtSortChirho::BitVecChirho(w_chirho) => format!("(_ BitVec {})", w_chirho),
            SmtSortChirho::IntChirho => "Int".to_string(),
            SmtSortChirho::EnumChirho(id_chirho) => format!("Enum{}", id_chirho),
        }
    }

    /// Generate complete SMT-LIB2 script
    pub fn to_smtlib_chirho(&self) -> String {
        let mut script_chirho = String::new();

        // Set logic
        writeln!(&mut script_chirho, "(set-logic {})", self.logic_chirho).unwrap();
        writeln!(&mut script_chirho, "(set-option :produce-models true)").unwrap();
        writeln!(&mut script_chirho).unwrap();

        // Declare variables
        for (name_chirho, sort_chirho) in &self.vars_chirho {
            writeln!(
                &mut script_chirho,
                "(declare-const {} {})",
                name_chirho,
                Self::sort_to_smtlib_chirho(sort_chirho)
            )
            .unwrap();
        }
        writeln!(&mut script_chirho).unwrap();

        // Assert constraints
        for constraint_chirho in &self.constraints_chirho {
            writeln!(
                &mut script_chirho,
                "(assert {})",
                constraint_chirho.to_smtlib_chirho()
            )
            .unwrap();
        }
        writeln!(&mut script_chirho).unwrap();

        // Check satisfiability and get model
        writeln!(&mut script_chirho, "(check-sat)").unwrap();
        writeln!(&mut script_chirho, "(get-model)").unwrap();

        script_chirho
    }
}

/// Convert miniKanren domain constraint to SMT
/// Domain is a 64-bit mask; we encode as disjunction of equalities
pub fn domain_to_smt_chirho(var_name_chirho: &str, domain_chirho: u64) -> SmtExprChirho {
    let mut disjuncts_chirho = Vec::new();

    for i_chirho in 0..64u32 {
        if (domain_chirho & (1u64 << i_chirho)) != 0 {
            disjuncts_chirho.push(SmtExprChirho::eq_chirho(
                SmtExprChirho::VarChirho(var_name_chirho.to_string()),
                SmtExprChirho::IntConstChirho(i_chirho as i64),
            ));
        }
    }

    SmtExprChirho::or_chirho(disjuncts_chirho)
}

/// Convert miniKanren domain to SMT bitvector membership
/// More efficient for SMT solver than disjunction
pub fn domain_to_bv_smt_chirho(
    var_name_chirho: &str,
    domain_chirho: u64,
    domain_bv_name_chirho: &str,
) -> SmtExprChirho {
    // (bvand (bvshl (_ bv1 64) var) domain) != 0
    // But simpler: assert var is one of the set bits
    SmtExprChirho::NotChirho(Box::new(SmtExprChirho::eq_chirho(
        SmtExprChirho::BvAndChirho(
            Box::new(SmtExprChirho::VarChirho(domain_bv_name_chirho.to_string())),
            Box::new(SmtExprChirho::VarChirho(var_name_chirho.to_string())),
        ),
        SmtExprChirho::BvConstChirho(0, 64),
    )))
}

/// Parse SMT solver output for satisfiability
pub fn parse_sat_result_chirho(output_chirho: &str) -> Option<bool> {
    let trimmed_chirho = output_chirho.trim();
    if trimmed_chirho.starts_with("sat") {
        Some(true)
    } else if trimmed_chirho.starts_with("unsat") {
        Some(false)
    } else {
        None
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_smt_expr_to_smtlib_chirho() {
        let expr_chirho = SmtExprChirho::eq_chirho(
            SmtExprChirho::VarChirho("x_chirho".to_string()),
            SmtExprChirho::IntConstChirho(42),
        );

        assert_eq!(expr_chirho.to_smtlib_chirho(), "(= x_chirho 42)");
    }

    #[test]
    fn test_smt_and_or_chirho() {
        let and_expr_chirho = SmtExprChirho::and_chirho(vec![
            SmtExprChirho::VarChirho("a_chirho".to_string()),
            SmtExprChirho::VarChirho("b_chirho".to_string()),
        ]);
        assert_eq!(and_expr_chirho.to_smtlib_chirho(), "(and a_chirho b_chirho)");

        let or_expr_chirho = SmtExprChirho::or_chirho(vec![
            SmtExprChirho::VarChirho("x_chirho".to_string()),
            SmtExprChirho::VarChirho("y_chirho".to_string()),
        ]);
        assert_eq!(or_expr_chirho.to_smtlib_chirho(), "(or x_chirho y_chirho)");
    }

    #[test]
    fn test_domain_to_smt_chirho() {
        // Domain {0, 2, 3} = 0b1101
        let expr_chirho = domain_to_smt_chirho("v_chirho", 0b1101);
        let smtlib_chirho = expr_chirho.to_smtlib_chirho();

        assert!(smtlib_chirho.contains("(= v_chirho 0)"));
        assert!(smtlib_chirho.contains("(= v_chirho 2)"));
        assert!(smtlib_chirho.contains("(= v_chirho 3)"));
        assert!(!smtlib_chirho.contains("(= v_chirho 1)"));
    }

    #[test]
    fn test_smt_problem_chirho() {
        let mut problem_chirho = SmtProblemChirho::new_qflia_chirho();

        problem_chirho.declare_var_chirho("x_chirho", SmtSortChirho::IntChirho);
        problem_chirho.declare_var_chirho("y_chirho", SmtSortChirho::IntChirho);

        // x == y
        problem_chirho.assert_eq_chirho("x_chirho", "y_chirho");

        // x in {1, 2, 3}
        problem_chirho.assert_chirho(domain_to_smt_chirho("x_chirho", 0b1110));

        let smtlib_chirho = problem_chirho.to_smtlib_chirho();

        assert!(smtlib_chirho.contains("(set-logic QF_LIA)"));
        assert!(smtlib_chirho.contains("(declare-const x_chirho Int)"));
        assert!(smtlib_chirho.contains("(assert (= x_chirho y_chirho))"));
        assert!(smtlib_chirho.contains("(check-sat)"));
    }

    #[test]
    fn test_parse_sat_result_chirho() {
        assert_eq!(parse_sat_result_chirho("sat\n"), Some(true));
        assert_eq!(parse_sat_result_chirho("unsat"), Some(false));
        assert_eq!(parse_sat_result_chirho("unknown"), None);
    }
}
