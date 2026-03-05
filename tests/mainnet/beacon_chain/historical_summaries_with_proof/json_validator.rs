use serde_json::Value;
use jsonschema::JSONSchema;

pub fn validate_json(schema: &str, data: &str) -> Result<(), Vec<String>> {
    let schema_value: Value = serde_json::from_str(schema)
        .map_err(|e| vec![format!("Invalid schema: {}", e)])?;
    
    let data_value: Value = serde_json::from_str(data)
        .map_err(|e| vec![format!("Invalid JSON data: {}", e)])?;
    
    let compiled_schema = JSONSchema::compile(&schema_value)
        .map_err(|e| vec![format!("Schema compilation failed: {}", e)])?;
    
    match compiled_schema.validate(&data_value) {
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
        
        let data = r#"{"name": "Alice", "age": 30}"#;
        
        assert!(validate_json(schema, data).is_ok());
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
        
        let data = r#"{"age": 30}"#;
        
        assert!(validate_json(schema, data).is_err());
    }
}use serde_json::{Value, from_str};
use std::error::Error;
use std::fs;

pub struct JsonValidator {
    schema: Value,
}

impl JsonValidator {
    pub fn new(schema_path: &str) -> Result<Self, Box<dyn Error>> {
        let schema_content = fs::read_to_string(schema_path)?;
        let schema: Value = from_str(&schema_content)?;
        Ok(JsonValidator { schema })
    }

    pub fn validate(&self, json_path: &str) -> Result<bool, Box<dyn Error>> {
        let json_content = fs::read_to_string(json_path)?;
        let data: Value = from_str(&json_content)?;
        
        self.validate_value(&data)
    }

    pub fn validate_value(&self, data: &Value) -> Result<bool, Box<dyn Error>> {
        if self.schema["type"] == "object" {
            self.validate_object(data)
        } else if self.schema["type"] == "array" {
            self.validate_array(data)
        } else {
            self.validate_primitive(data)
        }
    }

    fn validate_object(&self, data: &Value) -> Result<bool, Box<dyn Error>> {
        if !data.is_object() {
            return Ok(false);
        }

        if let Some(required) = self.schema.get("required") {
            if let Some(required_array) = required.as_array() {
                for field in required_array {
                    if let Some(field_str) = field.as_str() {
                        if !data.get(field_str).is_some() {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        if let Some(properties) = self.schema.get("properties") {
            if let Some(props_obj) = properties.as_object() {
                for (key, prop_schema) in props_obj {
                    if let Some(value) = data.get(key) {
                        let validator = JsonValidator {
                            schema: prop_schema.clone(),
                        };
                        if !validator.validate_value(value)? {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    fn validate_array(&self, data: &Value) -> Result<bool, Box<dyn Error>> {
        if !data.is_array() {
            return Ok(false);
        }

        if let Some(items) = self.schema.get("items") {
            let array = data.as_array().unwrap();
            for item in array {
                let validator = JsonValidator {
                    schema: items.clone(),
                };
                if !validator.validate_value(item)? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn validate_primitive(&self, data: &Value) -> Result<bool, Box<dyn Error>> {
        let schema_type = self.schema["type"].as_str().unwrap_or("string");
        
        match schema_type {
            "string" => Ok(data.is_string()),
            "number" => Ok(data.is_number()),
            "integer" => Ok(data.is_i64() || data.is_u64()),
            "boolean" => Ok(data.is_boolean()),
            "null" => Ok(data.is_null()),
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_object_validation() {
        let schema = json!({
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            }
        });

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), schema.to_string()).unwrap();

        let validator = JsonValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let valid_data = json!({"name": "John", "age": 30});
        assert!(validator.validate_value(&valid_data).unwrap());

        let invalid_data = json!({"name": "John"});
        assert!(!validator.validate_value(&invalid_data).unwrap());
    }

    #[test]
    fn test_array_validation() {
        let schema = json!({
            "type": "array",
            "items": {"type": "string"}
        });

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), schema.to_string()).unwrap();

        let validator = JsonValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let valid_data = json!(["a", "b", "c"]);
        assert!(validator.validate_value(&valid_data).unwrap());

        let invalid_data = json!([1, 2, 3]);
        assert!(!validator.validate_value(&invalid_data).unwrap());
    }
}