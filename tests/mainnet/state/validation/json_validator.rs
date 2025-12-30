use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl Error for ValidationError {}

pub struct JsonValidator {
    required_fields: HashSet<String>,
    field_types: Map<String, String>,
    allowed_values: Map<String, Vec<Value>>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            field_types: Map::new(),
            allowed_values: Map::new(),
        }
    }

    pub fn require_field(mut self, field: &str) -> Self {
        self.required_fields.insert(field.to_string());
        self
    }

    pub fn set_field_type(mut self, field: &str, type_name: &str) -> Self {
        self.field_types.insert(field.to_string(), type_name.to_string());
        self
    }

    pub fn set_allowed_values(mut self, field: &str, values: Vec<Value>) -> Self {
        self.allowed_values.insert(field.to_string(), values);
        self
    }

    pub fn validate(&self, json: &Value) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Value::Object(obj) = json {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    errors.push(ValidationError {
                        path: field.clone(),
                        message: "Required field is missing".to_string(),
                    });
                }
            }

            for (field, value) in obj {
                self.validate_field(field, value, &mut errors);
            }
        } else {
            errors.push(ValidationError {
                path: "root".to_string(),
                message: "Expected JSON object".to_string(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_field(&self, field: &str, value: &Value, errors: &mut Vec<ValidationError>) {
        if let Some(expected_type) = self.field_types.get(field) {
            let actual_type = match value {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
            };

            if actual_type != expected_type {
                errors.push(ValidationError {
                    path: field.to_string(),
                    message: format!("Expected type '{}', got '{}'", expected_type, actual_type),
                });
            }
        }

        if let Some(allowed) = self.allowed_values.get(field) {
            if !allowed.contains(value) {
                errors.push(ValidationError {
                    path: field.to_string(),
                    message: "Value not in allowed set".to_string(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validation_success() {
        let validator = JsonValidator::new()
            .require_field("name")
            .set_field_type("name", "string")
            .set_field_type("age", "number")
            .set_allowed_values("status", vec![json!("active"), json!("inactive")]);

        let data = json!({
            "name": "John",
            "age": 30,
            "status": "active"
        });

        assert!(validator.validate(&data).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let validator = JsonValidator::new()
            .require_field("name")
            .set_field_type("age", "number");

        let data = json!({
            "age": "thirty"
        });

        let result = validator.validate(&data);
        assert!(result.is_err());
        
        if let Err(errors) = result {
            assert_eq!(errors.len(), 2);
            assert!(errors.iter().any(|e| e.path == "name"));
            assert!(errors.iter().any(|e| e.path == "age"));
        }
    }
}