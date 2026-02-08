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

    pub fn validate_numeric_fields(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, String> {
        let mut numeric_values = Vec::new();
        
        for (row_num, row) in data.iter().enumerate() {
            if column_index >= row.len() {
                return Err(format!("Row {}: Column index out of bounds", row_num + 1));
            }
            
            match row[column_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", 
                    row_num + 1, row[column_index])),
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        let count = values.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
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
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000.5"]);
    }

    #[test]
    fn test_numeric_validation() {
        let data = vec![
            vec!["10.5".to_string(), "text".to_string()],
            vec!["20.0".to_string(), "more".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.validate_numeric_fields(&data, 0).unwrap();
        
        assert_eq!(result, vec![10.5, 20.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let processor = DataProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value < 0.0 {
            return Err(format!("Invalid value {} for record {}", record.value, record.id).into());
        }
        
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(file, "2,ItemB,20.3,Category2").unwrap();
        
        let result = process_csv_data(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "B".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 15.0);
        assert_eq!(variance, 25.0);
        assert_eq!(std_dev, 5.0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field '{}' contains invalid values", rule.field_name));
            }

            if let Some(&value) = data.iter().find(|&&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!(
                    "Value {} for field '{}' is outside allowed range [{}, {}]",
                    value, rule.field_name, rule.min_value, rule.max_value
                ));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&x| x * 2.0)
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());

        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStatistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            DatasetStatistics {
                mean,
                variance,
                count: data.len(),
                min: *data.iter().fold(&f64::INFINITY, |a, b| a.min(b)),
                max: *data.iter().fold(&f64::NEG_INFINITY, |a, b| a.max(b)),
            }
        })
    }
}

pub struct DatasetStatistics {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("temperature", -50.0, 100.0, true);
        processor.add_validation_rule(rule);

        let data = vec![20.5, 25.0, 30.2, 18.7];
        let result = processor.process_dataset("weather_data", &data);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 10.0, true);
        processor.add_validation_rule(rule);

        let data = vec![5.0, 15.0, 8.0];
        let result = processor.process_dataset("pressure_data", &data);
        
        assert!(result.is_err());
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

impl Record {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
        let mut count = 0;
        for result in rdr.deserialize() {
            let record: Record = result?;
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }
        
        Ok(count)
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

