
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
    MissingMetadata,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
            ValidationError::MissingMetadata => write!(f, "Required metadata fields are missing"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ValidationError> {
        Self::validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
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
        
        if record.metadata.get("source").is_none() {
            return Err(ValidationError::MissingMetadata);
        }
        
        Ok(())
    }

    pub fn process_records(&mut self) -> HashMap<String, f64> {
        let mut category_totals = HashMap::new();
        
        for record in &self.records {
            let total = category_totals
                .entry(record.category.clone())
                .or_insert(0.0);
            *total += record.value;
        }
        
        category_totals
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values
            .iter()
            .map(|value| {
                let diff = mean - *value;
                diff * diff
            })
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "A".to_string(),
            metadata,
        };
        
        assert!(DataProcessor::validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "D".to_string(),
            metadata: HashMap::new(),
        };
        
        assert!(DataProcessor::validate_record(&record).is_err());
    }

    #[test]
    fn test_process_records() {
        let mut processor = DataProcessor::new();
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 50.0,
            category: "A".to_string(),
            metadata: metadata.clone(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 75.0,
            category: "A".to_string(),
            metadata,
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let totals = processor.process_records();
        assert_eq!(totals.get("A"), Some(&125.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn summary(&self) -> String {
        format!("ID: {}, Value: {:.2}, Category: {}", self.id, self.value, self.category)
    }
}

pub fn process_csv_file(file_path: &Path) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line_num == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
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

        let record = DataRecord::new(id, value, parts[2]);
        records.push(record);
    }

    Ok(records)
}

pub fn filter_valid_records(records: &[DataRecord]) -> Vec<&DataRecord> {
    records.iter().filter(|r| r.is_valid()).collect()
}

pub fn calculate_average(records: &[&DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "A");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "B");
        assert!(!record.is_valid());
    }

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,TypeA").unwrap();
        writeln!(temp_file, "2,200.3,TypeB").unwrap();
        writeln!(temp_file, "3,invalid,TypeC").unwrap();

        let records = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_filter_and_average() {
        let records = vec![
            DataRecord::new(1, 100.0, "A"),
            DataRecord::new(2, 200.0, "B"),
            DataRecord::new(3, -50.0, "C"),
        ];

        let valid_records = filter_valid_records(&records);
        assert_eq!(valid_records.len(), 2);

        let avg = calculate_average(&valid_records);
        assert_eq!(avg, Some(150.0));
    }
}