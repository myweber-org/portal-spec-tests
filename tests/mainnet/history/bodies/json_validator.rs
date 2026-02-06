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
                .map(|e| format!("Validation error: {}", e))
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
}use serde_json::Value;
use std::collections::HashSet;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: Vec<&'static str>,
}

impl JsonValidator {
    pub fn new(required_fields: Vec<&str>) -> Self {
        JsonValidator {
            required_fields: required_fields.into_iter().map(String::from).collect(),
            allowed_types: vec!["object", "array", "string", "number", "boolean", "null"],
        }
    }

    pub fn validate(&self, json_str: &str) -> Result<(), String> {
        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON format: {}", e))?;

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
                Ok(())
            }
            Value::Array(arr) => {
                for item in arr {
                    self.validate_structure(item)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
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

    pub fn validate_type(&self, value: &Value) -> bool {
        let type_str = match value {
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Null => "null",
        };
        self.allowed_types.contains(&type_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let validator = JsonValidator::new(vec!["name", "age"]);
        let json = r#"{"name": "John", "age": 30, "city": "New York"}"#;
        assert!(validator.validate(json).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let validator = JsonValidator::new(vec!["name", "age"]);
        let json = r#"{"name": "John"}"#;
        assert!(validator.validate(json).is_err());
    }

    #[test]
    fn test_invalid_json() {
        let validator = JsonValidator::new(vec![]);
        let json = r#"{"name": "John", "age": }"#;
        assert!(validator.validate(json).is_err());
    }
}