    pub fn filter_by_value(&self, threshold: f64) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.value > threshold)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn count_active(&self) -> usize {
        self.records.iter().filter(|r| r.active).count()
    }

    pub fn add_record(&mut self, record: Record) {
        if record.is_valid() {
            self.records.push(record);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            active: true,
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            active: false,
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        assert!(processor.is_empty());

        processor.add_record(Record {
            id: 1,
            name: "Item1".to_string(),
            value: 15.0,
            active: true,
        });

        processor.add_record(Record {
            id: 2,
            name: "Item2".to_string(),
            value: 5.0,
            active: false,
        });

        assert_eq!(processor.len(), 2);
        assert_eq!(processor.count_active(), 1);
        assert_eq!(processor.calculate_average(), Some(10.0));

        let filtered = processor.filter_by_value(10.0);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_csv_roundtrip() {
        let mut processor = DataProcessor::new();
        processor.add_record(Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.0,
            active: true,
        });

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        processor.save_to_csv(path).unwrap();
        
        let mut new_processor = DataProcessor::new();
        let count = new_processor.load_from_csv(path).unwrap();
        
        assert_eq!(count, 1);
        assert_eq!(new_processor.len(), 1);
        assert_eq!(new_processor.records[0].name, "Test");
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_data(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<HashMap<String, f64>>, String> {
        let mut processed = Vec::new();

        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(validated_record) => {
                    let transformed = self.transform_record(&validated_record);
                    self.cache_record(index, &transformed);
                    processed.push(transformed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }

        Ok(processed)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<HashMap<String, f64>, String> {
        for rule in &self.validation_rules {
            match record.get(&rule.field_name) {
                Some(&value) => {
                    if value < rule.min_value || value > rule.max_value {
                        return Err(format!(
                            "Field '{}' value {} is outside allowed range [{}, {}]",
                            rule.field_name, value, rule.min_value, rule.max_value
                        ));
                    }
                }
                None => {
                    if rule.required {
                        return Err(format!("Required field '{}' is missing", rule.field_name));
                    }
                }
            }
        }
        Ok(record.clone())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = record.clone();
        
        for (key, value) in transformed.iter_mut() {
            if key.starts_with("normalized_") {
                *value = (*value * 100.0).round() / 100.0;
            }
        }

        transformed
    }

    fn cache_record(&mut self, index: usize, record: &HashMap<String, f64>) {
        let cache_key = format!("record_{}", index);
        let values: Vec<f64> = record.values().copied().collect();
        self.cache.insert(cache_key, values);
    }

    pub fn get_cached_data(&self, index: usize) -> Option<&Vec<f64>> {
        let key = format!("record_{}", index);
        self.cache.get(&key)
    }

    pub fn calculate_statistics(&self, field_name: &str, dataset: &[HashMap<String, f64>]) -> Option<Statistics> {
        let values: Vec<f64> = dataset
            .iter()
            .filter_map(|record| record.get(field_name).copied())
            .collect();

        if values.is_empty() {
            return None;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        Some(Statistics {
            mean,
            variance,
            std_dev,
            count: values.len(),
            min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 100.0,
            required: true,
        });

        let test_data = vec![
            [("temperature".to_string(), 25.5)].iter().cloned().collect(),
            [("temperature".to_string(), 30.0)].iter().cloned().collect(),
        ];

        let result = processor.process_data(&test_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "pressure".to_string(),
            min_value: 0.0,
            max_value: 10.0,
            required: true,
        });

        let invalid_data = vec![
            [("pressure".to_string(), 15.0)].iter().cloned().collect(),
        ];

        let result = processor.process_data(&invalid_data);
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidInput(String),
    TransformationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ProcessingError> {
        if threshold <= 0.0 {
            return Err(ProcessingError::InvalidInput(
                "Threshold must be positive".to_string(),
            ));
        }
        Ok(DataProcessor { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::InvalidInput("Empty input array".to_string()));
        }

        let mut result = Vec::with_capacity(values.len());
        for &value in values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidInput(
                    "Invalid numeric value detected".to_string(),
                ));
            }

            let processed = self.transform_value(value)?;
            result.push(processed);
        }

        Ok(result)
    }

    fn transform_value(&self, value: f64) -> Result<f64, ProcessingError> {
        let transformed = (value * value).sqrt() / self.threshold;

        if transformed.is_nan() || transformed.is_infinite() {
            Err(ProcessingError::TransformationFailed(
                "Numerical overflow during transformation".to_string(),
            ))
        } else {
            Ok(transformed)
        }
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64), ProcessingError> {
        let processed = self.process_values(values)?;

        let sum: f64 = processed.iter().sum();
        let mean = sum / processed.len() as f64;

        let variance: f64 = processed
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / processed.len() as f64;

        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processing() {
        let processor = DataProcessor::new(2.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0];
        let result = processor.process_values(&values).unwrap();
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_invalid_threshold() {
        let result = DataProcessor::new(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_input() {
        let processor = DataProcessor::new(1.0).unwrap();
        let result = processor.process_values(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(1.0).unwrap();
        let values = vec![2.0, 4.0, 6.0, 8.0];
        let (mean, std_dev) = processor.calculate_statistics(&values).unwrap();
        assert!(mean > 0.0);
        assert!(std_dev >= 0.0);
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
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let valid_categories = ["A", "B", "C"];
        if !valid_categories.contains(&record.category.as_str()) {
            return Err(ValidationError::InvalidCategory);
        }
        
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(format!("Record with ID {} already exists", record.id).into());
        }
        
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }

    pub fn get_records_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.category == category)
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

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ValidationError::InvalidId)
        ));
    }

    #[test]
    fn test_add_and_retrieve_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 42,
            name: "Sample".to_string(),
            value: 50.5,
            category: "B".to_string(),
        };
        
        assert!(processor.add_record(record.clone()).is_ok());
        assert_eq!(processor.count_records(), 1);
        
        let retrieved = processor.get_record(42);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Sample");
    }

    #[test]
    fn test_calculate_total() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "First".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Second".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Third".to_string(),
                value: 30.0,
                category: "C".to_string(),
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.calculate_total_value(), 60.0);
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 2.0);
        
        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 20.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        self.data.clear();
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
            }
        }
        
        self.metadata.insert("source".to_string(), file_path.to_string());
        self.metadata.insert("loaded_at".to_string(), chrono::Local::now().to_rfc3339());
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }
        
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        
        stats
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x >= threshold)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
    }
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
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,10.5").unwrap();
        writeln!(temp_file, "2,20.3").unwrap();
        writeln!(temp_file, "3,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats["mean"] - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_data(15.0);
        assert_eq!(filtered.len(), 2);
    }
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<Box<dyn Fn(&[f64]) -> bool>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                Box::new(|data: &[f64]| !data.is_empty()),
                Box::new(|data: &[f64]| data.iter().all(|&x| x.is_finite())),
                Box::new(|data: &[f64]| data.len() < 10000),
            ],
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if !self.validate_data(&data) {
            return Err("Data validation failed".to_string());
        }

        let processed = self.transform_data(data);
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn validate_data(&self, data: &[f64]) -> bool {
        self.validation_rules.iter().all(|rule| rule(data))
    }

    fn transform_data(&self, mut data: Vec<f64>) -> Vec<f64> {
        if data.len() < 2 {
            return data;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            for value in data.iter_mut() {
                *value = (*value - mean) / std_dev;
            }
        }

        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        data
    }

    pub fn get_cached_result(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let test_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("test", test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 5);
        assert!(processor.get_cached_result("test").is_some());
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![f64::INFINITY, 2.0];
        
        let result = processor.process_dataset("invalid", invalid_data);
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: i64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp must be non-negative"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        if timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<f64, &'static str> {
        if multiplier <= 0.0 {
            return Err("Multiplier must be positive");
        }
        
        let transformed = self.value * multiplier;
        if transformed > 10000.0 {
            return Err("Transformed value exceeds maximum limit");
        }
        
        Ok(transformed)
    }
    
    pub fn normalize(&self, base_value: f64) -> f64 {
        if base_value == 0.0 {
            return 0.0;
        }
        self.value / base_value
    }
}

pub fn process_records(records: &[DataRecord]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = sum / count as f64;
    
    let max_value = records
        .iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);
    
    (average, max_value, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, 1234567890).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.timestamp, 1234567890);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, 500.0, 1234567890);
        assert!(matches!(result, Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_valid() {
        let record = DataRecord::new(1, 100.0, 1234567890).unwrap();
        let result = record.transform(2.5).unwrap();
        assert_eq!(result, 250.0);
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 100.0, 1000).unwrap(),
            DataRecord::new(2, 200.0, 2000).unwrap(),
            DataRecord::new(3, 300.0, 3000).unwrap(),
        ];
        
        let (avg, max, count) = process_records(&records);
        assert_eq!(avg, 200.0);
        assert_eq!(max, 300.0);
        assert_eq!(count, 3);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
            }
        }
        
        self.metadata.insert("source".to_string(), file_path.to_string());
        self.metadata.insert("loaded_timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }
        
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        
        stats
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
    }
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
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,10.5").unwrap();
        writeln!(temp_file, "2,20.3").unwrap();
        writeln!(temp_file, "3,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats["mean"] - 15.5).abs() < 0.1);
        assert_eq!(stats["count"], 3.0);
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
    }
}