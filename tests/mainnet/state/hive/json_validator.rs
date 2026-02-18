
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field_path: String,
    pub required: bool,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Null,
    Any,
}

pub struct JsonValidator {
    rules: Vec<ValidationRule>,
}

impl JsonValidator {
    pub fn new(rules: Vec<ValidationRule>) -> Self {
        Self { rules }
    }

    pub fn validate(&self, json_data: &Value) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut validated_paths = HashSet::new();

        for rule in &self.rules {
            let field_value = Self::get_field_by_path(json_data, &rule.field_path);

            match field_value {
                Some(value) => {
                    validated_paths.insert(rule.field_path.clone());

                    if !Self::check_data_type(value, &rule.data_type) {
                        errors.push(format!(
                            "Field '{}' has incorrect data type. Expected: {:?}",
                            rule.field_path, rule.data_type
                        ));
                    }
                }
                None => {
                    if rule.required {
                        errors.push(format!("Required field '{}' is missing", rule.field_path));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn get_field_by_path(json_data: &Value, path: &str) -> Option<&Value> {
        let mut current = json_data;
        for part in path.split('.') {
            current = current.get(part)?;
        }
        Some(current)
    }

    fn check_data_type(value: &Value, expected_type: &DataType) -> bool {
        match expected_type {
            DataType::String => value.is_string(),
            DataType::Number => value.is_number(),
            DataType::Boolean => value.is_boolean(),
            DataType::Object => value.is_object(),
            DataType::Array => value.is_array(),
            DataType::Null => value.is_null(),
            DataType::Any => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_json_validation() {
        let rules = vec![
            ValidationRule {
                field_path: "name".to_string(),
                required: true,
                data_type: DataType::String,
            },
            ValidationRule {
                field_path: "age".to_string(),
                required: false,
                data_type: DataType::Number,
            },
            ValidationRule {
                field_path: "address.city".to_string(),
                required: true,
                data_type: DataType::String,
            },
        ];

        let validator = JsonValidator::new(rules);
        let test_data = json!({
            "name": "John Doe",
            "age": 30,
            "address": {
                "city": "New York",
                "street": "123 Main St"
            }
        });

        let result = validator.validate(&test_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let rules = vec![ValidationRule {
            field_path: "email".to_string(),
            required: true,
            data_type: DataType::String,
        }];

        let validator = JsonValidator::new(rules);
        let test_data = json!({
            "name": "John Doe"
        });

        let result = validator.validate(&test_data);
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors[0].contains("Required field 'email' is missing"));
        }
    }

    #[test]
    fn test_incorrect_data_type() {
        let rules = vec![ValidationRule {
            field_path: "count".to_string(),
            required: true,
            data_type: DataType::Number,
        }];

        let validator = JsonValidator::new(rules);
        let test_data = json!({
            "count": "not_a_number"
        });

        let result = validator.validate(&test_data);
        assert!(result.is_err());
    }
}