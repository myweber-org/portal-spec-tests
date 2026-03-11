
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }

    println!("Processing complete:");
    println!("  Valid records: {}", valid_count);
    println!("  Invalid records: {}", invalid_count);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "processed_data.csv";
    
    process_csv(input_file, output_file)?;
    
    Ok(())
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, DataError> {
        if value.is_nan() || value.is_infinite() {
            return Err(DataError::InvalidValue);
        }
        
        if timestamp < 0 {
            return Err(DataError::InvalidTimestamp);
        }
        
        Ok(Self {
            id,
            value,
            timestamp,
        })
    }
    
    pub fn validate(&self) -> Result<(), DataError> {
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(DataError::ValidationFailed(
                format!("Value {} out of range [0, 1000]", self.value)
            ));
        }
        
        let current_time = chrono::Utc::now().timestamp();
        if self.timestamp > current_time + 3600 {
            return Err(DataError::ValidationFailed(
                "Timestamp is more than 1 hour in the future".to_string()
            ));
        }
        
        Ok(())
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<Self, DataError> {
        if multiplier <= 0.0 {
            return Err(DataError::InvalidValue);
        }
        
        let transformed_value = self.value * multiplier;
        Self::new(self.id, transformed_value, self.timestamp)
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let transformed = record.transform(multiplier)?;
        processed.push(transformed);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1672531200);
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.timestamp, 1672531200);
    }
    
    #[test]
    fn test_invalid_value() {
        let record = DataRecord::new(1, f64::NAN, 1672531200);
        assert!(matches!(record, Err(DataError::InvalidValue)));
    }
    
    #[test]
    fn test_validation_success() {
        let record = DataRecord::new(1, 500.0, 1672531200).unwrap();
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_transform_record() {
        let record = DataRecord::new(1, 100.0, 1672531200).unwrap();
        let transformed = record.transform(2.0).unwrap();
        assert_eq!(transformed.value, 200.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
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
            
            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[2].to_string();
            let valid = match parts[3].to_lowercase().as_str() {
                "true" => true,
                "false" => false,
                _ => continue,
            };
            
            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.valid)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.records
            .iter()
            .filter(|record| record.valid)
            .collect();
        
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|record| record.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_statistics(&self) -> Statistics {
        let valid_count = self.filter_valid().len();
        let total_count = self.count_records();
        let average = self.calculate_average().unwrap_or(0.0);
        
        Statistics {
            total_records: total_count,
            valid_records: valid_count,
            invalid_records: total_count - valid_count,
            average_value: average,
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub total_records: usize,
    pub valid_records: usize,
    pub invalid_records: usize,
    pub average_value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,category_a,true").unwrap();
        writeln!(temp_file, "2,20.3,category_b,false").unwrap();
        writeln!(temp_file, "3,15.7,category_a,true").unwrap();
        
        let count = processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        let average = processor.calculate_average().unwrap();
        assert!((average - 13.1).abs() < 0.001);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.valid_records, 2);
        assert_eq!(stats.invalid_records, 1);
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
        if !self.validation_rules.required_keys.contains(&key) {
            return Err(format!("Key '{}' is not in required keys list", key));
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!("Value {} is outside allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, Statistics> {
        let mut stats = HashMap::new();
        
        for (key, values) in &self.data {
            if values.is_empty() {
                continue;
            }

            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[count as usize / 2]
            };

            stats.insert(key.clone(), Statistics {
                mean,
                median,
                std_dev,
                min: *sorted_values.first().unwrap(),
                max: *sorted_values.last().unwrap(),
                count: values.len(),
            });
        }
        
        stats
    }

    pub fn normalize_data(&mut self) {
        for values in self.data.values_mut() {
            if values.is_empty() {
                continue;
            }
            
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let range = max - min;
            
            if range > 0.0 {
                for value in values.iter_mut() {
                    *value = (*value - min) / range;
                }
            }
        }
    }
}

pub struct Statistics {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
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
    fn test_data_processor() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["temperature".to_string(), "humidity".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        
        assert!(processor.add_dataset(
            "temperature".to_string(),
            vec![20.5, 22.3, 18.7, 25.1]
        ).is_ok());
        
        assert!(processor.add_dataset(
            "pressure".to_string(),
            vec![1013.0, 1015.0]
        ).is_err());
        
        let stats = processor.calculate_statistics();
        assert!(stats.contains_key("temperature"));
        
        processor.normalize_data();
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validation_rules: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformation_pipeline: Vec<Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, field: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validation_rules.insert(field.to_string(), validator);
    }

    pub fn add_transformation(&mut self, transformer: Box<dyn Fn(String) -> String>) {
        self.transformation_pipeline.push(transformer);
    }

    pub fn process_record(&self, record: &mut HashMap<String, String>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for (field, validator) in &self.validation_rules {
            if let Some(value) = record.get(field) {
                if !validator(value) {
                    errors.push(format!("Validation failed for field: {}", field));
                }
            } else {
                errors.push(format!("Missing required field: {}", field));
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        for transformer in &self.transformation_pipeline {
            for (_, value) in record.iter_mut() {
                *value = transformer(value.clone());
            }
        }

        Ok(())
    }

    pub fn batch_process(
        &self,
        records: &mut [HashMap<String, String>],
    ) -> Vec<Result<(), Vec<String>>> {
        records
            .iter_mut()
            .map(|record| self.process_record(record))
            .collect()
    }
}

pub fn create_email_validator() -> Box<dyn Fn(&str) -> bool> {
    Box::new(|email: &str| {
        email.contains('@') && email.contains('.') && email.len() > 5
    })
}

pub fn create_uppercase_transformer() -> Box<dyn Fn(String) -> String> {
    Box::new(|s: String| s.to_uppercase())
}

pub fn create_trim_transformer() -> Box<dyn Fn(String) -> String> {
    Box::new(|s: String| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule("email", create_email_validator());
        processor.add_transformation(create_trim_transformer());
        processor.add_transformation(create_uppercase_transformer());

        let mut record = HashMap::new();
        record.insert("email".to_string(), "  test@example.com  ".to_string());
        record.insert("name".to_string(), "john doe".to_string());

        let result = processor.process_record(&mut record);
        assert!(result.is_ok());
        assert_eq!(record.get("email").unwrap(), "TEST@EXAMPLE.COM");
        assert_eq!(record.get("name").unwrap(), "JOHN DOE");
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule("email", create_email_validator());

        let mut record = HashMap::new();
        record.insert("email".to_string(), "invalid-email".to_string());

        let result = processor.process_record(&mut record);
        assert!(result.is_err());
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

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = if count > 0.0 { sum / count } else { 0.0 };

        let variance: f64 = if count > 0.0 {
            values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count
        } else {
            0.0
        };

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let csv_data = "id,name,value,category\n1,Test1,10.5,A\n2,Test2,15.3,B\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 2);
        
        let errors = processor.validate_data();
        assert!(errors.is_empty());
        
        let (mean, _, _) = processor.calculate_statistics();
        assert!((mean - 12.9).abs() < 0.001);
        
        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 1);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

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
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
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
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut categories = std::collections::HashMap::new();
        for record in &self.records {
            categories
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,name,value,category\n1,ItemA,10.5,Alpha\n2,ItemB,15.0,Beta\n3,ItemC,-5.0,Alpha"
        )
        .unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(file.path()).unwrap();

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.validate_records().len(), 2);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 12.75).abs() < 0.001);

        let groups = processor.group_by_category();
        assert_eq!(groups.get("Alpha").unwrap().len(), 2);
        assert_eq!(groups.get("Beta").unwrap().len(), 1);
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
    active: bool,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let active_count = records.iter().filter(|r| r.active).count();
    
    (sum, average, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_data_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,-5.0,false").unwrap();
        writeln!(temp_file, "3,Test3,15.0,true").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].value, 15.0);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (sum, avg, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
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
    InvalidValue(f64),
    MissingField(String),
    DuplicateId(u32),
    CategoryNotFound(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(val) => write!(f, "Invalid value: {}", val),
            ProcessingError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ProcessingError::DuplicateId(id) => write!(f, "Duplicate record ID: {}", id),
            ProcessingError::CategoryNotFound(cat) => write!(f, "Category not found: {}", cat),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(valid_categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            categories: valid_categories,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        if record.value <= 0.0 || record.value > 1000.0 {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if !self.categories.contains(&record.category) {
            return Err(ProcessingError::CategoryNotFound(record.category));
        }

        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::DuplicateId(record.id));
        }

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let count = self.records.len() as f64;
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        let avg = sum / count;
        
        let min = self.records.values()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
        
        let max = self.records.values()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        stats.insert("total_records".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("minimum".to_string(), min);
        stats.insert("maximum".to_string(), max);

        stats
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        self.records.remove(&id)
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let categories = vec!["A".to_string(), "B".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_add_invalid_value() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: -10.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "A".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats = processor.calculate_statistics();
        assert_eq!(stats["total_records"], 3.0);
        assert_eq!(stats["sum"], 60.0);
        assert_eq!(stats["average"], 20.0);
        assert_eq!(stats["minimum"], 10.0);
        assert_eq!(stats["maximum"], 30.0);
    }

    #[test]
    fn test_value_transformation() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };

        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 2.0);
        
        let retrieved = processor.get_record(1).unwrap();
        assert_eq!(retrieved.value, 20.0);
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
pub enum DataError {
    InvalidId,
    InvalidValue,
    InvalidCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid value field"),
            DataError::InvalidCategory => write!(f, "Invalid category"),
            DataError::DuplicateRecord => write!(f, "Duplicate record detected"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub total_records: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value < 0.0 || record.value.is_nan() {
            return Err(DataError::InvalidValue);
        }

        if record.category.trim().is_empty() {
            return Err(DataError::InvalidCategory);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.records.insert(record.id, record.clone());
        self.update_category_stats(&record);

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                total_records: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.total_records += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.total_records as f64;
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn filter_by_value(&self, min_value: f64, max_value: f64) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.value >= min_value && record.value <= max_value)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
        self.recalculate_stats();
    }

    fn recalculate_stats(&mut self) {
        self.category_stats.clear();
        for record in self.records.values() {
            self.update_category_stats(record);
        }
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.records.values().map(|r| r.value).sum::<f64>() / self.records.len() as f64
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
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.total_records(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Invalid".to_string(),
            value: 50.0,
            category: "B".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "R1".to_string(),
                value: 10.0,
                category: "X".to_string(),
            },
            DataRecord {
                id: 2,
                name: "R2".to_string(),
                value: 20.0,
                category: "X".to_string(),
            },
            DataRecord {
                id: 3,
                name: "R3".to_string(),
                value: 30.0,
                category: "Y".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats_x = processor.get_category_stats("X").unwrap();
        assert_eq!(stats_x.total_records, 2);
        assert_eq!(stats_x.average_value, 15.0);

        let stats_y = processor.get_category_stats("Y").unwrap();
        assert_eq!(stats_y.total_records, 1);
        assert_eq!(stats_y.average_value, 30.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "Z".to_string(),
        };

        processor.add_record(record).unwrap();
        processor.transform_values(|v| v * 2.0);

        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 20.0);
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,15.0,B").unwrap();

        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "X".to_string() },
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 15.0);
        assert_eq!(variance, 25.0);
        assert_eq!(std_dev, 5.0);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
        ];

        let filtered = filter_by_category(records, "X");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}