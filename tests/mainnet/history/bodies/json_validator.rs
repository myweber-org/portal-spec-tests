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
}use serde_json::{Result, Value};
use std::fs;

pub fn validate_json_from_str(json_str: &str) -> Result<Value> {
    let parsed: Value = serde_json::from_str(json_str)?;
    Ok(parsed)
}

pub fn validate_json_from_file(file_path: &str) -> Result<Value> {
    let file_content = fs::read_to_string(file_path)?;
    validate_json_from_str(&file_content)
}

pub fn is_valid_json(json_str: &str) -> bool {
    validate_json_from_str(json_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_string() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(is_valid_json(valid_json));
    }

    #[test]
    fn test_invalid_json_string() {
        let invalid_json = r#"{"name": test, "value": 42}"#;
        assert!(!is_valid_json(invalid_json));
    }

    #[test]
    fn test_parse_valid_json() {
        let valid_json = r#"{"key": "value"}"#;
        let result = validate_json_from_str(valid_json);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed["key"], "value");
    }
}