use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &Value, instance: &Value) -> Result<(), Vec<String>> {
    let compiled = JSONSchema::compile(schema)
        .map_err(|e| vec![format!("Schema compilation failed: {}", e)])?;

    let validation_result = compiled.validate(instance);
    if validation_result.is_ok() {
        Ok(())
    } else {
        let errors: Vec<String> = validation_result
            .unwrap_err()
            .map(|error| format!("Validation error: {}", error))
            .collect();
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_json_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer", "minimum": 0}
            },
            "required": ["name"]
        });

        let instance = json!({
            "name": "Alice",
            "age": 30
        });

        assert!(validate_json(&schema, &instance).is_ok());
    }

    #[test]
    fn test_invalid_json_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "email": {"type": "string", "format": "email"}
            },
            "required": ["email"]
        });

        let instance = json!({
            "email": "not-an-email"
        });

        let result = validate_json(&schema, &instance);
        assert!(result.is_err());
        assert!(!result.unwrap_err().is_empty());
    }
}