use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDocument {
    data: Value,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidJson(String),
    MissingField(String),
    TypeMismatch(String),
    Custom(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            ParseError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ParseError::TypeMismatch(expected) => write!(f, "Type mismatch, expected: {}", expected),
            ParseError::Custom(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for ParseError {}

impl JsonDocument {
    pub fn parse(json_str: &str) -> std::result::Result<Self, ParseError> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| ParseError::InvalidJson(e.to_string()))?;
        
        let mut metadata = HashMap::new();
        metadata.insert("parsed_at".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("source_length".to_string(), json_str.len().to_string());
        
        Ok(JsonDocument {
            data: value,
            metadata,
        })
    }
    
    pub fn get_string(&self, path: &str) -> std::result::Result<String, ParseError> {
        let keys: Vec<&str> = path.split('.').collect();
        let mut current = &self.data;
        
        for key in keys {
            current = current.get(key)
                .ok_or_else(|| ParseError::MissingField(key.to_string()))?;
        }
        
        current.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ParseError::TypeMismatch("string".to_string()))
    }
    
    pub fn get_number(&self, path: &str) -> std::result::Result<f64, ParseError> {
        let keys: Vec<&str> = path.split('.').collect();
        let mut current = &self.data;
        
        for key in keys {
            current = current.get(key)
                .ok_or_else(|| ParseError::MissingField(key.to_string()))?;
        }
        
        current.as_f64()
            .ok_or_else(|| ParseError::TypeMismatch("number".to_string()))
    }
    
    pub fn validate_schema(&self, required_fields: &[&str]) -> std::result::Result<(), ParseError> {
        for field in required_fields {
            let keys: Vec<&str> = field.split('.').collect();
            let mut current = &self.data;
            
            for key in keys {
                current = match current.get(key) {
                    Some(val) => val,
                    None => return Err(ParseError::MissingField(field.to_string())),
                };
            }
        }
        Ok(())
    }
    
    pub fn to_pretty_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.data)
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_json_parsing() {
        let json_data = r#"
        {
            "user": {
                "name": "John Doe",
                "age": 30,
                "email": "john@example.com"
            }
        }"#;
        
        let doc = JsonDocument::parse(json_data).unwrap();
        assert_eq!(doc.get_string("user.name").unwrap(), "John Doe");
        assert_eq!(doc.get_number("user.age").unwrap(), 30.0);
    }
    
    #[test]
    fn test_validation() {
        let json_data = r#"{"user": {"name": "Alice"}}"#;
        let doc = JsonDocument::parse(json_data).unwrap();
        
        let required = vec!["user.name", "user.email"];
        let result = doc.validate_schema(&required);
        assert!(result.is_err());
    }
}