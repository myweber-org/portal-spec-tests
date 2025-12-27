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
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            let _ = lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64)> {
        let mut values = Vec::new();
        
        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        Some((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            vec!["50000.0".to_string()],
            vec!["45000.0".to_string()],
            vec!["55000.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0);
        
        assert!(stats.is_some());
        let (mean, std_dev) = stats.unwrap();
        assert!((mean - 50000.0).abs() < 0.1);
        assert!(std_dev > 0.0);
    }
}