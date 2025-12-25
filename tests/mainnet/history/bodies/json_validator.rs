use serde_json::{Value, Error as JsonError};
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidJson(String),
    MissingField(String),
    TypeMismatch(String, String),
    ConstraintViolation(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            ValidationError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ValidationError::TypeMismatch(field, expected) => {
                write!(f, "Field '{}' must be of type {}", field, expected)
            }
            ValidationError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
        }
    }
}

pub struct JsonValidator {
    required_fields: HashSet<String>,
    field_types: Vec<(String, String)>,
    custom_validators: Vec<Box<dyn Fn(&Value) -> Result<(), ValidationError>>>,
}

impl JsonValidator {
    pub fn new() -> Self {
        JsonValidator {
            required_fields: HashSet::new(),
            field_types: Vec::new(),
            custom_validators: Vec::new(),
        }
    }

    pub fn require_field(mut self, field: &str) -> Self {
        self.required_fields.insert(field.to_string());
        self
    }

    pub fn expect_type(mut self, field: &str, type_name: &str) -> Self {
        self.field_types.push((field.to_string(), type_name.to_string()));
        self
    }

    pub fn add_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&Value) -> Result<(), ValidationError> + 'static,
    {
        self.custom_validators.push(Box::new(validator));
        self
    }

    pub fn validate(&self, json_str: &str) -> Result<Value, ValidationError> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e: JsonError| ValidationError::InvalidJson(e.to_string()))?;

        for field in &self.required_fields {
            if !value.get(field).is_some() {
                return Err(ValidationError::MissingField(field.clone()));
            }
        }

        for (field, expected_type) in &self.field_types {
            if let Some(field_value) = value.get(field) {
                let actual_type = match field_value {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                };

                if actual_type != expected_type {
                    return Err(ValidationError::TypeMismatch(
                        field.clone(),
                        expected_type.clone(),
                    ));
                }
            }
        }

        for validator in &self.custom_validators {
            validator(&value)?;
        }

        Ok(value)
    }
}

pub fn validate_email_format(value: &Value) -> Result<(), ValidationError> {
    if let Value::String(email) = value {
        if email.contains('@') && email.contains('.') {
            Ok(())
        } else {
            Err(ValidationError::ConstraintViolation(
                "Email must contain '@' and '.'".to_string(),
            ))
        }
    } else {
        Err(ValidationError::TypeMismatch(
            "email".to_string(),
            "string".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let validator = JsonValidator::new()
            .require_field("name")
            .require_field("age")
            .expect_type("name", "string")
            .expect_type("age", "number");

        let valid_json = r#"{"name": "Alice", "age": 30}"#;
        assert!(validator.validate(valid_json).is_ok());

        let missing_field = r#"{"name": "Bob"}"#;
        assert!(matches!(
            validator.validate(missing_field),
            Err(ValidationError::MissingField(_))
        ));

        let wrong_type = r#"{"name": "Charlie", "age": "thirty"}"#;
        assert!(matches!(
            validator.validate(wrong_type),
            Err(ValidationError::TypeMismatch(_, _))
        ));
    }

    #[test]
    fn test_custom_validator() {
        let validator = JsonValidator::new()
            .require_field("email")
            .add_validator(|v| {
                if let Some(email) = v.get("email") {
                    validate_email_format(email)
                } else {
                    Ok(())
                }
            });

        let valid_email = r#"{"email": "test@example.com"}"#;
        assert!(validator.validate(valid_email).is_ok());

        let invalid_email = r#"{"email": "not-an-email"}"#;
        assert!(matches!(
            validator.validate(invalid_email),
            Err(ValidationError::ConstraintViolation(_))
        ));
    }
}