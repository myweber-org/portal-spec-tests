use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ValidationError {
    MissingField(String),
    TypeMismatch(String, String),
    InvalidValue(String),
    SchemaError(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ValidationError::TypeMismatch(field, expected) => {
                write!(f, "Field '{}' must be of type {}", field, expected)
            }
            ValidationError::InvalidValue(field) => write!(f, "Invalid value for field: {}", field),
            ValidationError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
        }
    }
}

impl Error for ValidationError {}

pub struct JsonValidator {
    schema: Map<String, Value>,
    required_fields: HashSet<String>,
}

impl JsonValidator {
    pub fn new(schema: Value) -> Result<Self, ValidationError> {
        let schema_map = match schema {
            Value::Object(map) => map,
            _ => return Err(ValidationError::SchemaError("Schema must be a JSON object".to_string())),
        };

        let mut required_fields = HashSet::new();
        if let Some(Value::Array(fields)) = schema_map.get("required") {
            for field in fields {
                if let Value::String(field_str) = field {
                    required_fields.insert(field_str.clone());
                }
            }
        }

        Ok(JsonValidator {
            schema: schema_map,
            required_fields,
        })
    }

    pub fn validate(&self, data: &Value) -> Result<(), ValidationError> {
        let data_map = match data {
            Value::Object(map) => map,
            _ => return Err(ValidationError::TypeMismatch("root".to_string(), "object".to_string())),
        };

        for required_field in &self.required_fields {
            if !data_map.contains_key(required_field) {
                return Err(ValidationError::MissingField(required_field.clone()));
            }
        }

        if let Some(Value::Object(properties)) = self.schema.get("properties") {
            for (field, value) in data_map {
                if let Some(field_schema) = properties.get(field) {
                    self.validate_field(field, value, field_schema)?;
                }
            }
        }

        Ok(())
    }

    fn validate_field(&self, field_name: &str, value: &Value, schema: &Value) -> Result<(), ValidationError> {
        if let Value::Object(schema_obj) = schema {
            if let Some(Value::String(type_str)) = schema_obj.get("type") {
                match type_str.as_str() {
                    "string" => {
                        if !value.is_string() {
                            return Err(ValidationError::TypeMismatch(
                                field_name.to_string(),
                                "string".to_string(),
                            ));
                        }
                    }
                    "number" => {
                        if !value.is_number() {
                            return Err(ValidationError::TypeMismatch(
                                field_name.to_string(),
                                "number".to_string(),
                            ));
                        }
                    }
                    "boolean" => {
                        if !value.is_boolean() {
                            return Err(ValidationError::TypeMismatch(
                                field_name.to_string(),
                                "boolean".to_string(),
                            ));
                        }
                    }
                    "array" => {
                        if !value.is_array() {
                            return Err(ValidationError::TypeMismatch(
                                field_name.to_string(),
                                "array".to_string(),
                            ));
                        }
                    }
                    "object" => {
                        if !value.is_object() {
                            return Err(ValidationError::TypeMismatch(
                                field_name.to_string(),
                                "object".to_string(),
                            ));
                        }
                    }
                    _ => return Err(ValidationError::SchemaError(format!("Unknown type: {}", type_str))),
                }
            }

            if let Some(Value::Array(enum_values)) = schema_obj.get("enum") {
                if !enum_values.contains(value) {
                    return Err(ValidationError::InvalidValue(field_name.to_string()));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_json() {
        let schema = json!({
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"},
                "active": {"type": "boolean"}
            }
        });

        let validator = JsonValidator::new(schema).unwrap();
        let data = json!({
            "name": "John",
            "age": 30,
            "active": true
        });

        assert!(validator.validate(&data).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let schema = json!({
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            }
        });

        let validator = JsonValidator::new(schema).unwrap();
        let data = json!({
            "name": "John"
        });

        assert!(matches!(
            validator.validate(&data),
            Err(ValidationError::MissingField(_))
        ));
    }

    #[test]
    fn test_type_mismatch() {
        let schema = json!({
            "type": "object",
            "properties": {
                "age": {"type": "number"}
            }
        });

        let validator = JsonValidator::new(schema).unwrap();
        let data = json!({
            "age": "thirty"
        });

        assert!(matches!(
            validator.validate(&data),
            Err(ValidationError::TypeMismatch(_, _))
        ));
    }
}