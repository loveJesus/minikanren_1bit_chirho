// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! JSON Schema Validator using 1-Bit Domain Propagation ☧
//!
//! Validates JSON values against JSON Schema using constraint propagation.
//! Each constraint maps to domain operations:
//!
//! | Schema Constraint | Domain Operation |
//! |-------------------|------------------|
//! | `type: "string"` | Domain bit AND |
//! | `enum: [...]` | Domain = allowed values |
//! | `required` | Non-empty domain |
//! | `minimum/maximum` | Range constraint |
//! | `properties` | Recursive validation |
//!
//! "Prove all things; hold fast that which is good." — 1 Thessalonians 5:21

use std::collections::HashMap;

// ============================================================================
// JSON Value Representation
// ============================================================================

/// JSON value with efficient representation for validation
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValueChirho {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValueChirho>),
    Object(HashMap<String, JsonValueChirho>),
}

impl JsonValueChirho {
    /// Get type bits for this value (used in domain operations)
    pub fn type_bits_chirho(&self) -> u8 {
        match self {
            JsonValueChirho::Null => TYPE_NULL_CHIRHO,
            JsonValueChirho::Bool(_) => TYPE_BOOL_CHIRHO,
            JsonValueChirho::Number(_) => TYPE_NUMBER_CHIRHO,
            JsonValueChirho::String(_) => TYPE_STRING_CHIRHO,
            JsonValueChirho::Array(_) => TYPE_ARRAY_CHIRHO,
            JsonValueChirho::Object(_) => TYPE_OBJECT_CHIRHO,
        }
    }

    /// Check if this value is an integer (no fractional part)
    pub fn is_integer_chirho(&self) -> bool {
        match self {
            JsonValueChirho::Number(n) => n.fract() == 0.0,
            _ => false,
        }
    }
}

// Type bits (6 JSON types fit in 6 bits)
pub const TYPE_NULL_CHIRHO: u8 = 0b000001;
pub const TYPE_BOOL_CHIRHO: u8 = 0b000010;
pub const TYPE_NUMBER_CHIRHO: u8 = 0b000100;
pub const TYPE_STRING_CHIRHO: u8 = 0b001000;
pub const TYPE_ARRAY_CHIRHO: u8 = 0b010000;
pub const TYPE_OBJECT_CHIRHO: u8 = 0b100000;
pub const TYPE_INTEGER_CHIRHO: u8 = 0b000100; // Same as number, checked separately
pub const TYPE_ALL_CHIRHO: u8 = 0b111111;

// ============================================================================
// Schema Representation
// ============================================================================

/// JSON Schema with constraints as domain operations
#[derive(Debug, Clone, Default)]
pub struct SchemaChirho {
    /// Allowed types (bitmask)
    pub type_mask_chirho: u8,

    /// Enum constraint (allowed values)
    pub enum_values_chirho: Option<Vec<JsonValueChirho>>,

    /// Required property names (for objects)
    pub required_chirho: Vec<String>,

    /// Property schemas (for objects)
    pub properties_chirho: HashMap<String, SchemaChirho>,

    /// Additional properties schema (for objects)
    pub additional_properties_chirho: Option<Box<SchemaChirho>>,

    /// Items schema (for arrays)
    pub items_chirho: Option<Box<SchemaChirho>>,

    /// Number constraints
    pub minimum_chirho: Option<f64>,
    pub maximum_chirho: Option<f64>,
    pub exclusive_minimum_chirho: Option<f64>,
    pub exclusive_maximum_chirho: Option<f64>,
    pub multiple_of_chirho: Option<f64>,

    /// String constraints
    pub min_length_chirho: Option<usize>,
    pub max_length_chirho: Option<usize>,
    pub pattern_chirho: Option<String>,

    /// Array constraints
    pub min_items_chirho: Option<usize>,
    pub max_items_chirho: Option<usize>,
    pub unique_items_chirho: bool,

    /// Object constraints
    pub min_properties_chirho: Option<usize>,
    pub max_properties_chirho: Option<usize>,

    /// Constant value
    pub const_value_chirho: Option<JsonValueChirho>,

