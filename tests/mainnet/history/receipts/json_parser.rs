
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub settings: HashMap<String, Value>,
    pub enabled: bool,
}

pub fn parse_json_file(file_path: &str) -> Result<Config> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| serde_json::Error::io(e))?;
    
    let config: Config = serde_json::from_str(&content)?;
    
    if config.name.is_empty() {
        return Err(serde_json::Error::custom("Name field cannot be empty"));
    }
    
    if config.version.is_empty() {
        return Err(serde_json::Error::custom("Version field cannot be empty"));
    }
    
    Ok(config)
}

pub fn validate_json_structure(json_str: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(json_str)?;
    
    if !value.is_object() {
        return Err(serde_json::Error::custom("JSON must be an object"));
    }
    
    Ok(value)
}

pub fn merge_json_objects(a: &Value, b: &Value) -> Result<Value> {
    if !a.is_object() || !b.is_object() {
        return Err(serde_json::Error::custom("Both values must be JSON objects"));
    }
    
    let mut merged = a.as_object().unwrap().clone();
    let b_obj = b.as_object().unwrap();
    
    for (key, value) in b_obj {
        merged.insert(key.clone(), value.clone());
    }
    
    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_json_structure() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(validate_json_structure(valid_json).is_ok());
        
        let invalid_json = r#"["array", "not", "object"]"#;
        assert!(validate_json_structure(invalid_json).is_err());
    }
    
    #[test]
    fn test_merge_json_objects() {
        let json_a = serde_json::json!({"a": 1, "b": 2});
        let json_b = serde_json::json!({"b": 3, "c": 4});
        
        let result = merge_json_objects(&json_a, &json_b).unwrap();
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 3);
        assert_eq!(result["c"], 4);
    }
}