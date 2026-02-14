
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_stats(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        
        if count == 0 {
            return (0, None, None);
        }

        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
        let max = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);

        (count, Some(min), Some(max))
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error")]
    TransformationError,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

pub struct DataProcessor {
    validation_rules: Vec<ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        self.validate(record)?;
        self.transform(record)?;
        Ok(())
    }

    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            if !rule.check(record) {
                return Err(ProcessingError::ValidationFailed(
                    rule.error_message.clone(),
                ));
            }
        }
        Ok(())
    }

    fn transform(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for transformation in &self.transformation_pipeline {
            transformation.apply(record)?;
        }
        Ok(())
    }
}

pub struct ValidationRule {
    pub name: String,
    pub error_message: String,
    pub check_fn: Box<dyn Fn(&DataRecord) -> bool>,
}

impl ValidationRule {
    pub fn new<F>(name: &str, error_message: &str, check_fn: F) -> Self
    where
        F: Fn(&DataRecord) -> bool + 'static,
    {
        ValidationRule {
            name: name.to_string(),
            error_message: error_message.to_string(),
            check_fn: Box::new(check_fn),
        }
    }
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError>;
}

pub struct NormalizeTransformation {
    pub field: String,
}

impl Transformation for NormalizeTransformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if let Some(value) = record.values.get_mut(&self.field) {
            if *value < 0.0 {
                return Err(ProcessingError::TransformationError);
            }
            *value = value.clamp(0.0, 1.0);
        }
        Ok(())
    }
}

pub struct TagFilterTransformation {
    pub allowed_tags: Vec<String>,
}

