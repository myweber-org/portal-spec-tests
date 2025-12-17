
use std::collections::HashMap;

pub struct DataProcessor {
    validation_rules: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformation_pipelines: HashMap<String, Vec<Box<dyn Fn(String) -> String>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipelines: HashMap::new(),
        }
    }

    pub fn add_validation_rule(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validation_rules.insert(name.to_string(), validator);
    }

    pub fn add_transformation_pipeline(&mut self, name: &str, pipeline: Vec<Box<dyn Fn(String) -> String>>) {
        self.transformation_pipelines.insert(name.to_string(), pipeline);
    }

    pub fn validate_data(&self, rule_name: &str, data: &str) -> Result<(), String> {
        match self.validation_rules.get(rule_name) {
            Some(validator) => {
                if validator(data) {
                    Ok(())
                } else {
                    Err(format!("Validation failed for rule: {}", rule_name))
                }
            }
            None => Err(format!("Validation rule not found: {}", rule_name)),
        }
    }

    pub fn transform_data(&self, pipeline_name: &str, data: String) -> Result<String, String> {
        match self.transformation_pipelines.get(pipeline_name) {
            Some(pipeline) => {
                let mut result = data;
                for transform in pipeline {
                    result = transform(result);
                }
                Ok(result)
            }
            None => Err(format!("Transformation pipeline not found: {}", pipeline_name)),
        }
    }

    pub fn process_data(&self, validation_rule: &str, pipeline_name: &str, data: String) -> Result<String, String> {
        self.validate_data(validation_rule, &data)?;
        self.transform_data(pipeline_name, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        processor.add_validation_rule("non_empty", Box::new(|s: &str| !s.trim().is_empty()));
        processor.add_validation_rule("is_numeric", Box::new(|s: &str| s.chars().all(|c| c.is_digit(10))));

        let uppercase_pipeline = vec![
            Box::new(|s: String| s.to_uppercase()) as Box<dyn Fn(String) -> String>,
            Box::new(|s: String| s.trim().to_string()),
        ];

        processor.add_transformation_pipeline("uppercase_trim", uppercase_pipeline);

        assert!(processor.validate_data("non_empty", "test").is_ok());
        assert!(processor.validate_data("non_empty", "").is_err());
        assert!(processor.validate_data("is_numeric", "123").is_ok());
        assert!(processor.validate_data("is_numeric", "abc").is_err());

        let result = processor.transform_data("uppercase_trim", "  hello world  ".to_string());
        assert_eq!(result.unwrap(), "HELLO WORLD");

        let processed = processor.process_data("non_empty", "uppercase_trim", "  valid data  ".to_string());
        assert_eq!(processed.unwrap(), "VALID DATA");
    }
}