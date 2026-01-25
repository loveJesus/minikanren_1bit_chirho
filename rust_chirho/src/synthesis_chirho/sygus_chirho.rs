//! SyGuS Parser ☧
//!
//! Parses SyGuS (Syntax-Guided Synthesis) .sl files into our internal
//! representation for domain-based synthesis.

use std::collections::HashMap;

/// A SyGuS synthesis problem
#[derive(Debug, Clone)]
pub struct SygusProblemChirho {
    /// Name of the function to synthesize
    pub func_name_chirho: String,
    /// Input parameter names and types
    pub params_chirho: Vec<(String, SygusTypeChirho)>,
    /// Return type
    pub return_type_chirho: SygusTypeChirho,
    /// Grammar for synthesis
    pub grammar_chirho: SygusGrammarChirho,
    /// I/O constraints
    pub constraints_chirho: Vec<SygusConstraintChirho>,
}

/// SyGuS type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SygusTypeChirho {
    IntChirho,
    BoolChirho,
    StringChirho,
    BitVecChirho(u32),
}

/// Grammar for synthesis
#[derive(Debug, Clone)]
pub struct SygusGrammarChirho {
    /// Non-terminals and their productions
    pub rules_chirho: HashMap<String, Vec<SygusProductionChirho>>,
    /// Start symbol
    pub start_chirho: String,
}

/// A single grammar production
#[derive(Debug, Clone)]
pub enum SygusProductionChirho {
    /// Variable reference
    VarChirho(String),
    /// Integer constant
    IntConstChirho(i64),
    /// Boolean constant
    BoolConstChirho(bool),
    /// Binary operation: (op arg1 arg2)
    BinOpChirho {
        op_chirho: String,
        arg1_chirho: String,
        arg2_chirho: String,
    },
    /// Unary operation: (op arg)
    UnaryOpChirho {
        op_chirho: String,
        arg_chirho: String,
    },
    /// If-then-else: (ite cond then else)
    IteChirho {
        cond_chirho: String,
        then_chirho: String,
        else_chirho: String,
    },
    /// Constant placeholder (to be filled)
    ConstantPlaceholderChirho(SygusTypeChirho),
}

/// A constraint (I/O example or property)
#[derive(Debug, Clone)]
pub enum SygusConstraintChirho {
    /// (= (f x y) expected)
    IoExampleChirho {
        inputs_chirho: Vec<i64>,
        output_chirho: i64,
    },
    /// General constraint (not yet fully supported)
    GeneralChirho(String),
}

impl SygusProblemChirho {
    /// Parse a SyGuS problem from string content
    pub fn parse_chirho(content_chirho: &str) -> Result<Self, String> {
        let mut lines_chirho = content_chirho.lines().peekable();
        let mut func_name_chirho = String::new();
        let mut params_chirho = Vec::new();
        let mut return_type_chirho = SygusTypeChirho::IntChirho;
        let mut grammar_chirho = SygusGrammarChirho {
            rules_chirho: HashMap::new(),
            start_chirho: "Start".to_string(),
        };
        let mut constraints_chirho = Vec::new();

        // Simple line-by-line parser for SyGuS subset
        while let Some(line_chirho) = lines_chirho.next() {
            let trimmed_chirho = line_chirho.trim();

            if trimmed_chirho.starts_with("(synth-fun") {
                // Parse synth-fun declaration
                let parts_chirho: Vec<&str> = trimmed_chirho.split_whitespace().collect();
                if parts_chirho.len() >= 2 {
                    func_name_chirho = parts_chirho[1].to_string();
                }
                // Extract parameters from ((x Int) (y Int))
                if let Some(params_start_chirho) = trimmed_chirho.find("((") {
                    let params_section_chirho = &trimmed_chirho[params_start_chirho..];
                    params_chirho = Self::parse_params_chirho(params_section_chirho);
                }
                // Find return type
                return_type_chirho = Self::extract_return_type_chirho(trimmed_chirho);
            } else if trimmed_chirho.starts_with("(constraint") {
                // Parse constraint
                if let Some(constraint_chirho) = Self::parse_constraint_chirho(trimmed_chirho, &func_name_chirho) {
                    constraints_chirho.push(constraint_chirho);
                }
            } else if trimmed_chirho.starts_with("(declare") {
                // Skip declarations for now
            }
        }

        // Build default grammar if not specified
        if grammar_chirho.rules_chirho.is_empty() {
            grammar_chirho = Self::default_grammar_chirho(&params_chirho, &return_type_chirho);
        }

        Ok(SygusProblemChirho {
            func_name_chirho,
            params_chirho,
            return_type_chirho,
            grammar_chirho,
            constraints_chirho,
        })
    }