impl Transformation for TagFilterTransformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        record.tags.retain(|tag| self.allowed_tags.contains(tag));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();

        processor.add_validation_rule(ValidationRule::new(
            "timestamp_check",
            "Timestamp must be positive",
            |r| r.timestamp > 0,
        ));

        processor.add_transformation(NormalizeTransformation {
            field: "temperature".to_string(),
        });

        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: {
                let mut map = HashMap::new();
                map.insert("temperature".to_string(), 25.5);
                map.insert("humidity".to_string(), 0.75);
                map
            },
            tags: vec!["sensor".to_string(), "room1".to_string()],
        };

        assert!(processor.process(&mut record).is_ok());
        assert_eq!(*record.values.get("temperature").unwrap(), 0.255);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataPoint {
    timestamp: i64,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub struct DataSet {
    points: Vec<DataPoint>,
    name: String,
}

impl DataSet {
    pub fn new(name: &str) -> Self {
        DataSet {
            points: Vec::new(),
            name: name.to_string(),
        }
    }

    pub fn add_point(&mut self, point: DataPoint) {
        self.points.push(point);
    }

    pub fn calculate_statistics(&self) -> Statistics {
        let values: Vec<f64> = self.points.iter().map(|p| p.value).collect();
        let count = values.len();
        
        if count == 0 {
            return Statistics::empty();
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Statistics {
            count,
            mean,
            std_dev,
            min,
            max,
            sum,
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataPoint> {
        self.points.iter()
            .filter(|p| p.category == category)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl Statistics {
    fn empty() -> Self {
        Statistics {
            count: 0,
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            sum: 0.0,
        }
    }
}

pub fn parse_csv_data<R: Read>(reader: R) -> Result<DataSet, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut dataset = DataSet::new("csv_import");

    for result in rdr.records() {
        let record = result?;
        
        if record.len() >= 3 {
            let timestamp = record[0].parse::<i64>()?;
            let value = record[1].parse::<f64>()?;
            let category = record[2].to_string();

            let point = DataPoint {
                timestamp,
                value,
                category,
            };

            dataset.add_point(point);
        }
    }

    Ok(dataset)
}

pub fn load_dataset_from_file<P: AsRef<Path>>(path: P) -> Result<DataSet, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_csv_data(reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_dataset_statistics() {
        let dataset = DataSet::new("test");
        let stats = dataset.calculate_statistics();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.mean, 0.0);
    }

    #[test]
    fn test_dataset_with_points() {
        let mut dataset = DataSet::new("test");
        
        dataset.add_point(DataPoint {
            timestamp: 1000,
            value: 10.5,
            category: "A".to_string(),
        });
        
        dataset.add_point(DataPoint {
            timestamp: 2000,
            value: 20.5,
            category: "B".to_string(),
        });

        let stats = dataset.calculate_statistics();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.mean, 15.5);
        assert_eq!(stats.sum, 31.0);
    }

    #[test]
    fn test_filter_by_category() {
        let mut dataset = DataSet::new("test");
        
        dataset.add_point(DataPoint {
            timestamp: 1000,
            value: 10.5,
            category: "A".to_string(),
        });
        
        dataset.add_point(DataPoint {
            timestamp: 2000,
            value: 20.5,
            category: "B".to_string(),
        });
        
        dataset.add_point(DataPoint {
            timestamp: 3000,
            value: 30.5,
            category: "A".to_string(),
        });

        let filtered = dataset.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|p| p.category == "A"));
    }
}
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
        
        processor.register_default_validators();
        processor.register_default_transformers();
        
        processor
    }
    
    fn register_default_validators(&mut self) {
        self.validators.insert(
            "email".to_string(),
            Box::new(|input: &str| {
                input.contains('@') && input.contains('.') && input.len() > 5
            })
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|input: &str| {
                input.chars().all(|c| c.is_ascii_digit())
            })
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|input: String| input.to_uppercase())
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|input: String| input.trim().to_string())
        );
    }
    
    pub fn validate(&self, validator_name: &str, input: &str) -> bool {
        match self.validators.get(validator_name) {
            Some(validator) => validator(input),
            None => false,
        }
    }
    
    pub fn transform(&self, transformer_name: &str, input: String) -> Option<String> {
        self.transformers.get(transformer_name)
            .map(|transformer| transformer(input))
    }
    
    pub fn process_data(&self, input: &str) -> Option<String> {
        if self.validate("email", input) {
            let trimmed = self.transform("trim", input.to_string())?;
            Some(self.transform("uppercase", trimmed)?)
        } else {
            None
        }
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
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("numeric", "12345"));
        assert!(!processor.validate("numeric", "123abc"));
    }
    
    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new();
        let result = processor.process_data("  user@domain.com  ");
        assert_eq!(result, Some("USER@DOMAIN.COM".to_string()));
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

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.has_header && index == 0 {
                continue;
            }

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

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
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
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 3);
        assert!(processor.validate_record(&records[0]));

        let stats = processor.calculate_statistics(&records, 1);
        assert!(stats.is_some());

        let (mean, _, _) = stats.unwrap();
        assert!((mean - 30.0).abs() < 0.001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if columns.len() > filter_column && columns[filter_column] == filter_value {
                results.push(columns);
            }
        }

        Ok(results)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut sum = 0.0;
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if let Some(value_str) = columns.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Ok(0.0)
        }
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
        writeln!(temp_file, "Alice,25,New York").unwrap();
        writeln!(temp_file, "Bob,30,London").unwrap();
        writeln!(temp_file, "Charlie,25,Paris").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv(2, "London").unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["Bob", "30", "London"]);
    }

    #[test]
    fn test_calculate_average() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,score").unwrap();
        writeln!(temp_file, "Alice,85.5").unwrap();
        writeln!(temp_file, "Bob,92.0").unwrap();
        writeln!(temp_file, "Charlie,78.5").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(1).unwrap();
        
        assert!((average - 85.333).abs() < 0.001);
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            record.validate()?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: Record) -> Result<(), String> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.calculate_total_value() / self.records.len() as f64)
        }
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record::new(2, "".to_string(), -10.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = Record::new(1, "Item1".to_string(), 50.0, "CategoryA".to_string());
        let record2 = Record::new(2, "Item2".to_string(), 75.0, "CategoryB".to_string());
        
        assert!(processor.add_record(record1.clone()).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.calculate_total_value(), 125.0);
        assert_eq!(processor.get_average_value(), Some(62.5));
        
        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_csv_operations() {
        let mut processor = DataProcessor::new();
        let record = Record::new(1, "TestItem".to_string(), 42.5, "TestCategory".to_string());
        processor.add_record(record).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        assert!(processor.save_to_csv(path).is_ok());

        let mut new_processor = DataProcessor::new();
        assert!(new_processor.load_from_csv(path).is_ok());
        assert_eq!(new_processor.count_records(), 1);
    }
}