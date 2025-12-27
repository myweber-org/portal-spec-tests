use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn register_validator<N, F>(&mut self, name: N, validator: F)
    where
        N: Into<String>,
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.insert(name.into(), Box::new(validator));
    }

    pub fn register_transformer<N, F>(&mut self, name: N, transformer: F)
    where
        N: Into<String>,
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name.into(), Box::new(transformer));
    }

    pub fn validate(&self, name: &str, value: &str) -> bool {
        self.validators
            .get(name)
            .map(|validator| validator(value))
            .unwrap_or(false)
    }

    pub fn transform(&self, name: &str, value: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(value))
    }

    pub fn process_pipeline(&self, value: &str, operations: &[(&str, &str)]) -> Result<String, String> {
        let mut current = value.to_string();

        for (op_type, op_name) in operations {
            match *op_type {
                "validate" => {
                    if !self.validate(op_name, &current) {
                        return Err(format!("Validation '{}' failed for value: {}", op_name, current));
                    }
                }
                "transform" => {
                    current = self.transform(op_name, current)
                        .ok_or_else(|| format!("Transformer '{}' not found", op_name))?;
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
            }
        }

        Ok(current)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("non_empty", |s| !s.trim().is_empty());
    processor.register_validator("is_numeric", |s| s.chars().all(|c| c.is_ascii_digit()));
    processor.register_validator("is_alpha", |s| s.chars().all(|c| c.is_ascii_alphabetic()));

    processor.register_transformer("to_uppercase", |s| s.to_uppercase());
    processor.register_transformer("to_lowercase", |s| s.to_lowercase());
    processor.register_transformer("trim_spaces", |s| s.trim().to_string());
    processor.register_transformer("reverse", |s| s.chars().rev().collect());

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_pipeline() {
        let processor = create_default_processor();
        
        let result = processor.process_pipeline("  Hello  ", &[
            ("validate", "non_empty"),
            ("transform", "trim_spaces"),
            ("transform", "to_uppercase"),
            ("validate", "is_alpha"),
        ]);

        assert_eq!(result, Ok("HELLO".to_string()));
    }

    #[test]
    fn test_failed_validation() {
        let processor = create_default_processor();
        
        let result = processor.process_pipeline("", &[
            ("validate", "non_empty"),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_numeric_processing() {
        let processor = create_default_processor();
        
        let result = processor.process_pipeline("12345", &[
            ("validate", "is_numeric"),
            ("transform", "reverse"),
        ]);

        assert_eq!(result, Ok("54321".to_string()));
    }
}