    fn parse_params_chirho(section_chirho: &str) -> Vec<(String, SygusTypeChirho)> {
        let mut params_chirho = Vec::new();
        let mut depth_chirho = 0;
        let mut current_chirho = String::new();

        for c_chirho in section_chirho.chars() {
            match c_chirho {
                '(' => {
                    depth_chirho += 1;
                    if depth_chirho > 1 {
                        current_chirho.push(c_chirho);
                    }
                }
                ')' => {
                    depth_chirho -= 1;
                    if depth_chirho == 1 {
                        // Parse single param like "x Int"
                        let parts_chirho: Vec<&str> = current_chirho.trim().split_whitespace().collect();
                        if parts_chirho.len() >= 2 {
                            let type_chirho = match parts_chirho[1] {
                                "Int" => SygusTypeChirho::IntChirho,
                                "Bool" => SygusTypeChirho::BoolChirho,
                                _ => SygusTypeChirho::IntChirho,
                            };
                            params_chirho.push((parts_chirho[0].to_string(), type_chirho));
                        }
                        current_chirho.clear();
                    } else if depth_chirho == 0 {
                        break;
                    } else {
                        current_chirho.push(c_chirho);
                    }
                }
                _ => {
                    if depth_chirho > 1 {
                        current_chirho.push(c_chirho);
                    }
                }
            }
        }

        params_chirho
    }

    fn extract_return_type_chirho(line_chirho: &str) -> SygusTypeChirho {
        // Look for )) Int pattern
        if line_chirho.contains(")) Int") {
            SygusTypeChirho::IntChirho
        } else if line_chirho.contains(")) Bool") {
            SygusTypeChirho::BoolChirho
        } else {
            SygusTypeChirho::IntChirho
        }
    }

    fn parse_constraint_chirho(line_chirho: &str, func_name_chirho: &str) -> Option<SygusConstraintChirho> {
        // Parse (constraint (= (func x y) result))
        // Simple pattern matching for I/O examples
        if line_chirho.contains(&format!("({}", func_name_chirho)) {
            // Extract numbers from the constraint
            let nums_chirho: Vec<i64> = line_chirho
                .split(|c_chirho: char| !c_chirho.is_ascii_digit() && c_chirho != '-')
                .filter_map(|s_chirho| s_chirho.parse().ok())
                .collect();

            if nums_chirho.len() >= 2 {
                let output_chirho = *nums_chirho.last().unwrap();
                let inputs_chirho = nums_chirho[..nums_chirho.len() - 1].to_vec();
                return Some(SygusConstraintChirho::IoExampleChirho {
                    inputs_chirho,
                    output_chirho,
                });
            }
        }

        Some(SygusConstraintChirho::GeneralChirho(line_chirho.to_string()))
    }

    fn default_grammar_chirho(
        params_chirho: &[(String, SygusTypeChirho)],
        _return_type_chirho: &SygusTypeChirho,
    ) -> SygusGrammarChirho {
        let mut rules_chirho = HashMap::new();

        // Start symbol
        let mut start_prods_chirho = Vec::new();

        // Add variables
        for (name_chirho, _type_chirho) in params_chirho {
            start_prods_chirho.push(SygusProductionChirho::VarChirho(name_chirho.clone()));
        }

        // Add constants
        for i_chirho in 0..=2 {
            start_prods_chirho.push(SygusProductionChirho::IntConstChirho(i_chirho));
        }

        // Add binary ops
        for op_chirho in ["+", "-", "*", "<=", ">=", "<", ">", "="] {
            start_prods_chirho.push(SygusProductionChirho::BinOpChirho {
                op_chirho: op_chirho.to_string(),
                arg1_chirho: "Start".to_string(),
                arg2_chirho: "Start".to_string(),
            });
        }

        // Add ite
        start_prods_chirho.push(SygusProductionChirho::IteChirho {
            cond_chirho: "StartBool".to_string(),
            then_chirho: "Start".to_string(),
            else_chirho: "Start".to_string(),
        });

        rules_chirho.insert("Start".to_string(), start_prods_chirho);

        // Boolean productions
        let mut bool_prods_chirho = Vec::new();
        for op_chirho in ["<=", ">=", "<", ">", "="] {
            bool_prods_chirho.push(SygusProductionChirho::BinOpChirho {
                op_chirho: op_chirho.to_string(),
                arg1_chirho: "Start".to_string(),
                arg2_chirho: "Start".to_string(),
            });
        }
        rules_chirho.insert("StartBool".to_string(), bool_prods_chirho);

        SygusGrammarChirho {
            rules_chirho,
            start_chirho: "Start".to_string(),
        }
    }

    /// Get the number of I/O examples
    pub fn num_examples_chirho(&self) -> usize {
        self.constraints_chirho
            .iter()
            .filter(|c_chirho| matches!(c_chirho, SygusConstraintChirho::IoExampleChirho { .. }))
            .count()
    }

    /// Get I/O examples as (inputs, output) pairs
    pub fn io_examples_chirho(&self) -> Vec<(Vec<i64>, i64)> {
        self.constraints_chirho
            .iter()
            .filter_map(|c_chirho| {
                if let SygusConstraintChirho::IoExampleChirho { inputs_chirho, output_chirho } = c_chirho {
                    Some((inputs_chirho.clone(), *output_chirho))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_parse_max2_chirho() {
        let content_chirho = r#"
(synth-fun max2 ((x Int) (y Int)) Int)
(constraint (= (max2 0 1) 1))
(constraint (= (max2 1 0) 1))
(constraint (= (max2 3 5) 5))
"#;
        let problem_chirho = SygusProblemChirho::parse_chirho(content_chirho).unwrap();
        assert_eq!(problem_chirho.func_name_chirho, "max2");
        assert_eq!(problem_chirho.params_chirho.len(), 2);
        assert_eq!(problem_chirho.num_examples_chirho(), 3);
    }
}
