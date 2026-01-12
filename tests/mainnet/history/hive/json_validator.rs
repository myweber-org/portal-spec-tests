use serde_json::Value;
use std::collections::HashSet;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: HashSet<&'static str>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: HashSet::from(["string", "number", "boolean", "object", "array"]),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn validate(&self, json_str: &str) -> Result<(), String> {
        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        self.validate_structure(&parsed)?;
        self.validate_required_fields(&parsed)?;

        Ok(())
    }

    fn validate_structure(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::Object(map) => {
                for (_, v) in map {
                    self.validate_structure(v)?;
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.validate_structure(item)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_required_fields(&self, value: &Value) -> Result<(), String> {
        if let Value::Object(map) = value {
            for field in &self.required_fields {
                if !map.contains_key(field) {
                    return Err(format!("Missing required field: {}", field));
                }
            }
        }
        Ok(())
    }

    pub fn get_field_type(value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        
        let json = r#"{"name": "test", "value": 42}"#;
        assert!(validator.validate(json).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        
        let json = r#"{"value": 42}"#;
        assert!(validator.validate(json).is_err());
    }
}