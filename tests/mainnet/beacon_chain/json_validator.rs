use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &Value, data: &Value) -> Result<(), String> {
    let compiled = JSONSchema::compile(schema)
        .map_err(|e| format!("Schema compilation failed: {}", e))?;
    
    match compiled.validate(data) {
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
    use serde_json::json;

    #[test]
    fn test_valid_json() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number", "minimum": 0}
            },
            "required": ["name"]
        });

        let data = json!({"name": "Alice", "age": 30});
        assert!(validate_json(&schema, &data).is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let schema = json!({
            "type": "object",
            "properties": {
                "email": {"type": "string", "format": "email"}
            },
            "required": ["email"]
        });

        let data = json!({"email": "not-an-email"});
        assert!(validate_json(&schema, &data).is_err());
    }
}