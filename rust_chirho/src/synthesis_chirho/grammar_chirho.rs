//! Grammar Domain Encoding ☧
//!
//! Encodes grammar productions as domain values using Hierarchical4kChirho
//! for efficient constraint propagation during synthesis.

use crate::approaches_chirho::Hierarchical4kChirho;
use super::sygus_chirho::{SygusGrammarChirho, SygusProductionChirho, SygusTypeChirho};
use std::collections::HashMap;

/// Maximum production ID (fits in Hierarchical4kChirho = 4096)
pub const MAX_PRODUCTIONS_CHIRHO: u32 = 4096;

/// Encoded grammar with production IDs mapping to domains
#[derive(Debug, Clone)]
pub struct EncodedGrammarChirho {
    /// Production ID → Production
    pub productions_chirho: Vec<ProductionInfoChirho>,
    /// Non-terminal → valid production IDs
    pub nonterminal_domains_chirho: HashMap<String, Hierarchical4kChirho>,
    /// Production ID → result type
    pub result_types_chirho: Vec<SygusTypeChirho>,
    /// Number of productions
    pub num_productions_chirho: u32,
}

/// Information about a single production
#[derive(Debug, Clone)]
pub struct ProductionInfoChirho {
    /// Production ID
    pub id_chirho: u32,
    /// Non-terminal this belongs to
    pub nonterminal_chirho: String,
    /// The production itself
    pub production_chirho: SygusProductionChirho,
    /// Child non-terminals (for non-leaf productions)
    pub children_chirho: Vec<String>,
    /// Arity (number of children)
    pub arity_chirho: u32,
}

impl EncodedGrammarChirho {
    /// Encode a SyGuS grammar into domain representation
    pub fn from_sygus_chirho(grammar_chirho: &SygusGrammarChirho) -> Self {
        let mut productions_chirho = Vec::new();
        let mut nonterminal_domains_chirho = HashMap::new();
        let mut result_types_chirho = Vec::new();
        let mut id_chirho = 0u32;

        for (nt_chirho, prods_chirho) in &grammar_chirho.rules_chirho {
            let mut domain_chirho = Hierarchical4kChirho::empty_chirho();

            for prod_chirho in prods_chirho {
                if id_chirho >= MAX_PRODUCTIONS_CHIRHO {
                    break;
                }

                let (children_chirho, arity_chirho) = Self::extract_children_chirho(prod_chirho);
                let result_type_chirho = Self::infer_type_chirho(prod_chirho, nt_chirho);

                productions_chirho.push(ProductionInfoChirho {
                    id_chirho,
                    nonterminal_chirho: nt_chirho.clone(),
                    production_chirho: prod_chirho.clone(),
                    children_chirho,
                    arity_chirho,
                });

                result_types_chirho.push(result_type_chirho);

                // Add to domain
                let leaf_idx_chirho = (id_chirho / 64) as usize;
                let bit_idx_chirho = id_chirho % 64;
                if leaf_idx_chirho < 64 {
                    domain_chirho.leaves_chirho[leaf_idx_chirho].0 |= 1u64 << bit_idx_chirho;
                    domain_chirho.root_chirho.0 |= 1u64 << leaf_idx_chirho;
                }

                id_chirho += 1;
            }

            nonterminal_domains_chirho.insert(nt_chirho.clone(), domain_chirho);
        }

        EncodedGrammarChirho {
            productions_chirho,
            nonterminal_domains_chirho,
            result_types_chirho,
            num_productions_chirho: id_chirho,
        }
    }

    fn extract_children_chirho(prod_chirho: &SygusProductionChirho) -> (Vec<String>, u32) {
        match prod_chirho {
            SygusProductionChirho::VarChirho(_) => (vec![], 0),
            SygusProductionChirho::IntConstChirho(_) => (vec![], 0),
            SygusProductionChirho::BoolConstChirho(_) => (vec![], 0),
            SygusProductionChirho::ConstantPlaceholderChirho(_) => (vec![], 0),
            SygusProductionChirho::UnaryOpChirho { arg_chirho, .. } => {
                (vec![arg_chirho.clone()], 1)
            }
            SygusProductionChirho::BinOpChirho { arg1_chirho, arg2_chirho, .. } => {
                (vec![arg1_chirho.clone(), arg2_chirho.clone()], 2)
            }
            SygusProductionChirho::IteChirho { cond_chirho, then_chirho, else_chirho } => {
                (vec![cond_chirho.clone(), then_chirho.clone(), else_chirho.clone()], 3)
            }
        }
    }

