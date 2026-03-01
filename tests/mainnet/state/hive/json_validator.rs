
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
}use serde_json::{Value, from_str};
use std::fs;

pub fn validate_json_schema(file_path: &str, schema: &Value) -> Result<(), String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let data: Value = from_str(&content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    if !is_subset(&data, schema) {
        return Err("JSON does not match required schema".to_string());
    }
    
    Ok(())
}

fn is_subset(data: &Value, schema: &Value) -> bool {
    match (data, schema) {
        (Value::Object(data_map), Value::Object(schema_map)) => {
            for (key, schema_value) in schema_map {
                match data_map.get(key) {
                    Some(data_value) => {
                        if !is_subset(data_value, schema_value) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            true
        }
        (Value::Array(data_arr), Value::Array(schema_arr)) => {
            if data_arr.len() != schema_arr.len() {
                return false;
            }
            data_arr.iter().zip(schema_arr.iter()).all(|(d, s)| is_subset(d, s))
        }
        (Value::String(_), Value::String(_)) => true,
        (Value::Number(_), Value::Number(_)) => true,
        (Value::Bool(_), Value::Bool(_)) => true,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_config() {
        let schema = json!({
            "name": "",
            "version": 0,
            "features": []
        });
        
        let temp_file = "temp_config.json";
        fs::write(temp_file, r#"{"name": "test", "version": 1, "features": ["a", "b"]}"#).unwrap();
        
        assert!(validate_json_schema(temp_file, &schema).is_ok());
        fs::remove_file(temp_file).unwrap();
    }
}