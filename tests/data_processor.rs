
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct DataError {
    message: String,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Data processing error: {}", self.message)
    }
}

impl Error for DataError {}

impl DataError {
    pub fn new(msg: &str) -> Self {
        DataError {
            message: msg.to_string(),
        }
    }
}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&str) -> Result<(), DataError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&str) -> Result<(), DataError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn process(&self, input: &str) -> Result<String, DataError> {
        for rule in &self.validation_rules {
            rule(input)?;
        }

        let transformed = Self::transform_data(input);
        Ok(transformed)
    }

    fn transform_data(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .to_uppercase()
    }
}

pub fn validate_length(input: &str) -> Result<(), DataError> {
    if input.len() < 5 {
        Err(DataError::new("Input must be at least 5 characters"))
    } else if input.len() > 100 {
        Err(DataError::new("Input must not exceed 100 characters"))
    } else {
        Ok(())
    }
}

pub fn validate_alphanumeric(input: &str) -> Result<(), DataError> {
    if input.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
        Ok(())
    } else {
        Err(DataError::new("Input must contain only alphanumeric characters and spaces"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_length);
        processor.add_validation_rule(validate_alphanumeric);

        let result = processor.process("Hello World 123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HELLO WORLD 123");

        let result = processor.process("Hi");
        assert!(result.is_err());

        let result = processor.process("Invalid@Symbol");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_functions() {
        assert!(validate_length("ValidInput").is_ok());
        assert!(validate_length("Short").is_err());
        
        assert!(validate_alphanumeric("Alpha123").is_ok());
        assert!(validate_alphanumeric("Invalid!").is_err());
    }
}