    fn infer_type_chirho(prod_chirho: &SygusProductionChirho, nt_chirho: &str) -> SygusTypeChirho {
        match prod_chirho {
            SygusProductionChirho::IntConstChirho(_) => SygusTypeChirho::IntChirho,
            SygusProductionChirho::BoolConstChirho(_) => SygusTypeChirho::BoolChirho,
            SygusProductionChirho::BinOpChirho { op_chirho, .. } => {
                if ["<=", ">=", "<", ">", "=", "and", "or"].contains(&op_chirho.as_str()) {
                    SygusTypeChirho::BoolChirho
                } else {
                    SygusTypeChirho::IntChirho
                }
            }
            _ => {
                if nt_chirho.contains("Bool") {
                    SygusTypeChirho::BoolChirho
                } else {
                    SygusTypeChirho::IntChirho
                }
            }
        }
    }

    /// Get domain of all productions for a non-terminal
    pub fn domain_for_chirho(&self, nonterminal_chirho: &str) -> Hierarchical4kChirho {
        self.nonterminal_domains_chirho
            .get(nonterminal_chirho)
            .cloned()
            .unwrap_or_else(Hierarchical4kChirho::empty_chirho)
    }

    /// Get domain of leaf productions (terminals)
    pub fn leaf_domain_chirho(&self) -> Hierarchical4kChirho {
        let mut domain_chirho = Hierarchical4kChirho::empty_chirho();

        for prod_info_chirho in &self.productions_chirho {
            if prod_info_chirho.arity_chirho == 0 {
                let leaf_idx_chirho = (prod_info_chirho.id_chirho / 64) as usize;
                let bit_idx_chirho = prod_info_chirho.id_chirho % 64;
                if leaf_idx_chirho < 64 {
                    domain_chirho.leaves_chirho[leaf_idx_chirho].0 |= 1u64 << bit_idx_chirho;
                    domain_chirho.root_chirho.0 |= 1u64 << leaf_idx_chirho;
                }
            }
        }

        domain_chirho
    }

    /// Get domain of productions with given result type
    pub fn domain_for_type_chirho(&self, type_chirho: &SygusTypeChirho) -> Hierarchical4kChirho {
        let mut domain_chirho = Hierarchical4kChirho::empty_chirho();

        for (idx_chirho, result_type_chirho) in self.result_types_chirho.iter().enumerate() {
            if result_type_chirho == type_chirho {
                let leaf_idx_chirho = idx_chirho / 64;
                let bit_idx_chirho = idx_chirho % 64;
                if leaf_idx_chirho < 64 {
                    domain_chirho.leaves_chirho[leaf_idx_chirho].0 |= 1u64 << bit_idx_chirho;
                    domain_chirho.root_chirho.0 |= 1u64 << leaf_idx_chirho;
                }
            }
        }

        domain_chirho
    }

    /// Filter domain by maximum depth (only leaves at max depth)
    pub fn domain_at_depth_chirho(&self, depth_chirho: u32, max_depth_chirho: u32) -> Hierarchical4kChirho {
        if depth_chirho >= max_depth_chirho {
            // At max depth, only terminals allowed
            self.leaf_domain_chirho()
        } else {
            // Any production allowed
            let mut domain_chirho = Hierarchical4kChirho::empty_chirho();
            for id_chirho in 0..self.num_productions_chirho {
                let leaf_idx_chirho = (id_chirho / 64) as usize;
                let bit_idx_chirho = id_chirho % 64;
                if leaf_idx_chirho < 64 {
                    domain_chirho.leaves_chirho[leaf_idx_chirho].0 |= 1u64 << bit_idx_chirho;
                    domain_chirho.root_chirho.0 |= 1u64 << leaf_idx_chirho;
                }
            }
            domain_chirho
        }
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;
    use crate::synthesis_chirho::sygus_chirho::SygusProblemChirho;

    #[test]
    fn test_encode_grammar_chirho() {
        let content_chirho = r#"
(synth-fun max2 ((x Int) (y Int)) Int)
(constraint (= (max2 0 1) 1))
"#;
        let problem_chirho = SygusProblemChirho::parse_chirho(content_chirho).unwrap();
        let encoded_chirho = EncodedGrammarChirho::from_sygus_chirho(&problem_chirho.grammar_chirho);

        assert!(encoded_chirho.num_productions_chirho > 0);
        assert!(!encoded_chirho.productions_chirho.is_empty());
    }

    #[test]
    fn test_leaf_domain_chirho() {
        let content_chirho = r#"
(synth-fun f ((x Int)) Int)
"#;
        let problem_chirho = SygusProblemChirho::parse_chirho(content_chirho).unwrap();
        let encoded_chirho = EncodedGrammarChirho::from_sygus_chirho(&problem_chirho.grammar_chirho);

        let leaf_domain_chirho = encoded_chirho.leaf_domain_chirho();
        assert!(leaf_domain_chirho.count_chirho() > 0);
    }
}
