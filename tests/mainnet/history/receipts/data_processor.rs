
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.is_empty() || record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["".to_string(), "c".to_string()],
            vec!["d".to_string(), "".to_string()],
            vec!["e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1, 2]);
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1);
        
        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: Vec<Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: Vec::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator<F>(&mut self, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.push(Box::new(validator));
    }

    pub fn add_transformer<F>(&mut self, name: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers
            .insert(name.to_string(), Box::new(transformer));
    }

    pub fn validate(&self, input: &str) -> bool {
        self.validators.iter().all(|validator| validator(input))
    }

    pub fn transform(&self, name: &str, input: String) -> Option<String> {
        self.transformers.get(name).map(|transformer| transformer(input))
    }

    pub fn process_pipeline(&self, input: &str, pipeline: &[&str]) -> Option<String> {
        if !self.validate(input) {
            return None;
        }

        let mut result = input.to_string();
        for step in pipeline {
            match self.transformers.get(*step) {
                Some(transformer) => result = transformer(result),
                None => return None,
            }
        }
        Some(result)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validator(|s| !s.trim().is_empty());
    processor.add_validator(|s| s.len() <= 1000);

    processor.add_transformer("trim", |s| s.trim().to_string());
    processor.add_transformer("uppercase", |s| s.to_uppercase());
    processor.add_transformer("reverse", |s| s.chars().rev().collect());

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("hello"));
        assert!(!processor.validate(""));
        assert!(!processor.validate(&"a".repeat(1001)));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(
            processor.transform("uppercase", "hello".to_string()),
            Some("HELLO".to_string())
        );
        assert_eq!(
            processor.transform("reverse", "abc".to_string()),
            Some("cba".to_string())
        );
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let pipeline = vec!["trim", "uppercase", "reverse"];
        assert_eq!(
            processor.process_pipeline("  hello world  ", &pipeline),
            Some("DLROW OLLEH".to_string())
        );
    }
}