use serde_json::Value;

pub fn validate_required_fields(data: &Value, required_fields: &[&str]) -> Result<(), Vec<String>> {
    let mut missing_fields = Vec::new();
    
    if let Value::Object(map) = data {
        for field in required_fields {
            if !map.contains_key(*field) {
                missing_fields.push(field.to_string());
            }
        }
    } else {
        return Err(vec!["Input must be a JSON object".to_string()]);
    }
    
    if missing_fields.is_empty() {
        Ok(())
    } else {
        Err(missing_fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_required_fields_success() {
        let data = json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com"
        });
        
        let required = vec!["name", "email"];
        assert!(validate_required_fields(&data, &required).is_ok());
    }

    #[test]
    fn test_validate_required_fields_missing() {
        let data = json!({
            "name": "John",
            "age": 30
        });
        
        let required = vec!["name", "email"];
        let result = validate_required_fields(&data, &required);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["email"]);
    }

    #[test]
    fn test_validate_required_fields_invalid_input() {
        let data = json!([1, 2, 3]);
        
        let required = vec!["name"];
        let result = validate_required_fields(&data, &required);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["Input must be a JSON object"]);
    }
}