    /// Combinators
    pub all_of_chirho: Vec<SchemaChirho>,
    pub any_of_chirho: Vec<SchemaChirho>,
    pub one_of_chirho: Vec<SchemaChirho>,
    pub not_chirho: Option<Box<SchemaChirho>>,
}

impl SchemaChirho {
    /// Create empty schema (accepts anything)
    pub fn new_chirho() -> Self {
        Self {
            type_mask_chirho: TYPE_ALL_CHIRHO,
            ..Default::default()
        }
    }

    /// Create schema accepting only specific types
    pub fn typed_chirho(type_mask_chirho: u8) -> Self {
        Self {
            type_mask_chirho,
            ..Default::default()
        }
    }

    /// Create string schema
    pub fn string_chirho() -> Self {
        Self::typed_chirho(TYPE_STRING_CHIRHO)
    }

    /// Create number schema
    pub fn number_chirho() -> Self {
        Self::typed_chirho(TYPE_NUMBER_CHIRHO)
    }

    /// Create integer schema
    pub fn integer_chirho() -> Self {
        Self::typed_chirho(TYPE_INTEGER_CHIRHO)
    }

    /// Create boolean schema
    pub fn boolean_chirho() -> Self {
        Self::typed_chirho(TYPE_BOOL_CHIRHO)
    }

    /// Create null schema
    pub fn null_chirho() -> Self {
        Self::typed_chirho(TYPE_NULL_CHIRHO)
    }

    /// Create array schema
    pub fn array_chirho() -> Self {
        Self::typed_chirho(TYPE_ARRAY_CHIRHO)
    }

    /// Create object schema
    pub fn object_chirho() -> Self {
        Self::typed_chirho(TYPE_OBJECT_CHIRHO)
    }

    /// Set items schema (for arrays)
    pub fn with_items_chirho(mut self, items_chirho: SchemaChirho) -> Self {
        self.items_chirho = Some(Box::new(items_chirho));
        self
    }

    /// Add a property schema
    pub fn with_property_chirho(mut self, name_chirho: &str, schema_chirho: SchemaChirho) -> Self {
        self.properties_chirho.insert(name_chirho.to_string(), schema_chirho);
        self
    }

    /// Add required property
    pub fn with_required_chirho(mut self, names_chirho: &[&str]) -> Self {
        self.required_chirho = names_chirho.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set minimum constraint
    pub fn with_minimum_chirho(mut self, min_chirho: f64) -> Self {
        self.minimum_chirho = Some(min_chirho);
        self
    }

    /// Set maximum constraint
    pub fn with_maximum_chirho(mut self, max_chirho: f64) -> Self {
        self.maximum_chirho = Some(max_chirho);
        self
    }

    /// Set min length (for strings)
    pub fn with_min_length_chirho(mut self, min_chirho: usize) -> Self {
        self.min_length_chirho = Some(min_chirho);
        self
    }

    /// Set max length (for strings)
    pub fn with_max_length_chirho(mut self, max_chirho: usize) -> Self {
        self.max_length_chirho = Some(max_chirho);
        self
    }

    /// Set enum values
    pub fn with_enum_chirho(mut self, values_chirho: Vec<JsonValueChirho>) -> Self {
        self.enum_values_chirho = Some(values_chirho);
        self
    }

    /// Set min items (for arrays)
    pub fn with_min_items_chirho(mut self, min_chirho: usize) -> Self {
        self.min_items_chirho = Some(min_chirho);
        self
    }

    /// Set max items (for arrays)
    pub fn with_max_items_chirho(mut self, max_chirho: usize) -> Self {
        self.max_items_chirho = Some(max_chirho);
        self
    }
}

// ============================================================================
// Validation Errors
// ============================================================================

/// Validation error with path information
#[derive(Debug, Clone)]
pub struct ValidationErrorChirho {
    pub path_chirho: String,
    pub message_chirho: String,
    pub keyword_chirho: &'static str,
}

impl ValidationErrorChirho {
    fn new_chirho(path_chirho: &str, keyword_chirho: &'static str, message_chirho: String) -> Self {
        Self {
            path_chirho: path_chirho.to_string(),
            message_chirho,
            keyword_chirho,
        }
    }
}

impl std::fmt::Display for ValidationErrorChirho {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({})", self.path_chirho, self.message_chirho, self.keyword_chirho)
    }
}

