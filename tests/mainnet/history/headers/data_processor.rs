
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str, timestamp: u64) -> Self {
        Self {
            id,
            value,
            category: category.to_string(),
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
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

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();
            let timestamp = match parts[3].parse::<u64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, &category, timestamp);
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

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test", 1234567890);
        assert!(valid_record.is_valid());

        let invalid_id = DataRecord::new(0, 42.5, "test", 1234567890);
        assert!(!invalid_id.is_valid());

        let invalid_value = DataRecord::new(1, f64::NAN, "test", 1234567890);
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(1, 42.5, "", 1234567890);
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_load_from_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,category_a,1234567890").unwrap();
        writeln!(temp_file, "2,invalid,category_b,1234567891").unwrap();
        writeln!(temp_file, "3,78.9,category_a,1234567892").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.get_records().len(), 2);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 42.5, "category_a", 1234567890));
        processor.records.push(DataRecord::new(2, 78.9, "category_b", 1234567891));
        processor.records.push(DataRecord::new(3, 15.3, "category_a", 1234567892));

        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        let filtered = processor.filter_by_category("category_b");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        assert!(processor.calculate_average().is_none());

        processor.records.push(DataRecord::new(1, 10.0, "test", 1234567890));
        processor.records.push(DataRecord::new(2, 20.0, "test", 1234567891));
        processor.records.push(DataRecord::new(3, 30.0, "test", 1234567892));

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert_eq!(average.unwrap(), 20.0);
    }
}use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidCategory,
    TransformationFailed,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value must be positive"),
            ProcessingError::InvalidCategory => write!(f, "Category cannot be empty"),
            ProcessingError::TransformationFailed => write!(f, "Data transformation failed"),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, ProcessingError> {
        if value <= 0.0 {
            return Err(ProcessingError::InvalidValue);
        }
        if category.trim().is_empty() {
            return Err(ProcessingError::InvalidCategory);
        }

        Ok(DataRecord {
            id,
            value,
            category: category.to_string(),
        })
    }

    pub fn transform(&self, multiplier: f64) -> Result<f64, ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::TransformationFailed);
        }
        Ok(self.value * multiplier)
    }

    pub fn normalize(&self, max_value: f64) -> f64 {
        if max_value > 0.0 {
            self.value / max_value
        } else {
            0.0
        }
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<f64, ProcessingError>> {
    records
        .iter()
        .map(|record| record.transform(2.5))
        .collect()
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "analytics").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "analytics");
    }

    #[test]
    fn test_invalid_value() {
        let result = DataRecord::new(2, -5.0, "test");
        assert!(matches!(result, Err(ProcessingError::InvalidValue)));
    }

    #[test]
    fn test_transformation() {
        let record = DataRecord::new(3, 10.0, "data").unwrap();
        let transformed = record.transform(3.0).unwrap();
        assert_eq!(transformed, 30.0);
    }

    #[test]
    fn test_normalize() {
        let record = DataRecord::new(4, 75.0, "metrics").unwrap();
        let normalized = record.normalize(100.0);
        assert_eq!(normalized, 0.75);
    }
}