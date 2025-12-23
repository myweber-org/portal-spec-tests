use serde_json::{Value, json};
use std::fs;

pub fn parse_json_file(file_path: &str) -> Result<Value, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let json_value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    Ok(json_value)
}

pub fn validate_json_structure(value: &Value, required_fields: &[&str]) -> bool {
    if let Value::Object(map) = value {
        required_fields.iter().all(|field| map.contains_key(*field))
    } else {
        false
    }
}

pub fn pretty_print_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| String::from("Invalid JSON"))
}

pub fn create_sample_json() -> Value {
    json!({
        "name": "Data Processor",
        "version": "1.0.0",
        "active": true,
        "features": ["parsing", "validation", "formatting"],
        "metadata": {
            "author": "System",
            "timestamp": 1234567890
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_json() {
        let json_data = r#"{"test": "value", "number": 42}"#;
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), json_data).unwrap();
        
        let result = parse_json_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_structure() {
        let json_value = json!({
            "name": "test",
            "id": 123
        });
        
        assert!(validate_json_structure(&json_value, &["name", "id"]));
        assert!(!validate_json_structure(&json_value, &["name", "missing"]));
    }

    #[test]
    fn test_pretty_print() {
        let json_value = json!({"compact": true});
        let printed = pretty_print_json(&json_value);
        assert!(printed.contains("compact"));
    }
}