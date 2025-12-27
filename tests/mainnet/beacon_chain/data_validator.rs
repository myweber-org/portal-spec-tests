
use serde_json::Value;
use std::collections::HashSet;

pub struct DataValidator {
    required_fields: HashSet<String>,
    field_types: Vec<(String, FieldType)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Integer,
    Nullable(Box<FieldType>),
}

impl DataValidator {
    pub fn new() -> Self {
        DataValidator {
            required_fields: HashSet::new(),
            field_types: Vec::new(),
        }
    }

    pub fn require_field(mut self, field: &str) -> Self {
        self.required_fields.insert(field.to_string());
        self
    }

    pub fn expect_type(mut self, field: &str, field_type: FieldType) -> Self {
        self.field_types.push((field.to_string(), field_type));
        self
    }

    pub fn validate(&self, data: &Value) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for field in &self.required_fields {
            if !data.get(field).is_some() {
                errors.push(format!("Missing required field: {}", field));
            }
        }

        for (field, expected_type) in &self.field_types {
            if let Some(value) = data.get(field) {
                if !self.check_type(value, expected_type) {
                    errors.push(format!(
                        "Field '{}' has wrong type. Expected: {:?}, Got: {:?}",
                        field,
                        expected_type,
                        self.get_value_type(value)
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn check_type(&self, value: &Value, field_type: &FieldType) -> bool {
        match field_type {
            FieldType::String => value.is_string(),
            FieldType::Number => value.is_number(),
            FieldType::Boolean => value.is_boolean(),
            FieldType::Array => value.is_array(),
            FieldType::Object => value.is_object(),
            FieldType::Integer => value.is_number() && value.as_f64().map(|n| n.fract() == 0.0).unwrap_or(false),
            FieldType::Nullable(inner) => value.is_null() || self.check_type(value, inner),
        }
    }

    fn get_value_type(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Object(_) => "object".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validation() {
        let validator = DataValidator::new()
            .require_field("name")
            .require_field("age")
            .expect_type("name", FieldType::String)
            .expect_type("age", FieldType::Integer)
            .expect_type("active", FieldType::Nullable(Box::new(FieldType::Boolean)));

        let valid_data = json!({
            "name": "John",
            "age": 30,
            "active": true
        });

        let invalid_data = json!({
            "name": "Jane",
            "age": 25.5
        });

        assert!(validator.validate(&valid_data).is_ok());
        assert!(validator.validate(&invalid_data).is_err());
    }
}