
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: HashMap<String, ValidationRule>,
}

pub struct ValidationRule {
    min_value: Option<f64>,
    max_value: Option<f64>,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) -> Result<(), String> {
        if self.data.contains_key(&name) {
            return Err(format!("Dataset '{}' already exists", name));
        }
        
        if let Some(rule) = self.validation_rules.get(&name) {
            if rule.required && values.is_empty() {
                return Err(format!("Dataset '{}' cannot be empty", name));
            }
            
            for &value in &values {
                if let Some(min) = rule.min_value {
                    if value < min {
                        return Err(format!("Value {} below minimum {}", value, min));
                    }
                }
                
                if let Some(max) = rule.max_value {
                    if value > max {
                        return Err(format!("Value {} above maximum {}", value, max));
                    }
                }
            }
        }
        
        self.data.insert(name, values);
        Ok(())
    }

    pub fn set_validation_rule(&mut self, dataset_name: String, rule: ValidationRule) {
        self.validation_rules.insert(dataset_name, rule);
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.data.get(dataset_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = if count > 0 { sum / count as f64 } else { 0.0 };
            
            let variance = if count > 1 {
                let squared_diff: f64 = values.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum();
                squared_diff / (count - 1) as f64
            } else {
                0.0
            };
            
            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev: variance.sqrt(),
            }
        })
    }

    pub fn transform_data<F>(&mut self, dataset_name: &str, transform_fn: F) -> Result<(), String>
    where
        F: Fn(f64) -> f64,
    {
        if let Some(values) = self.data.get_mut(dataset_name) {
            for value in values {
                *value = transform_fn(*value);
            }
            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", dataset_name))
        }
    }

    pub fn merge_datasets(&mut self, target_name: String, source_names: Vec<&str>) -> Result<(), String> {
        let mut merged_data = Vec::new();
        
        for &source_name in &source_names {
            if let Some(data) = self.data.get(source_name) {
                merged_data.extend(data.clone());
            } else {
                return Err(format!("Source dataset '{}' not found", source_name));
            }
        }
        
        self.data.insert(target_name, merged_data);
        Ok(())
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl ValidationRule {
    pub fn new(min: Option<f64>, max: Option<f64>, required: bool) -> Self {
        ValidationRule {
            min_value: min,
            max_value: max,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_validate_dataset() {
        let mut processor = DataProcessor::new();
        
        let rule = ValidationRule::new(Some(0.0), Some(100.0), true);
        processor.set_validation_rule("scores".to_string(), rule);
        
        let result = processor.add_dataset("scores".to_string(), vec![85.5, 92.0, 78.3]);
        assert!(result.is_ok());
        
        let invalid_result = processor.add_dataset("scores".to_string(), vec![120.0]);
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("test").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }
}
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }

        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }

        for &value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err(format!("Invalid value detected: {}", value));
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<DataRecord, String>> {
    let mut results = Vec::new();

    for record in records {
        match record.validate() {
            Ok(_) => {
                let mut processed = record.clone();
                processed.normalize();
                results.push(Ok(processed));
            }
            Err(e) => {
                results.push(Err(format!("Record {} failed: {}", record.id, e)));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![2.0, 4.0, 6.0]);
        record.normalize();
        assert_eq!(record.values, vec![1.0 / 3.0, 2.0 / 3.0, 1.0]);
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

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
        let valid_record = vec!["test".to_string(), "data".to_string()];
        let invalid_record = vec!["".to_string(), "data".to_string()];

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
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }
            
            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2];
            
            match DataRecord::new(id, value, category) {
                Ok(record) => self.records.push(record),
                Err(e) => eprintln!("Skipping invalid record at line {}: {}", line_num + 1, e),
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.0,category_b").unwrap();
        writeln!(temp_file, "3,15.5,category_a").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.total_records(), 3);
        assert_eq!(processor.calculate_average(), Some(15.333333333333334));
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();

        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        groups
    }

    pub fn get_statistics(&self) -> Statistics {
        let valid_records = self.validate_records();
        let count = valid_records.len();

        let min = valid_records
            .iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);

        let max = valid_records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        let average = if count > 0 { sum / count as f64 } else { 0.0 };

        Statistics {
            total_records: self.records.len(),
            valid_records: count,
            min_value: min,
            max_value: max,
            average_value: average,
        }
    }
}

pub struct Statistics {
    pub total_records: usize,
    pub valid_records: usize,
    pub min_value: f64,
    pub max_value: f64,
    pub average_value: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Total: {}, Valid: {}, Min: {:.2}, Max: {:.2}, Avg: {:.2}",
            self.total_records,
            self.valid_records,
            self.min_value,
            self.max_value,
            self.average_value
        )
    }
}