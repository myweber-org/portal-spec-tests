
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        };

        processor.register_validator("email", |s| s.contains('@') && s.contains('.'));
        processor.register_validator("numeric", |s| s.chars().all(|c| c.is_ascii_digit()));
        
        processor.register_transformer("uppercase", |s| s.to_uppercase());
        processor.register_transformer("trim", |s| s.trim().to_string());
        processor.register_transformer("reverse", |s| s.chars().rev().collect());

        processor
    }

    pub fn register_validator<F>(&mut self, name: &str, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.insert(name.to_string(), Box::new(validator));
    }

    pub fn register_transformer<F>(&mut self, name: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name.to_string(), Box::new(transformer));
    }

    pub fn validate(&self, data: &str, validator_name: &str) -> bool {
        self.validators
            .get(validator_name)
            .map(|validator| validator(data))
            .unwrap_or(false)
    }

    pub fn transform(&self, data: String, transformer_name: &str) -> Option<String> {
        self.transformers
            .get(transformer_name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: String, operations: &[&str]) -> Result<String, String> {
        let mut result = data;
        
        for op in operations {
            if self.validators.contains_key(*op) {
                if !self.validate(&result, op) {
                    return Err(format!("Validation failed for operation: {}", op));
                }
            } else if self.transformers.contains_key(*op) {
                result = self.transform(result, op)
                    .ok_or_else(|| format!("Transformation failed for operation: {}", op))?;
            } else {
                return Err(format!("Unknown operation: {}", op));
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("test@example.com", "email"));
        assert!(!processor.validate("invalid-email", "email"));
    }

    #[test]
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("12345", "numeric"));
        assert!(!processor.validate("123abc", "numeric"));
    }

    #[test]
    fn test_transformations() {
        let processor = DataProcessor::new();
        
        let uppercase_result = processor.transform("hello".to_string(), "uppercase");
        assert_eq!(uppercase_result, Some("HELLO".to_string()));
        
        let trim_result = processor.transform("  spaced  ".to_string(), "trim");
        assert_eq!(trim_result, Some("spaced".to_string()));
        
        let reverse_result = processor.transform("rust".to_string(), "reverse");
        assert_eq!(reverse_result, Some("tsur".to_string()));
    }

    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::new();
        
        let result = processor.process_pipeline(
            "  test@example.com  ".to_string(),
            &["trim", "email", "uppercase"]
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TEST@EXAMPLE.COM");
    }
}