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
}use serde_json;

pub fn is_valid_json(input: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let valid_json = r#"{"name": "Alice", "age": 30}"#;
        assert!(is_valid_json(valid_json));
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"{"name": "Bob", age: 25}"#;
        assert!(!is_valid_json(invalid_json));
    }

    #[test]
    fn test_empty_string() {
        assert!(!is_valid_json(""));
    }

    #[test]
    fn test_valid_array_json() {
        let array_json = r#"[1, 2, 3, "four"]"#;
        assert!(is_valid_json(array_json));
    }
}use serde_json::Value;
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
}use serde_json::{Result, Value};
use std::collections::HashSet;

pub fn is_valid_json(json_str: &str) -> bool {
    serde_json::from_str::<Value>(json_str).is_ok()
}

pub fn validate_json_structure(json_str: &str, required_keys: &[&str]) -> Result<bool> {
    let parsed: Value = serde_json::from_str(json_str)?;
    
    if let Value::Object(map) = parsed {
        let keys: HashSet<String> = map.keys().cloned().collect();
        let required_set: HashSet<&str> = required_keys.iter().cloned().collect();
        
        return Ok(required_set.is_subset(&keys.iter().map(|s| s.as_str()).collect()));
    }
    
    Ok(false)
}

pub fn extract_string_field(json_str: &str, field: &str) -> Result<Option<String>> {
    let parsed: Value = serde_json::from_str(json_str)?;
    
    if let Value::Object(map) = parsed {
        if let Some(value) = map.get(field) {
            if let Value::String(s) = value {
                return Ok(Some(s.clone()));
            }
        }
    }
    
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(is_valid_json(valid_json));
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"{"name": test}"#;
        assert!(!is_valid_json(invalid_json));
    }

    #[test]
    fn test_validate_structure() {
        let json = r#"{"id": 1, "name": "item"}"#;
        let required = vec!["id", "name"];
        assert!(validate_json_structure(json, &required).unwrap());
    }

    #[test]
    fn test_extract_field() {
        let json = r#"{"title": "example", "count": 5}"#;
        let result = extract_string_field(json, "title").unwrap();
        assert_eq!(result, Some("example".to_string()));
    }
}