
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStatistics>,
}

#[derive(Debug, Clone)]
pub struct CategoryStatistics {
    pub count: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Err(ProcessingError::InvalidData("No records to process".to_string()));
        }

        self.calculate_statistics();
        self.transform_values()?;
        Ok(())
    }

    pub fn get_category_statistics(&self, category: &str) -> Option<&CategoryStatistics> {
        self.category_stats.get(category)
    }

    pub fn get_all_statistics(&self) -> &HashMap<String, CategoryStatistics> {
        &self.category_stats
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Name cannot be empty".to_string()));
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError("Value cannot be negative".to_string()));
        }
        
        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Category cannot be empty".to_string()));
        }
        
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStatistics {
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });
        
        stats.count += 1;
        stats.total_value += record.value;
    }

    fn calculate_statistics(&mut self) {
        for (_, stats) in self.category_stats.iter_mut() {
            if stats.count > 0 {
                stats.average_value = stats.total_value / stats.count as f64;
            }
        }
    }

    fn transform_values(&mut self) -> Result<(), ProcessingError> {
        for record in &mut self.records {
            if record.value > 1000.0 {
                record.value = record.value.log10();
            } else if record.value < 1.0 {
                record.value = record.value.exp();
            }
            
            if record.value.is_nan() || record.value.is_infinite() {
                return Err(ProcessingError::TransformationFailed(
                    format!("Invalid value after transformation for record {}", record.id)
                ));
            }
        }
        Ok(())
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 42.5,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_process_records() {
        let mut processor = DataProcessor::new();
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 100.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 200.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 50.0,
                category: "CategoryB".to_string(),
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert!(processor.process_records().is_ok());
        
        let stats = processor.get_category_statistics("CategoryA").unwrap();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.total_value, 300.0);
        assert_eq!(stats.average_value, 150.0);
    }

    #[test]
    fn test_filter_by_threshold() {
        let mut processor = DataProcessor::new();
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 100.0,
                category: "Test".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 50.0,
                category: "Test".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 150.0,
                category: "Test".to_string(),
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let filtered = processor.filter_by_threshold(100.0);
        assert_eq!(filtered.len(), 2);
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
        
        if let Some((mean, _, std_dev)) = stats {
            assert!((mean - 30.0).abs() < 0.001);
            assert!(std_dev > 0.0);
        }
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(15.5));
        assert_eq!(processor.calculate_median(), Some(15.7));
        
        let filtered = processor.filter_by_threshold(12.0);
        assert_eq!(filtered.len(), 2);
        
        let freq = processor.get_frequency_distribution();
        assert_eq!(freq.get("A"), Some(&2));
        assert_eq!(freq.get("B"), Some(&1));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ProcessedData {
    pub record_id: u64,
    pub normalized_values: Vec<f64>,
    pub statistics: DataStatistics,
    pub is_valid: bool,
}

#[derive(Debug)]
pub struct DataStatistics {
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
}

pub struct DataProcessor {
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor {
            validation_threshold: threshold,
        }
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<ProcessedData, Box<dyn Error>> {
        if record.values.is_empty() {
            return Err("Empty values vector".into());
        }

        let normalized = self.normalize_values(&record.values);
        let stats = self.calculate_statistics(&normalized);
        let is_valid = self.validate_record(&normalized, &stats);

        Ok(ProcessedData {
            record_id: record.id,
            normalized_values: normalized,
            statistics: stats,
            is_valid,
        })
    }

    fn normalize_values(&self, values: &[f64]) -> Vec<f64> {
        if values.len() < 2 {
            return values.to_vec();
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return values.to_vec();
        }

        values.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn calculate_statistics(&self, values: &[f64]) -> DataStatistics {
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        DataStatistics {
            mean,
            variance,
            min,
            max,
        }
    }

    fn validate_record(&self, values: &[f64], stats: &DataStatistics) -> bool {
        if values.is_empty() {
            return false;
        }

        let outlier_count = values.iter()
            .filter(|&&x| (x - stats.mean).abs() > self.validation_threshold * stats.variance.sqrt())
            .count();

        outlier_count <= values.len() / 10
    }

    pub fn batch_process(&self, records: &[DataRecord]) -> Vec<Result<ProcessedData, Box<dyn Error>>> {
        records.iter()
            .map(|record| self.process_record(record))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(3.0);
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_values(&values);
        
        let mean = normalized.iter().sum::<f64>() / normalized.len() as f64;
        let variance = normalized.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / normalized.len() as f64;
        
        assert!(mean.abs() < 1e-10);
        assert!((variance - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(3.0);
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&values);
        
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_record_processing() {
        let processor = DataProcessor::new(3.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata,
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.record_id, 1);
        assert!(processed.is_valid);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct Record {
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
        DataProcessor { records: Vec::new() }
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

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut map = std::collections::HashMap::new();
        for record in &self.records {
            map.entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,ItemA,10.5,CategoryX").unwrap();
        writeln!(file, "2,ItemB,15.0,CategoryY").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 2);
        assert_eq!(processor.calculate_total(), 25.5);
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
                input.contains('@') && input.contains('.')
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
            Box::new(|input: String| {
                input.to_uppercase()
            })
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|input: String| {
                input.trim().to_string()
            })
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
    
    pub fn process_pipeline(&self, input: String, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = input;
        
        for (op_type, op_name) in operations {
            match op_type {
                "validate" => {
                    if !self.validate(op_name, &result) {
                        return Err(format!("Validation '{}' failed for input", op_name));
                    }
                }
                "transform" => {
                    result = self.transform(op_name, result)
                        .ok_or_else(|| format!("Transformer '{}' not found", op_name))?;
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
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
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("numeric", "12345"));
        assert!(!processor.validate("numeric", "123abc"));
    }
    
    #[test]
    fn test_transformation_pipeline() {
        let processor = DataProcessor::new();
        let result = processor.process_pipeline(
            "  hello world  ".to_string(),
            vec![("transform", "trim"), ("transform", "uppercase")]
        );
        
        assert_eq!(result.unwrap(), "HELLO WORLD");
    }
}