
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            match self.process(record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
}

pub fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationError(
            "Timestamp cannot be negative".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_values_length(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::ValidationError(
            "Values array cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let sum: f64 = record.values.iter().sum();
    if sum == 0.0 {
        return Err(ProcessingError::TransformationFailed(
            "Cannot normalize zero-sum vector".to_string(),
        ));
    }

    let normalized_values: Vec<f64> = record.values.iter().map(|&v| v / sum).collect();
    
    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

pub fn add_processing_timestamp(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let processing_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ProcessingError::TransformationFailed("System time error".to_string()))?
        .as_secs() as i64;

    let mut metadata = record.metadata;
    metadata.insert("processed_at".to_string(), processing_time.to_string());
    
    Ok(DataRecord {
        metadata,
        ..record
    })
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
        let mut reader = Reader::from_reader(file);

        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_data(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.is_empty() {
                errors.push(format!("Record {}: Name is empty", index));
            }

            if record.value < 0.0 {
                errors.push(format!("Record {}: Value is negative", index));
            }

            if !["A", "B", "C"].contains(&record.category.as_str()) {
                errors.push(format!("Record {}: Invalid category", index));
            }
        }

        errors
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,A").unwrap();
        writeln!(temp_file, "2,Item2,20.0,B").unwrap();
        writeln!(temp_file, "3,Item3,15.75,C").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);

        let errors = processor.validate_data();
        assert!(errors.is_empty());

        let stats = processor.calculate_statistics();
        assert!((stats.0 - 15.416).abs() < 0.001);

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
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
    
    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
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
    
    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }
    
    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].trim();
            
            match DataRecord::new(id, value, category) {
                Ok(record) => self.records.push(record),
                Err(e) => return Err(format!("Error at line {}: {}", line_num + 1, e).into()),
            }
        }
        
        Ok(())
    }
    
    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 100.0, "electronics").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "electronics");
    }
    
    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(1, -10.0, "test").is_err());
        assert!(DataRecord::new(1, 10.0, "").is_err());
    }
    
    #[test]
    fn test_tax_calculation() {
        let record = DataRecord::new(1, 100.0, "test").unwrap();
        assert_eq!(record.calculate_tax(0.1), 10.0);
    }
    
    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,100.5,electronics")?;
        writeln!(temp_file, "2,200.0,furniture")?;
        writeln!(temp_file, "# This is a comment")?;
        writeln!(temp_file, "3,150.75,electronics")?;
        
        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path())?;
        
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.total_value(), 451.25);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        Ok(())
    }
    
    #[test]
    fn test_average_value() {
        let mut processor = DataProcessor::new();
        assert!(processor.average_value().is_none());
        
        processor.add_record(DataRecord::new(1, 100.0, "test").unwrap());
        processor.add_record(DataRecord::new(2, 200.0, "test").unwrap());
        
        assert_eq!(processor.average_value(), Some(150.0));
    }
}
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
    InvalidValue,
    MissingField,
    CategoryNotFound,
    TransformationFailed,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::MissingField => write!(f, "Required field is missing"),
            ProcessingError::CategoryNotFound => write!(f, "Category not found in mapping"),
            ProcessingError::TransformationFailed => write!(f, "Data transformation failed"),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    category_mapping: HashMap<String, String>,
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(category_mapping: HashMap<String, String>, validation_threshold: f64) -> Self {
        DataProcessor {
            category_mapping,
            validation_threshold,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < 0.0 || record.value > self.validation_threshold {
            return Err(ProcessingError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(ProcessingError::MissingField);
        }

        Ok(())
    }

    pub fn transform_category(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        match self.category_mapping.get(&record.category) {
            Some(new_category) => {
                record.category = new_category.clone();
                Ok(())
            }
            None => Err(ProcessingError::CategoryNotFound),
        }
    }

    pub fn normalize_value(&self, record: &mut DataRecord) {
        record.value = record.value / self.validation_threshold;
    }

    pub fn process_records(
        &self,
        records: &mut [DataRecord],
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::new();

        for record in records.iter_mut() {
            self.validate_record(record)?;
            self.transform_category(record)?;
            self.normalize_value(record);
            processed_records.push(record.clone());
        }

        Ok(processed_records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_mapping() -> HashMap<String, String> {
        let mut mapping = HashMap::new();
        mapping.insert("old_cat".to_string(), "new_cat".to_string());
        mapping.insert("legacy".to_string(), "modern".to_string());
        mapping
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(create_test_mapping(), 100.0);
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "old_cat".to_string(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(create_test_mapping(), 100.0);
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 150.0,
            category: "old_cat".to_string(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_category_transformation() {
        let processor = DataProcessor::new(create_test_mapping(), 100.0);
        let mut record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "old_cat".to_string(),
        };

        assert!(processor.transform_category(&mut record).is_ok());
        assert_eq!(record.category, "new_cat");
    }

    #[test]
    fn test_value_normalization() {
        let processor = DataProcessor::new(create_test_mapping(), 100.0);
        let mut record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "old_cat".to_string(),
        };

        processor.normalize_value(&mut record);
        assert_eq!(record.value, 0.5);
    }
}