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
}