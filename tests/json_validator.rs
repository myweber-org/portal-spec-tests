use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &str, data: &str) -> Result<(), Vec<String>> {
    let schema_value: Value = serde_json::from_str(schema)
        .map_err(|e| vec![format!("Invalid schema: {}", e)])?;
    
    let data_value: Value = serde_json::from_str(data)
        .map_err(|e| vec![format!("Invalid JSON data: {}", e)])?;
    
    let compiled_schema = JSONSchema::compile(&schema_value)
        .map_err(|e| vec![format!("Schema compilation failed: {}", e)])?;
    
    let validation_result = compiled_schema.validate(&data_value);
    
    match validation_result {
        Ok(_) => Ok(()),
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .map(|error| format!("Validation error: {}", error))
                .collect();
            Err(error_messages)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let schema = r#"
            {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "age": {"type": "number"}
                },
                "required": ["name"]
            }
        "#;

        let valid_data = r#"{"name": "Alice", "age": 30}"#;
        assert!(validate_json(schema, valid_data).is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let schema = r#"
            {
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                },
                "required": ["name"]
            }
        "#;

        let invalid_data = r#"{"age": 30}"#;
        assert!(validate_json(schema, invalid_data).is_err());
    }
}use serde_json::{Value, json};
use std::collections::HashSet;
use std::error::Error;

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

    pub fn validate(&self, json_str: &str) -> Result<Value, Box<dyn Error>> {
        let parsed: Value = serde_json::from_str(json_str)?;
        
        if let Value::Object(map) = &parsed {
            for field in &self.required_fields {
                if !map.contains_key(field) {
                    return Err(format!("Missing required field: {}", field).into());
                }
            }
        }
        
        self.validate_value_types(&parsed)?;
        Ok(parsed)
    }

    fn validate_value_types(&self, value: &Value) -> Result<(), Box<dyn Error>> {
        match value {
            Value::String(_) => Ok(()),
            Value::Number(_) => Ok(()),
            Value::Bool(_) => Ok(()),
            Value::Null => Ok(()),
            Value::Array(arr) => {
                for item in arr {
                    self.validate_value_types(item)?;
                }
                Ok(())
            }
            Value::Object(map) => {
                for (_, v) in map {
                    self.validate_value_types(v)?;
                }
                Ok(())
            }
        }
    }

    pub fn create_sample_schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"},
                "active": {"type": "boolean"}
            },
            "required": ["name"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("id");
        
        let json_data = r#"{"id": 123, "name": "test"}"#;
        let result = validator.validate(json_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("id");
        
        let json_data = r#"{"name": "test"}"#;
        let result = validator.validate(json_data);
        assert!(result.is_err());
    }
}