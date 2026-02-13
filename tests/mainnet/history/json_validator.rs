use serde_json::Value;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum JsonValidationError {
    InvalidSyntax(String),
    MissingField(String),
    TypeMismatch { field: String, expected: String },
}

impl fmt::Display for JsonValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValidationError::InvalidSyntax(msg) => write!(f, "Invalid JSON syntax: {}", msg),
            JsonValidationError::MissingField(field) => write!(f, "Missing required field: {}", field),
            JsonValidationError::TypeMismatch { field, expected } => {
                write!(f, "Field '{}' must be of type {}", field, expected)
            }
        }
    }
}

impl Error for JsonValidationError {}

pub struct JsonValidator {
    required_fields: Vec<String>,
    field_types: Vec<(String, String)>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: Vec::new(),
            field_types: Vec::new(),
        }
    }

    pub fn require_field(mut self, field: &str) -> Self {
        self.required_fields.push(field.to_string());
        self
    }

    pub fn expect_type(mut self, field: &str, type_name: &str) -> Self {
        self.field_types.push((field.to_string(), type_name.to_string()));
        self
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, JsonValidationError> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| JsonValidationError::InvalidSyntax(e.to_string()))?;

        for field in &self.required_fields {
            if !value.get(field).is_some() {
                return Err(JsonValidationError::MissingField(field.clone()));
            }
        }

        for (field, expected_type) in &self.field_types {
            if let Some(field_value) = value.get(field) {
                let actual_type = match field_value {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                };

                if actual_type != expected_type {
                    return Err(JsonValidationError::TypeMismatch {
                        field: field.clone(),
                        expected: expected_type.clone(),
                    });
                }
            }
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let validator = JsonValidator::new()
            .require_field("name")
            .expect_type("name", "string")
            .expect_type("age", "number");

        let json = r#"{"name": "John", "age": 30}"#;
        let result = validator.validate(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_field() {
        let validator = JsonValidator::new().require_field("name");
        let json = r#"{"age": 30}"#;
        let result = validator.validate(json);
        assert!(matches!(result, Err(JsonValidationError::MissingField(_))));
    }

    #[test]
    fn test_type_mismatch() {
        let validator = JsonValidator::new().expect_type("age", "number");
        let json = r#"{"age": "thirty"}"#;
        let result = validator.validate(json);
        assert!(matches!(result, Err(JsonValidationError::TypeMismatch { .. })));
    }
}use serde_json::{Value, json};
use std::collections::HashSet;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: HashSet<&'static str>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: HashSet::from(["string", "number", "boolean", "object", "array", "null"]),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, String> {
        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        self.validate_structure(&parsed)?;
        Ok(parsed)
    }

    fn validate_structure(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::Object(map) => {
                for field in &self.required_fields {
                    if !map.contains_key(field) {
                        return Err(format!("Missing required field: {}", field));
                    }
                }

                for (key, val) in map {
                    self.validate_value_type(val)?;
                    println!("Validated field: {} = {:?}", key, val);
                }
                Ok(())
            }
            _ => Err("Expected JSON object at root level".to_string()),
        }
    }

    fn validate_value_type(&self, value: &Value) -> Result<(), String> {
        let type_name = match value {
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::Null => "null",
        };

        if !self.allowed_types.contains(type_name) {
            return Err(format!("Disallowed type: {}", type_name));
        }
        Ok(())
    }
}

pub fn create_sample_schema() -> Value {
    json!({
        "name": "Test User",
        "age": 30,
        "active": true,
        "metadata": {
            "role": "admin"
        },
        "tags": ["rust", "json"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        
        let json_data = r#"{"name": "Alice", "age": 25}"#;
        assert!(validator.validate(json_data).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("email");
        
        let json_data = r#"{"name": "Bob"}"#;
        assert!(validator.validate(json_data).is_err());
    }
}