
use serde_json::{Result, Value};

pub fn validate_json(json_str: &str) -> Result<Value> {
    let parsed: Value = serde_json::from_str(json_str)?;
    Ok(parsed)
}

pub fn is_valid_json(json_str: &str) -> bool {
    validate_json(json_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let valid_json = r#"{"name": "Alice", "age": 30}"#;
        assert!(is_valid_json(valid_json));
        assert!(validate_json(valid_json).is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"{"name": "Bob", "age": }"#;
        assert!(!is_valid_json(invalid_json));
        assert!(validate_json(invalid_json).is_err());
    }

    #[test]
    fn test_nested_json() {
        let nested_json = r#"{"user": {"id": 1, "preferences": {"theme": "dark"}}}"#;
        let result = validate_json(nested_json);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed["user"]["id"], 1);
        assert_eq!(parsed["user"]["preferences"]["theme"], "dark");
    }
}