// ============================================================================
// Validator
// ============================================================================

/// JSON Schema validator using domain propagation
pub struct ValidatorChirho {
    pub errors_chirho: Vec<ValidationErrorChirho>,
}

impl ValidatorChirho {
    pub fn new_chirho() -> Self {
        Self { errors_chirho: Vec::new() }
    }

    /// Validate a value against a schema
    pub fn validate_chirho(
        &mut self,
        value_chirho: &JsonValueChirho,
        schema_chirho: &SchemaChirho,
    ) -> bool {
        self.errors_chirho.clear();
        self.validate_internal_chirho(value_chirho, schema_chirho, "$")
    }

    fn validate_internal_chirho(
        &mut self,
        value_chirho: &JsonValueChirho,
        schema_chirho: &SchemaChirho,
        path_chirho: &str,
    ) -> bool {
        let mut valid_chirho = true;

        // Type check (domain bit AND)
        valid_chirho &= self.check_type_chirho(value_chirho, schema_chirho, path_chirho);

        // Const check
        if let Some(ref const_val_chirho) = schema_chirho.const_value_chirho {
            if value_chirho != const_val_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "const", "value does not match const".to_string()
                ));
                valid_chirho = false;
            }
        }

        // Enum check (domain = allowed values)
        if let Some(ref enum_vals_chirho) = schema_chirho.enum_values_chirho {
            if !enum_vals_chirho.contains(value_chirho) {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "enum", "value not in enum".to_string()
                ));
                valid_chirho = false;
            }
        }

        // Type-specific validation
        match value_chirho {
            JsonValueChirho::Number(n) => {
                valid_chirho &= self.validate_number_chirho(*n, schema_chirho, path_chirho);
            }
            JsonValueChirho::String(s) => {
                valid_chirho &= self.validate_string_chirho(s, schema_chirho, path_chirho);
            }
            JsonValueChirho::Array(arr) => {
                valid_chirho &= self.validate_array_chirho(arr, schema_chirho, path_chirho);
            }
            JsonValueChirho::Object(obj) => {
                valid_chirho &= self.validate_object_chirho(obj, schema_chirho, path_chirho);
            }
            _ => {}
        }

        // Combinators
        valid_chirho &= self.validate_combinators_chirho(value_chirho, schema_chirho, path_chirho);

        valid_chirho
    }

    fn check_type_chirho(
        &mut self,
        value_chirho: &JsonValueChirho,
        schema_chirho: &SchemaChirho,
        path_chirho: &str,
    ) -> bool {
        let value_type_chirho = value_chirho.type_bits_chirho();
        let allowed_chirho = schema_chirho.type_mask_chirho;

        // Domain intersection: value_type AND allowed
        if (value_type_chirho & allowed_chirho) == 0 {
            self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                path_chirho, "type", format!("expected type mask {:06b}, got {:06b}", allowed_chirho, value_type_chirho)
            ));
            return false;
        }

        // Special case: integer check
        if allowed_chirho == TYPE_INTEGER_CHIRHO {
            if let JsonValueChirho::Number(n) = value_chirho {
                if n.fract() != 0.0 {
                    self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                        path_chirho, "type", "expected integer, got float".to_string()
                    ));
                    return false;
                }
            }
        }

        true
    }

    fn validate_number_chirho(
        &mut self,
        n_chirho: f64,
        schema_chirho: &SchemaChirho,
        path_chirho: &str,
    ) -> bool {
        let mut valid_chirho = true;

        if let Some(min_chirho) = schema_chirho.minimum_chirho {
            if n_chirho < min_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "minimum", format!("{} < minimum {}", n_chirho, min_chirho)
                ));
                valid_chirho = false;
            }
        }

        if let Some(max_chirho) = schema_chirho.maximum_chirho {
            if n_chirho > max_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "maximum", format!("{} > maximum {}", n_chirho, max_chirho)
                ));
                valid_chirho = false;
            }
        }

        if let Some(ex_min_chirho) = schema_chirho.exclusive_minimum_chirho {
            if n_chirho <= ex_min_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "exclusiveMinimum", format!("{} <= exclusiveMinimum {}", n_chirho, ex_min_chirho)
                ));
                valid_chirho = false;
            }
        }

        if let Some(ex_max_chirho) = schema_chirho.exclusive_maximum_chirho {
            if n_chirho >= ex_max_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "exclusiveMaximum", format!("{} >= exclusiveMaximum {}", n_chirho, ex_max_chirho)
                ));
                valid_chirho = false;
            }
        }

        if let Some(mult_chirho) = schema_chirho.multiple_of_chirho {
            if mult_chirho != 0.0 && (n_chirho / mult_chirho).fract() != 0.0 {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "multipleOf", format!("{} is not a multiple of {}", n_chirho, mult_chirho)
                ));
                valid_chirho = false;
            }
        }

        valid_chirho
    }

    fn validate_string_chirho(
        &mut self,
        s_chirho: &str,
        schema_chirho: &SchemaChirho,
        path_chirho: &str,
    ) -> bool {
        let mut valid_chirho = true;
        let len_chirho = s_chirho.chars().count();

        if let Some(min_chirho) = schema_chirho.min_length_chirho {
            if len_chirho < min_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "minLength", format!("length {} < minLength {}", len_chirho, min_chirho)
                ));
                valid_chirho = false;
            }
        }

        if let Some(max_chirho) = schema_chirho.max_length_chirho {
            if len_chirho > max_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "maxLength", format!("length {} > maxLength {}", len_chirho, max_chirho)
                ));
                valid_chirho = false;
            }
        }

        // Note: Pattern validation would require regex crate
        // For now, we store the pattern but don't validate it
        // In a full implementation, add regex dependency

        valid_chirho
    }

    fn validate_array_chirho(
        &mut self,
        arr_chirho: &[JsonValueChirho],
        schema_chirho: &SchemaChirho,
        path_chirho: &str,
    ) -> bool {
        let mut valid_chirho = true;
        let len_chirho = arr_chirho.len();

        if let Some(min_chirho) = schema_chirho.min_items_chirho {
            if len_chirho < min_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "minItems", format!("array length {} < minItems {}", len_chirho, min_chirho)
                ));
                valid_chirho = false;
            }
        }

        if let Some(max_chirho) = schema_chirho.max_items_chirho {
            if len_chirho > max_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "maxItems", format!("array length {} > maxItems {}", len_chirho, max_chirho)
                ));
                valid_chirho = false;
            }
        }

        if schema_chirho.unique_items_chirho {
            // O(n²) uniqueness check (could optimize with hash)
            for i in 0..len_chirho {
                for j in (i + 1)..len_chirho {
                    if arr_chirho[i] == arr_chirho[j] {
                        self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                            path_chirho, "uniqueItems", format!("duplicate items at indices {} and {}", i, j)
                        ));
                        valid_chirho = false;
                    }
                }
            }
        }

        // Validate items
        if let Some(ref items_schema_chirho) = schema_chirho.items_chirho {
            for (i_chirho, item_chirho) in arr_chirho.iter().enumerate() {
                let item_path_chirho = format!("{}[{}]", path_chirho, i_chirho);
                valid_chirho &= self.validate_internal_chirho(
                    item_chirho,
                    items_schema_chirho,
                    &item_path_chirho,
                );
            }
        }

        valid_chirho
    }

    fn validate_object_chirho(
        &mut self,
        obj_chirho: &HashMap<String, JsonValueChirho>,
        schema_chirho: &SchemaChirho,
        path_chirho: &str,
    ) -> bool {
        let mut valid_chirho = true;
        let len_chirho = obj_chirho.len();

        // Check required properties
        for req_chirho in &schema_chirho.required_chirho {
            if !obj_chirho.contains_key(req_chirho) {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "required", format!("missing required property '{}'", req_chirho)
                ));
                valid_chirho = false;
            }
        }

        // Check min/max properties
        if let Some(min_chirho) = schema_chirho.min_properties_chirho {
            if len_chirho < min_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "minProperties", format!("object has {} properties, minProperties is {}", len_chirho, min_chirho)
                ));
                valid_chirho = false;
            }
        }

        if let Some(max_chirho) = schema_chirho.max_properties_chirho {
            if len_chirho > max_chirho {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "maxProperties", format!("object has {} properties, maxProperties is {}", len_chirho, max_chirho)
                ));
                valid_chirho = false;
            }
        }

        // Validate known properties
        for (key_chirho, value_chirho) in obj_chirho {
            let prop_path_chirho = format!("{}.{}", path_chirho, key_chirho);

            if let Some(prop_schema_chirho) = schema_chirho.properties_chirho.get(key_chirho) {
                valid_chirho &= self.validate_internal_chirho(
                    value_chirho,
                    prop_schema_chirho,
                    &prop_path_chirho,
                );
            } else if let Some(ref add_schema_chirho) = schema_chirho.additional_properties_chirho {
                // Validate against additionalProperties schema
                valid_chirho &= self.validate_internal_chirho(
                    value_chirho,
                    add_schema_chirho,
                    &prop_path_chirho,
                );
            }
        }

        valid_chirho
    }

    fn validate_combinators_chirho(
        &mut self,
        value_chirho: &JsonValueChirho,
        schema_chirho: &SchemaChirho,
        path_chirho: &str,
    ) -> bool {
        let mut valid_chirho = true;

        // allOf: all schemas must match
        if !schema_chirho.all_of_chirho.is_empty() {
            for sub_schema_chirho in &schema_chirho.all_of_chirho {
                valid_chirho &= self.validate_internal_chirho(value_chirho, sub_schema_chirho, path_chirho);
            }
        }

        // anyOf: at least one schema must match
        if !schema_chirho.any_of_chirho.is_empty() {
            let mut any_match_chirho = false;
            let original_errors_chirho = self.errors_chirho.len();

            for sub_schema_chirho in &schema_chirho.any_of_chirho {
                let mut sub_validator_chirho = ValidatorChirho::new_chirho();
                if sub_validator_chirho.validate_internal_chirho(value_chirho, sub_schema_chirho, path_chirho) {
                    any_match_chirho = true;
                    break;
                }
            }

            if !any_match_chirho {
                // Keep only original errors, add anyOf error
                self.errors_chirho.truncate(original_errors_chirho);
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "anyOf", "value does not match any schema".to_string()
                ));
                valid_chirho = false;
            }
        }

        // oneOf: exactly one schema must match
        if !schema_chirho.one_of_chirho.is_empty() {
            let mut match_count_chirho = 0;

            for sub_schema_chirho in &schema_chirho.one_of_chirho {
                let mut sub_validator_chirho = ValidatorChirho::new_chirho();
                if sub_validator_chirho.validate_internal_chirho(value_chirho, sub_schema_chirho, path_chirho) {
                    match_count_chirho += 1;
                }
            }

            if match_count_chirho != 1 {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "oneOf", format!("value matches {} schemas, expected exactly 1", match_count_chirho)
                ));
                valid_chirho = false;
            }
        }

        // not: schema must NOT match
        if let Some(ref not_schema_chirho) = schema_chirho.not_chirho {
            let mut sub_validator_chirho = ValidatorChirho::new_chirho();
            if sub_validator_chirho.validate_internal_chirho(value_chirho, not_schema_chirho, path_chirho) {
                self.errors_chirho.push(ValidationErrorChirho::new_chirho(
                    path_chirho, "not", "value matches the 'not' schema".to_string()
                ));
                valid_chirho = false;
            }
        }

        valid_chirho
    }
}

