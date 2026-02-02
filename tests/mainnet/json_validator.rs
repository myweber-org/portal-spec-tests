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

        let valid_data = r#"{"name": "John", "age": 30}"#;
        
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
}use serde_json::{Result, Value};
use std::fs;

pub fn validate_json_file(file_path: &str) -> Result<Value> {
    let content = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", file_path));
    
    let parsed: Value = serde_json::from_str(&content)?;
    
    Ok(parsed)
}

pub fn validate_json_string(json_str: &str) -> Result<Value> {
    let parsed: Value = serde_json::from_str(json_str)?;
    
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_json_string() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        let result = validate_json_string(valid_json);
        assert!(result.is_ok());
        
        let parsed = result.unwrap();
        assert_eq!(parsed["name"], "test");
        assert_eq!(parsed["value"], 42);
    }

    #[test]
    fn test_invalid_json_string() {
        let invalid_json = r#"{"name": "test", "value": }"#;
        let result = validate_json_string(invalid_json);
        assert!(result.is_err());
    }
}use serde_json::{Value, from_str};
use std::fs;

pub struct JsonValidator {
    schema: Value,
}

impl JsonValidator {
    pub fn new(schema_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let schema_content = fs::read_to_string(schema_path)?;
        let schema: Value = from_str(&schema_content)?;
        Ok(JsonValidator { schema })
    }

    pub fn validate(&self, json_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let data_content = fs::read_to_string(json_path)?;
        let data: Value = from_str(&data_content)?;
        
        // Simplified validation logic
        let is_valid = self.validate_structure(&data);
        Ok(is_valid)
    }

    fn validate_structure(&self, data: &Value) -> bool {
        match (self.schema.get("type"), data) {
            (Some(Value::String(schema_type)), data_value) => {
                match schema_type.as_str() {
                    "object" => data_value.is_object(),
                    "array" => data_value.is_array(),
                    "string" => data_value.is_string(),
                    "number" => data_value.is_number(),
                    "boolean" => data_value.is_boolean(),
                    "null" => data_value.is_null(),
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_object_validation() {
        let schema_content = r#"{"type": "object"}"#;
        let data_content = r#"{"name": "test", "value": 42}"#;
        
        let mut schema_file = NamedTempFile::new().unwrap();
        schema_file.write_all(schema_content.as_bytes()).unwrap();
        
        let mut data_file = NamedTempFile::new().unwrap();
        data_file.write_all(data_content.as_bytes()).unwrap();
        
        let validator = JsonValidator::new(schema_file.path().to_str().unwrap()).unwrap();
        let result = validator.validate(data_file.path().to_str().unwrap()).unwrap();
        
        assert!(result);
    }

    #[test]
    fn test_invalid_type_validation() {
        let schema_content = r#"{"type": "string"}"#;
        let data_content = r#"{"not": "a_string"}"#;
        
        let mut schema_file = NamedTempFile::new().unwrap();
        schema_file.write_all(schema_content.as_bytes()).unwrap();
        
        let mut data_file = NamedTempFile::new().unwrap();
        data_file.write_all(data_content.as_bytes()).unwrap();
        
        let validator = JsonValidator::new(schema_file.path().to_str().unwrap()).unwrap();
        let result = validator.validate(data_file.path().to_str().unwrap()).unwrap();
        
        assert!(!result);
    }
}