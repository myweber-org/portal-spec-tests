use serde_json::{Value, Error};
use std::collections::HashMap;
use std::fs;

pub struct JsonParser {
    data: HashMap<String, Value>,
}

impl JsonParser {
    pub fn new() -> Self {
        JsonParser {
            data: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Error> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::io(e))?;
        
        let parsed: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(map) = parsed {
            for (key, value) in map {
                self.data.insert(key, value);
            }
            Ok(())
        } else {
            Err(Error::custom("Root must be a JSON object"))
        }
    }

    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn validate_schema(&self, schema: &HashMap<String, &str>) -> Vec<String> {
        let mut errors = Vec::new();
        
        for (key, expected_type) in schema {
            match self.data.get(key) {
                Some(value) => {
                    let actual_type = match value {
                        Value::Null => "null",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Array(_) => "array",
                        Value::Object(_) => "object",
                    };
                    
                    if actual_type != *expected_type {
                        errors.push(format!(
                            "Field '{}': expected type '{}', found '{}'",
                            key, expected_type, actual_type
                        ));
                    }
                }
                None => {
                    errors.push(format!("Missing required field: '{}'", key));
                }
            }
        }
        
        errors
    }

    pub fn extract_strings(&self) -> Vec<String> {
        let mut strings = Vec::new();
        
        for value in self.data.values() {
            Self::collect_strings(value, &mut strings);
        }
        
        strings
    }

    fn collect_strings(value: &Value, strings: &mut Vec<String>) {
        match value {
            Value::String(s) => strings.push(s.clone()),
            Value::Array(arr) => {
                for item in arr {
                    Self::collect_strings(item, strings);
                }
            }
            Value::Object(obj) => {
                for val in obj.values() {
                    Self::collect_strings(val, strings);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parser_operations() {
        let mut parser = JsonParser::new();
        
        let test_json = json!({
            "name": "test",
            "count": 42,
            "tags": ["rust", "json"],
            "metadata": {
                "version": "1.0"
            }
        });
        
        let temp_file = "test_temp.json";
        fs::write(temp_file, test_json.to_string()).unwrap();
        
        let result = parser.load_from_file(temp_file);
        assert!(result.is_ok());
        
        let schema = HashMap::from([
            ("name".to_string(), "string"),
            ("count".to_string(), "number"),
        ]);
        
        let errors = parser.validate_schema(&schema);
        assert!(errors.is_empty());
        
        let strings = parser.extract_strings();
        assert!(strings.contains(&"test".to_string()));
        assert!(strings.contains(&"rust".to_string()));
        
        fs::remove_file(temp_file).unwrap();
    }
}