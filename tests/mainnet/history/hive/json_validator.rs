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
}
use serde_json::{Value, from_str};
use std::fs;

pub fn validate_json_schema(json_str: &str, schema_path: &str) -> Result<(), String> {
    let json_data: Value = from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let schema_content = fs::read_to_string(schema_path)
        .map_err(|e| format!("Failed to read schema file: {}", e))?;

    let schema: Value = from_str(&schema_content)
        .map_err(|e| format!("Invalid schema JSON: {}", e))?;

    if json_data.is_object() && schema.is_object() {
        validate_object(&json_data, &schema)?;
        Ok(())
    } else {
        Err("Both JSON data and schema must be objects".to_string())
    }
}

fn validate_object(data: &Value, schema: &Value) -> Result<(), String> {
    if let Some(required_fields) = schema.get("required").and_then(|v| v.as_array()) {
        for field in required_fields {
            if let Some(field_name) = field.as_str() {
                if !data.get(field_name).is_some() {
                    return Err(format!("Missing required field: {}", field_name));
                }
            }
        }
    }

    if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
        for (key, prop_schema) in properties {
            if let Some(value) = data.get(key) {
                validate_type(value, prop_schema)?;
            }
        }
    }

    Ok(())
}

fn validate_type(value: &Value, schema: &Value) -> Result<(), String> {
    if let Some(schema_type) = schema.get("type").and_then(|v| v.as_str()) {
        match schema_type {
            "string" => {
                if !value.is_string() {
                    return Err(format!("Expected string, got {:?}", value));
                }
            }
            "number" => {
                if !value.is_number() {
                    return Err(format!("Expected number, got {:?}", value));
                }
            }
            "boolean" => {
                if !value.is_boolean() {
                    return Err(format!("Expected boolean, got {:?}", value));
                }
            }
            "object" => {
                if !value.is_object() {
                    return Err(format!("Expected object, got {:?}", value));
                }
                if let Some(properties) = schema.get("properties") {
                    validate_object(value, properties)?;
                }
            }
            "array" => {
                if !value.is_array() {
                    return Err(format!("Expected array, got {:?}", value));
                }
            }
            _ => return Err(format!("Unsupported type: {}", schema_type)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_validation() {
        let json_data = r#"
        {
            "name": "John Doe",
            "age": 30,
            "active": true
        }
        "#;

        let schema = r#"
        {
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"},
                "active": {"type": "boolean"}
            }
        }
        "#;

        fs::write("test_schema.json", schema).unwrap();
        let result = validate_json_schema(json_data, "test_schema.json");
        fs::remove_file("test_schema.json").unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let json_data = r#"
        {
            "name": "John Doe"
        }
        "#;

        let schema = r#"
        {
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            }
        }
        "#;

        fs::write("test_schema.json", schema).unwrap();
        let result = validate_json_schema(json_data, "test_schema.json");
        fs::remove_file("test_schema.json").unwrap();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field"));
    }
}