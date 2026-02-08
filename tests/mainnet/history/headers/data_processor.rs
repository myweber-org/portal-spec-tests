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
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["field1".to_string(), "field2".to_string()];
        let invalid_record = vec!["".to_string(), "field2".to_string()];

        assert!(processor.validate_record(&valid_record, 2));
        assert!(!processor.validate_record(&invalid_record, 2));
    }

    #[test]
    fn test_extract_column() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 0);

        assert_eq!(column, vec!["a".to_string(), "c".to_string()]);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: ValidationRules,
}

pub struct ValidationRules {
    min_value: f64,
    max_value: f64,
    required_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if values.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!(
                    "Value {} is outside allowed range [{}, {}]",
                    value, self.validation_rules.min_value, self.validation_rules.max_value
                ));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();

            Statistics {
                mean,
                variance,
                std_dev,
                count: values.len(),
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            }
        })
    }

    pub fn validate_all_datasets(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for required_key in &self.validation_rules.required_keys {
            if !self.data.contains_key(required_key) {
                errors.push(format!("Missing required dataset: {}", required_key));
            }
        }

        for (key, values) in &self.data {
            if values.len() < 2 {
                errors.push(format!("Dataset '{}' has insufficient data points", key));
            }
        }

        errors
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(key) {
            let stats = self.calculate_statistics(key).unwrap();
            
            if stats.std_dev == 0.0 {
                return Err("Cannot normalize data with zero standard deviation".to_string());
            }

            for value in values.iter_mut() {
                *value = (*value - stats.mean) / stats.std_dev;
            }
            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", key))
        }
    }

    pub fn get_dataset_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn merge_datasets(&mut self, target_key: &str, source_keys: &[&str]) -> Result<(), String> {
        let mut merged_data = Vec::new();
        
        for &source_key in source_keys {
            if let Some(data) = self.data.get(source_key) {
                merged_data.extend(data.clone());
            } else {
                return Err(format!("Source dataset '{}' not found", source_key));
            }
        }

        if merged_data.is_empty() {
            return Err("No data to merge".to_string());
        }

        self.data.insert(target_key.to_string(), merged_data);
        Ok(())
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

impl ValidationRules {
    pub fn new(min_value: f64, max_value: f64, required_keys: Vec<String>) -> Self {
        ValidationRules {
            min_value,
            max_value,
            required_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor_creation() {
        let rules = ValidationRules::new(0.0, 100.0, vec!["temperature".to_string()]);
        let processor = DataProcessor::new(rules);
        assert_eq!(processor.get_dataset_keys().len(), 0);
    }

    #[test]
    fn test_add_valid_dataset() {
        let rules = ValidationRules::new(0.0, 100.0, vec![]);
        let mut processor = DataProcessor::new(rules);
        
        let result = processor.add_dataset("test".to_string(), vec![10.0, 20.0, 30.0]);
        assert!(result.is_ok());
        assert_eq!(processor.get_dataset_keys(), vec!["test"]);
    }

    #[test]
    fn test_add_invalid_dataset_out_of_range() {
        let rules = ValidationRules::new(0.0, 100.0, vec![]);
        let mut processor = DataProcessor::new(rules);
        
        let result = processor.add_dataset("test".to_string(), vec![10.0, 150.0, 30.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let rules = ValidationRules::new(0.0, 100.0, vec![]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("test".to_string(), vec![10.0, 20.0, 30.0]).unwrap();
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_normalize_data() {
        let rules = ValidationRules::new(0.0, 100.0, vec![]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("test".to_string(), vec![10.0, 20.0, 30.0]).unwrap();
        processor.normalize_data("test").unwrap();
        
        let normalized = processor.data.get("test").unwrap();
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert!((stats.mean - 0.0).abs() < 0.0001);
        assert!((stats.std_dev - 1.0).abs() < 0.0001);
    }
}