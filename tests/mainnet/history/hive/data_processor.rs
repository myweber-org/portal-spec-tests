
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Some(DataStatistics {
            count,
            sum,
            mean,
            variance,
            std_dev,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<DataStatistics, String>> {
    records.iter()
        .map(|record| {
            record.validate()
                .and_then(|_| record.calculate_statistics()
                    .ok_or_else(|| "Failed to calculate statistics".to_string()))
        })
        .collect()
}

pub fn filter_valid_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records.into_iter()
        .filter(|record| record.validate().is_ok())
        .collect()
}

pub fn transform_records<F>(records: &[DataRecord], transform_fn: F) -> Vec<DataRecord>
where
    F: Fn(&DataRecord) -> DataRecord,
{
    records.iter()
        .map(transform_fn)
        .collect()
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

    pub fn process_data(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<ProcessedRecord>, String> {
        let mut results = Vec::new();

        for (index, data) in dataset.iter().enumerate() {
            match self.validate_record(data) {
                Ok(_) => {
                    let processed = self.transform_data(data);
                    self.cache.insert(format!("record_{}", index), processed.values().cloned().collect());
                    results.push(ProcessedRecord::new(processed));
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }

        Ok(results)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<(), String> {
        for rule in &self.validation_rules {
            if let Some(&value) = record.get(&rule.field_name) {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!("Field '{}' value {} out of range [{}, {}]", 
                        rule.field_name, value, rule.min_value, rule.max_value));
                }
            } else if rule.required {
                return Err(format!("Required field '{}' missing", rule.field_name));
            }
        }
        Ok(())
    }

    fn transform_data(&self, data: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = HashMap::new();
        
        for (key, value) in data {
            let new_value = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 1000.0,
                "humidity" => value.min(100.0).max(0.0),
                _ => *value,
            };
            transformed.insert(key.clone(), new_value);
        }
        
        transformed
    }

    pub fn get_cached_values(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

pub struct ProcessedRecord {
    data: HashMap<String, f64>,
    timestamp: std::time::SystemTime,
}

impl ProcessedRecord {
    pub fn new(data: HashMap<String, f64>) -> Self {
        ProcessedRecord {
            data,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn get_value(&self, field: &str) -> Option<f64> {
        self.data.get(field).copied()
    }

    pub fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }
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
            max_value: 150.0,
            required: true,
        });

        let test_data = vec![
            [("temperature".to_string(), 68.0)].iter().cloned().collect(),
            [("temperature".to_string(), 32.0)].iter().cloned().collect(),
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
            max_value: 100.0,
            required: true,
        });

        let invalid_data = vec![
            [("pressure".to_string(), 150.0)].iter().cloned().collect(),
        ];

        let result = processor.process_data(&invalid_data);
        assert!(result.is_err());
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        Self {
            id,
            name,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn transform_value<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.value = transformer(self.value);
    }

    pub fn to_json(&self) -> String {
        let metadata_json: Vec<String> = self
            .metadata
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect();

        format!(
            "{{\"id\":{},\"name\":\"{}\",\"value\":{},\"metadata\":{{{}}}}}",
            self.id,
            self.name,
            self.value,
            metadata_json.join(",")
        )
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<String>, ValidationError> {
    let mut results = Vec::new();

    for record in records {
        record.validate()?;
        record.transform_value(|v| v * 1.1);
        results.push(record.to_json());
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, "Test".to_string(), 100.0);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, "Test".to_string(), 100.0);
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_empty_name() {
        let record = DataRecord::new(1, "".to_string(), 100.0);
        assert!(matches!(record.validate(), Err(ValidationError::EmptyName)));
    }

    #[test]
    fn test_negative_value() {
        let record = DataRecord::new(1, "Test".to_string(), -50.0);
        assert!(matches!(record.validate(), Err(ValidationError::NegativeValue)));
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0);
        record.add_metadata("category".to_string(), "premium".to_string());
        assert_eq!(record.get_metadata("category"), Some(&"premium".to_string()));
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0);
        record.transform_value(|v| v * 2.0);
        assert_eq!(record.value, 200.0);
    }

    #[test]
    fn test_json_output() {
        let mut record = DataRecord::new(42, "Sample".to_string(), 75.5);
        record.add_metadata("status".to_string(), "active".to_string());
        let json = record.to_json();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"name\":\"Sample\""));
        assert!(json.contains("\"value\":75.5"));
        assert!(json.contains("\"status\":\"active\""));
    }

    #[test]
    fn test_batch_processing() {
        let mut records = vec![
            DataRecord::new(1, "First".to_string(), 10.0),
            DataRecord::new(2, "Second".to_string(), 20.0),
        ];

        let result = process_records(&mut records);
        assert!(result.is_ok());
        let json_results = result.unwrap();
        assert_eq!(json_results.len(), 2);
        assert!(json_results[0].contains("\"value\":11"));
        assert!(json_results[1].contains("\"value\":22"));
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn process_csv_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
        } else {
            eprintln!("Skipping invalid record: {:?}", record);
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn calculate_average_value(records: &[Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
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
            value: 42.5,
            category: "A".to_string(),
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            category: "B".to_string(),
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Z".to_string() },
        ];
        
        assert_eq!(calculate_average_value(&records), Some(20.0));
        assert_eq!(calculate_average_value(&[]), None);
    }
}