impl Default for ValidatorChirho {
    fn default() -> Self {
        Self::new_chirho()
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Validate JSON value against schema (convenience function)
pub fn validate_chirho(
    value_chirho: &JsonValueChirho,
    schema_chirho: &SchemaChirho,
) -> Result<(), Vec<ValidationErrorChirho>> {
    let mut validator_chirho = ValidatorChirho::new_chirho();
    if validator_chirho.validate_chirho(value_chirho, schema_chirho) {
        Ok(())
    } else {
        Err(validator_chirho.errors_chirho)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_type_string_chirho() {
        let schema_chirho = SchemaChirho::string_chirho();
        let valid_chirho = JsonValueChirho::String("hello".to_string());
        let invalid_chirho = JsonValueChirho::Number(42.0);

        assert!(validate_chirho(&valid_chirho, &schema_chirho).is_ok());
        assert!(validate_chirho(&invalid_chirho, &schema_chirho).is_err());
    }

    #[test]
    fn test_type_number_chirho() {
        let schema_chirho = SchemaChirho::number_chirho()
            .with_minimum_chirho(0.0)
            .with_maximum_chirho(100.0);

        assert!(validate_chirho(&JsonValueChirho::Number(50.0), &schema_chirho).is_ok());
        assert!(validate_chirho(&JsonValueChirho::Number(-1.0), &schema_chirho).is_err());
        assert!(validate_chirho(&JsonValueChirho::Number(101.0), &schema_chirho).is_err());
    }

    #[test]
    fn test_type_integer_chirho() {
        let schema_chirho = SchemaChirho::integer_chirho();

        assert!(validate_chirho(&JsonValueChirho::Number(42.0), &schema_chirho).is_ok());
        assert!(validate_chirho(&JsonValueChirho::Number(42.5), &schema_chirho).is_err());
    }

    #[test]
    fn test_string_length_chirho() {
        let schema_chirho = SchemaChirho::string_chirho()
            .with_min_length_chirho(2)
            .with_max_length_chirho(5);

        assert!(validate_chirho(&JsonValueChirho::String("ab".to_string()), &schema_chirho).is_ok());
        assert!(validate_chirho(&JsonValueChirho::String("abcde".to_string()), &schema_chirho).is_ok());
        assert!(validate_chirho(&JsonValueChirho::String("a".to_string()), &schema_chirho).is_err());
        assert!(validate_chirho(&JsonValueChirho::String("abcdef".to_string()), &schema_chirho).is_err());
    }

    #[test]
    fn test_enum_chirho() {
        let schema_chirho = SchemaChirho::string_chirho()
            .with_enum_chirho(vec![
                JsonValueChirho::String("red".to_string()),
                JsonValueChirho::String("green".to_string()),
                JsonValueChirho::String("blue".to_string()),
            ]);

        assert!(validate_chirho(&JsonValueChirho::String("red".to_string()), &schema_chirho).is_ok());
        assert!(validate_chirho(&JsonValueChirho::String("yellow".to_string()), &schema_chirho).is_err());
    }

    #[test]
    fn test_array_chirho() {
        let schema_chirho = SchemaChirho::array_chirho()
            .with_items_chirho(SchemaChirho::number_chirho())
            .with_min_items_chirho(1)
            .with_max_items_chirho(3);

        let valid_chirho = JsonValueChirho::Array(vec![
            JsonValueChirho::Number(1.0),
            JsonValueChirho::Number(2.0),
        ]);
        let empty_chirho = JsonValueChirho::Array(vec![]);
        let too_many_chirho = JsonValueChirho::Array(vec![
            JsonValueChirho::Number(1.0),
            JsonValueChirho::Number(2.0),
            JsonValueChirho::Number(3.0),
            JsonValueChirho::Number(4.0),
        ]);
        let wrong_type_chirho = JsonValueChirho::Array(vec![
            JsonValueChirho::String("not a number".to_string()),
        ]);

        assert!(validate_chirho(&valid_chirho, &schema_chirho).is_ok());
        assert!(validate_chirho(&empty_chirho, &schema_chirho).is_err());
        assert!(validate_chirho(&too_many_chirho, &schema_chirho).is_err());
        assert!(validate_chirho(&wrong_type_chirho, &schema_chirho).is_err());
    }

    #[test]
    fn test_object_required_chirho() {
        let schema_chirho = SchemaChirho::object_chirho()
            .with_property_chirho("name", SchemaChirho::string_chirho())
            .with_property_chirho("age", SchemaChirho::number_chirho())
            .with_required_chirho(&["name"]);

        let valid_chirho = JsonValueChirho::Object({
            let mut m = HashMap::new();
            m.insert("name".to_string(), JsonValueChirho::String("Alice".to_string()));
            m.insert("age".to_string(), JsonValueChirho::Number(30.0));
            m
        });

        let missing_required_chirho = JsonValueChirho::Object({
            let mut m = HashMap::new();
            m.insert("age".to_string(), JsonValueChirho::Number(30.0));
            m
        });

        assert!(validate_chirho(&valid_chirho, &schema_chirho).is_ok());
        assert!(validate_chirho(&missing_required_chirho, &schema_chirho).is_err());
    }

    #[test]
    fn test_nested_object_chirho() {
        let address_schema_chirho = SchemaChirho::object_chirho()
            .with_property_chirho("city", SchemaChirho::string_chirho())
            .with_property_chirho("zip", SchemaChirho::string_chirho())
            .with_required_chirho(&["city"]);

        let person_schema_chirho = SchemaChirho::object_chirho()
            .with_property_chirho("name", SchemaChirho::string_chirho())
            .with_property_chirho("address", address_schema_chirho)
            .with_required_chirho(&["name", "address"]);

        let valid_chirho = JsonValueChirho::Object({
            let mut p = HashMap::new();
            p.insert("name".to_string(), JsonValueChirho::String("Bob".to_string()));
            p.insert("address".to_string(), JsonValueChirho::Object({
                let mut a = HashMap::new();
                a.insert("city".to_string(), JsonValueChirho::String("NYC".to_string()));
                a.insert("zip".to_string(), JsonValueChirho::String("10001".to_string()));
                a
            }));
            p
        });

        let missing_city_chirho = JsonValueChirho::Object({
            let mut p = HashMap::new();
            p.insert("name".to_string(), JsonValueChirho::String("Bob".to_string()));
            p.insert("address".to_string(), JsonValueChirho::Object({
                let mut a = HashMap::new();
                a.insert("zip".to_string(), JsonValueChirho::String("10001".to_string()));
                a
            }));
            p
        });

        assert!(validate_chirho(&valid_chirho, &person_schema_chirho).is_ok());
        assert!(validate_chirho(&missing_city_chirho, &person_schema_chirho).is_err());
    }

    #[test]
    fn test_any_of_chirho() {
        let mut schema_chirho = SchemaChirho::new_chirho();
        schema_chirho.any_of_chirho = vec![
            SchemaChirho::string_chirho(),
            SchemaChirho::number_chirho(),
        ];

        assert!(validate_chirho(&JsonValueChirho::String("hello".to_string()), &schema_chirho).is_ok());
        assert!(validate_chirho(&JsonValueChirho::Number(42.0), &schema_chirho).is_ok());
        assert!(validate_chirho(&JsonValueChirho::Bool(true), &schema_chirho).is_err());
    }

    #[test]
    fn test_not_chirho() {
        let mut schema_chirho = SchemaChirho::new_chirho();
        schema_chirho.not_chirho = Some(Box::new(SchemaChirho::null_chirho()));

        assert!(validate_chirho(&JsonValueChirho::String("hello".to_string()), &schema_chirho).is_ok());
        assert!(validate_chirho(&JsonValueChirho::Number(42.0), &schema_chirho).is_ok());
        assert!(validate_chirho(&JsonValueChirho::Null, &schema_chirho).is_err());
    }

    #[test]
    fn test_unique_items_chirho() {
        let mut schema_chirho = SchemaChirho::array_chirho();
        schema_chirho.unique_items_chirho = true;

        let unique_chirho = JsonValueChirho::Array(vec![
            JsonValueChirho::Number(1.0),
            JsonValueChirho::Number(2.0),
            JsonValueChirho::Number(3.0),
        ]);

        let duplicate_chirho = JsonValueChirho::Array(vec![
            JsonValueChirho::Number(1.0),
            JsonValueChirho::Number(2.0),
            JsonValueChirho::Number(1.0),
        ]);

        assert!(validate_chirho(&unique_chirho, &schema_chirho).is_ok());
        assert!(validate_chirho(&duplicate_chirho, &schema_chirho).is_err());
    }
}
