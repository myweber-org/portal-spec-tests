
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
    InvalidName,
    InvalidValue,
    MissingMetadata,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::InvalidName => write!(f, "Name cannot be empty"),
            ValidationError::InvalidValue => write!(f, "Value must be positive"),
            ValidationError::MissingMetadata => write!(f, "Required metadata field is missing"),
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
            return Err(ValidationError::InvalidName);
        }
        
        if self.value <= 0.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if self.metadata.get("source").is_none() {
            return Err(ValidationError::MissingMetadata);
        }
        
        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }

    pub fn get_normalized_value(&self, base: f64) -> f64 {
        self.value / base
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ValidationError> {
    let mut valid_records = Vec::new();
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        processed_record.transform_value(1.5);
        
        if processed_record.get_normalized_value(100.0) > 1.0 {
            processed_record.add_metadata("category".to_string(), "high".to_string());
        } else {
            processed_record.add_metadata("category".to_string(), "normal".to_string());
        }
        
        valid_records.push(processed_record);
    }
    
    Ok(valid_records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let mut record = DataRecord::new(1, "test".to_string(), 50.0);
        record.add_metadata("source".to_string(), "test_source".to_string());
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, "test".to_string(), 50.0);
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "test".to_string(), 100.0);
        record.transform_value(2.0);
        
        assert_eq!(record.value, 200.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_number + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].to_string();

        let record = DataRecord::new(id, value, category);
        if !record.is_valid() {
            return Err(format!("Invalid data at line {}", line_number + 1).into());
        }

        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "A".to_string());
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord::new(0, 42.5, "A".to_string());
        assert!(!record1.is_valid());

        let record2 = DataRecord::new(1, -1.0, "A".to_string());
        assert!(!record2.is_valid());

        let record3 = DataRecord::new(1, 42.5, "".to_string());
        assert!(!record3.is_valid());
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,42.5,CategoryA")?;
        writeln!(temp_file, "2,38.2,CategoryB")?;
        writeln!(temp_file, "# This is a comment")?;
        writeln!(temp_file, "")?;
        writeln!(temp_file, "3,55.1,CategoryC")?;

        let records = process_csv_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].category, "CategoryB");
        assert_eq!(records[2].value, 55.1);

        Ok(())
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "A".to_string()),
            DataRecord::new(2, 20.0, "B".to_string()),
            DataRecord::new(3, 30.0, "C".to_string()),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}