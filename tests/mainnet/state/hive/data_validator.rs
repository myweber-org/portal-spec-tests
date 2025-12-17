
use serde_json::Value;

pub struct Validator {
    required_fields: Vec<String>,
}

impl Validator {
    pub fn new(required_fields: Vec<&str>) -> Self {
        Validator {
            required_fields: required_fields.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn validate(&self, data: &Value) -> Result<(), Vec<String>> {
        let mut missing_fields = Vec::new();

        for field in &self.required_fields {
            if !data.get(field).is_some() {
                missing_fields.push(field.clone());
            }
        }

        if missing_fields.is_empty() {
            Ok(())
        } else {
            Err(missing_fields)
        }
    }

    pub fn validate_with_context(&self, data: &Value, context: &str) -> Result<(), String> {
        match self.validate(data) {
            Ok(()) => Ok(()),
            Err(missing) => {
                let fields = missing.join(", ");
                Err(format!("{} validation failed. Missing fields: {}", context, fields))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validation_success() {
        let validator = Validator::new(vec!["name", "age", "email"]);
        let data = json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com"
        });

        assert!(validator.validate(&data).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let validator = Validator::new(vec!["name", "age", "email"]);
        let data = json!({
            "name": "John",
            "age": 30
        });

        let result = validator.validate(&data);
        assert!(result.is_err());
        
        if let Err(missing) = result {
            assert_eq!(missing, vec!["email"]);
        }
    }

    #[test]
    fn test_contextual_validation() {
        let validator = Validator::new(vec!["id", "timestamp"]);
        let data = json!({
            "id": "123"
        });

        let result = validator.validate_with_context(&data, "Event data");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Event data"));
    }
}