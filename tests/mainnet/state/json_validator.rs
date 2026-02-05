use serde_json::{Value, json};
use std::collections::HashSet;
use std::error::Error;

pub fn validate_json_schema(data: &str, schema: &Value) -> Result<bool, Box<dyn Error>> {
    let parsed_data: Value = serde_json::from_str(data)?;
    validate_value(&parsed_data, schema)
}

fn validate_value(data: &Value, schema: &Value) -> Result<bool, Box<dyn Error>> {
    match schema.get("type").and_then(|t| t.as_str()) {
        Some("object") => validate_object(data, schema),
        Some("array") => validate_array(data, schema),
        Some("string") => validate_string(data, schema),
        Some("number") => validate_number(data, schema),
        Some("boolean") => validate_boolean(data, schema),
        Some("null") => validate_null(data),
        _ => Ok(true),
    }
}

fn validate_object(data: &Value, schema: &Value) -> Result<bool, Box<dyn Error>> {
    if !data.is_object() {
        return Ok(false);
    }

    let obj = data.as_object().unwrap();
    let required_fields: HashSet<String> = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    for field in &required_fields {
        if !obj.contains_key(field) {
            return Ok(false);
        }
    }

    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        for (key, prop_schema) in properties {
            if let Some(value) = obj.get(key) {
                if !validate_value(value, prop_schema)? {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

fn validate_array(data: &Value, schema: &Value) -> Result<bool, Box<dyn Error>> {
    if !data.is_array() {
        return Ok(false);
    }

    let arr = data.as_array().unwrap();
    if let Some(min_items) = schema.get("minItems").and_then(|m| m.as_u64()) {
        if arr.len() < min_items as usize {
            return Ok(false);
        }
    }

    if let Some(max_items) = schema.get("maxItems").and_then(|m| m.as_u64()) {
        if arr.len() > max_items as usize {
            return Ok(false);
        }
    }

    if let Some(item_schema) = schema.get("items") {
        for item in arr {
            if !validate_value(item, item_schema)? {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

fn validate_string(data: &Value, schema: &Value) -> Result<bool, Box<dyn Error>> {
    if !data.is_string() {
        return Ok(false);
    }

    let s = data.as_str().unwrap();
    if let Some(min_len) = schema.get("minLength").and_then(|m| m.as_u64()) {
        if s.len() < min_len as usize {
            return Ok(false);
        }
    }

    if let Some(max_len) = schema.get("maxLength").and_then(|m| m.as_u64()) {
        if s.len() > max_len as usize {
            return Ok(false);
        }
    }

    if let Some(pattern) = schema.get("pattern").and_then(|p| p.as_str()) {
        let re = regex::Regex::new(pattern)?;
        if !re.is_match(s) {
            return Ok(false);
        }
    }

    Ok(true)
}

fn validate_number(data: &Value, schema: &Value) -> Result<bool, Box<dyn Error>> {
    if !data.is_number() {
        return Ok(false);
    }

    let num = data.as_f64().unwrap();
    if let Some(minimum) = schema.get("minimum").and_then(|m| m.as_f64()) {
        if num < minimum {
            return Ok(false);
        }
    }

    if let Some(maximum) = schema.get("maximum").and_then(|m| m.as_f64()) {
        if num > maximum {
            return Ok(false);
        }
    }

    Ok(true)
}

fn validate_boolean(data: &Value, _schema: &Value) -> Result<bool, Box<dyn Error>> {
    Ok(data.is_boolean())
}

fn validate_null(data: &Value) -> Result<bool, Box<dyn Error>> {
    Ok(data.is_null())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_validation() {
        let schema = json!({
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string", "minLength": 1},
                "age": {"type": "number", "minimum": 0}
            }
        });

        let valid_data = r#"{"name": "Alice", "age": 30}"#;
        let invalid_data = r#"{"name": "", "age": -5}"#;

        assert!(validate_json_schema(valid_data, &schema).unwrap());
        assert!(!validate_json_schema(invalid_data, &schema).unwrap());
    }

    #[test]
    fn test_array_validation() {
        let schema = json!({
            "type": "array",
            "minItems": 1,
            "maxItems": 3,
            "items": {"type": "string"}
        });

        let valid_data = r#"["a", "b"]"#;
        let invalid_data = r#"[]"#;

        assert!(validate_json_schema(valid_data, &schema).unwrap());
        assert!(!validate_json_schema(invalid_data, &schema).unwrap());
    }
}use serde_json::{Value, Map};
use std::collections::HashSet;

pub struct JsonValidator {
    required_fields: HashSet<String>,
    allowed_types: Map<String, String>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            allowed_types: Map::new(),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.insert(field.to_string());
    }

    pub fn define_field_type(&mut self, field: &str, type_name: &str) {
        self.allowed_types.insert(field.to_string(), type_name.to_string());
    }

    pub fn validate(&self, json_data: &Value) -> Result<(), String> {
        if !json_data.is_object() {
            return Err("Input must be a JSON object".to_string());
        }

        let obj = json_data.as_object().unwrap();

        for field in &self.required_fields {
            if !obj.contains_key(field) {
                return Err(format!("Missing required field: {}", field));
            }
        }

        for (field, value) in obj {
            if let Some(expected_type) = self.allowed_types.get(field) {
                if !self.check_type(value, expected_type) {
                    return Err(format!("Field '{}' must be of type {}", field, expected_type));
                }
            }
        }

        Ok(())
    }

    fn check_type(&self, value: &Value, expected_type: &str) -> bool {
        match expected_type {
            "string" => value.is_string(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            "null" => value.is_null(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_validation() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("name");
        validator.define_field_type("name", "string");
        validator.define_field_type("age", "number");

        let valid_data = json!({
            "name": "John",
            "age": 30
        });

        let invalid_data = json!({
            "name": 123,
            "age": "thirty"
        });

        assert!(validator.validate(&valid_data).is_ok());
        assert!(validator.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_missing_required_field() {
        let mut validator = JsonValidator::new();
        validator.add_required_field("id");

        let data = json!({
            "name": "test"
        });

        let result = validator.validate(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field"));
    }
}