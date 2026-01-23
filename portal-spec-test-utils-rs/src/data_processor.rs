
use std::collections::HashMap;

pub struct DataProcessor {
    validation_rules: HashMap<String, Box<dyn Fn(&str) -> bool>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            validation_rules: HashMap::new(),
        };
        
        processor.add_validation_rule("email", |value| {
            value.contains('@') && value.contains('.')
        });
        
        processor.add_validation_rule("phone", |value| {
            value.chars().all(|c| c.is_numeric()) && value.len() >= 10
        });
        
        processor
    }
    
    pub fn add_validation_rule<F>(&mut self, rule_name: &str, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validation_rules.insert(rule_name.to_string(), Box::new(validator));
    }
    
    pub fn validate(&self, rule_name: &str, value: &str) -> bool {
        match self.validation_rules.get(rule_name) {
            Some(validator) => validator(value),
            None => false,
        }
    }
    
    pub fn transform_data(&self, input: &str, transformation: &str) -> String {
        match transformation {
            "uppercase" => input.to_uppercase(),
            "lowercase" => input.to_lowercase(),
            "trim" => input.trim().to_string(),
            "reverse" => input.chars().rev().collect(),
            _ => input.to_string(),
        }
    }
    
    pub fn process_pipeline(&self, data: &str, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = data.to_string();
        
        for (op_type, op_value) in operations {
            match op_type {
                "validate" => {
                    if !self.validate(op_value, &result) {
                        return Err(format!("Validation failed for rule: {}", op_value));
                    }
                }
                "transform" => {
                    result = self.transform_data(&result, op_value);
                }
                _ => return Err(format!("Unknown operation: {}", op_type)),
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
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }
    
    #[test]
    fn test_phone_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("phone", "1234567890"));
        assert!(!processor.validate("phone", "abc123"));
    }
    
    #[test]
    fn test_transform_operations() {
        let processor = DataProcessor::new();
        assert_eq!(processor.transform_data("hello", "uppercase"), "HELLO");
        assert_eq!(processor.transform_data("WORLD", "lowercase"), "world");
        assert_eq!(processor.transform_data("  test  ", "trim"), "test");
        assert_eq!(processor.transform_data("rust", "reverse"), "tsur");
    }
    
    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::new();
        let operations = vec![
            ("validate", "email"),
            ("transform", "uppercase"),
            ("transform", "trim"),
        ];
        
        let result = processor.process_pipeline("  user@domain.com  ", operations);
        assert_eq!(result, Ok("USER@DOMAIN.COM".to_string()));
    }
}