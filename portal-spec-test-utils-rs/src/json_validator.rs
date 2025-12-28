use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &str, data: &str) -> Result<(), String> {
    let schema_value: Value = serde_json::from_str(schema)
        .map_err(|e| format!("Invalid schema: {}", e))?;
    
    let data_value: Value = serde_json::from_str(data)
        .map_err(|e| format!("Invalid JSON data: {}", e))?;
    
    let compiled_schema = JSONSchema::compile(&schema_value)
        .map_err(|e| format!("Schema compilation failed: {}", e))?;
    
    match compiled_schema.validate(&data_value) {
        Ok(_) => Ok(()),
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .map(|e| format!("Validation error: {}", e))
                .collect();
            Err(error_messages.join(", "))
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
        
        let data = r#"{"name": "Alice", "age": 30}"#;
        
        assert!(validate_json(schema, data).is_ok());
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
        
        let data = r#"{"age": 30}"#;
        
        assert!(validate_json(schema, data).is_err());
    }
}use serde_json::{Value, json};
use std::collections::HashSet;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: Vec<&'static str>,
}

impl JsonValidator {
    pub fn new(required_fields: Vec<&str>) -> Self {
        JsonValidator {
            required_fields: required_fields.into_iter().map(String::from).collect(),
            allowed_types: vec!["string", "number", "boolean", "object", "array", "null"],
        }
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, String> {
        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON format: {}", e))?;

        if let Value::Object(ref obj) = parsed {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    return Err(format!("Missing required field: {}", field));
                }
            }
        }

        self.validate_value_types(&parsed)?;
        Ok(parsed)
    }

    fn validate_value_types(&self, value: &Value) -> Result<(), String> {
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
            Value::Object(obj) => {
                for (_, v) in obj {
                    self.validate_value_types(v)?;
                }
                Ok(())
            }
        }
    }

    pub fn create_example(&self) -> Value {
        let mut example = json!({});
        if let Value::Object(ref mut obj) = example {
            for field in &self.required_fields {
                obj.insert(field.clone(), json!("example_value"));
            }
        }
        example
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let validator = JsonValidator::new(vec!["name", "age"]);
        let json_data = r#"{"name": "John", "age": 30, "active": true}"#;
        assert!(validator.validate(json_data).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let validator = JsonValidator::new(vec!["name", "email"]);
        let json_data = r#"{"name": "John"}"#;
        assert!(validator.validate(json_data).is_err());
    }

    #[test]
    fn test_invalid_json_format() {
        let validator = JsonValidator::new(vec![]);
        let json_data = r#"{"name": "John",}"#;
        assert!(validator.validate(json_data).is_err());
    }
}