use serde_json::{Value, Result};
use std::fs;

pub fn parse_json_file(file_path: &str) -> Result<Value> {
    let content = fs::read_to_string(file_path)?;
    let json_value: Value = serde_json::from_str(&content)?;
    Ok(json_value)
}

pub fn validate_json_structure(value: &Value, expected_structure: &str) -> bool {
    match expected_structure {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}

pub fn pretty_print_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| String::from("Invalid JSON"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_json() {
        let test_json = json!({
            "name": "test",
            "value": 42,
            "active": true
        });
        
        let temp_file = "test_temp.json";
        fs::write(temp_file, test_json.to_string()).unwrap();
        
        let result = parse_json_file(temp_file);
        assert!(result.is_ok());
        
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_validate_structure() {
        let obj = json!({"key": "value"});
        assert!(validate_json_structure(&obj, "object"));
        
        let arr = json!([1, 2, 3]);
        assert!(validate_json_structure(&arr, "array"));
    }

    #[test]
    fn test_pretty_print() {
        let simple = json!({"compact": false});
        let printed = pretty_print_json(&simple);
        assert!(printed.contains("\n"));
        assert!(printed.contains("compact"));
    }
}