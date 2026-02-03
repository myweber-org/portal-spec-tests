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
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() && !record.iter().all(|field| field.is_empty()) {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>], expected_columns: usize) -> Result<(), String> {
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                return Err(format!(
                    "Record {} has {} columns, expected {}",
                    index + 1,
                    record.len(),
                    expected_columns
                ));
            }
        }
        Ok(())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, String> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        let mut column_data = Vec::with_capacity(records.len());
        for (row_index, record) in records.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!(
                    "Column index {} out of bounds for record {}",
                    column_index,
                    row_index + 1
                ));
            }
            column_data.push(record[column_index].clone());
        }
        Ok(column_data)
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
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records, 3).is_ok());
        assert!(processor.validate_records(&records, 2).is_err());
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1).unwrap();
        assert_eq!(column, vec!["b", "e"]);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: u64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid value field"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: u64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if !value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        if timestamp == 0 {
            return Err(DataError::InvalidTimestamp);
        }

        Ok(Self {
            id,
            value,
            timestamp,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Result<Self, DataError> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(DataError::TransformationError(
                "Invalid multiplier value".to_string(),
            ));
        }

        let new_value = self.value * multiplier;
        Ok(Self {
            id: self.id,
            value: new_value,
            timestamp: self.timestamp,
        })
    }

    pub fn normalize(&self, max_value: f64) -> Result<Self, DataError> {
        if max_value <= 0.0 || !max_value.is_finite() {
            return Err(DataError::TransformationError(
                "Invalid max value for normalization".to_string(),
            ));
        }

        let normalized_value = self.value / max_value;
        Ok(Self {
            id: self.id,
            value: normalized_value,
            timestamp: self.timestamp,
        })
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    operation: fn(&DataRecord) -> Result<DataRecord, DataError>,
) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        let transformed = operation(&record)?;
        processed.push(transformed);
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1234567890).unwrap();
        assert_eq!(record.get_id(), 1);
        assert_eq!(record.get_value(), 42.5);
        assert_eq!(record.get_timestamp(), 1234567890);
    }

    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, 42.5, 1234567890);
        assert!(matches!(result, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord::new(1, 10.0, 1000).unwrap();
        let transformed = record.transform(2.5).unwrap();
        assert_eq!(transformed.get_value(), 25.0);
    }

    #[test]
    fn test_normalize_record() {
        let record = DataRecord::new(1, 75.0, 1000).unwrap();
        let normalized = record.normalize(100.0).unwrap();
        assert_eq!(normalized.get_value(), 0.75);
    }

    #[test]
    fn test_process_multiple_records() {
        let records = vec![
            DataRecord::new(1, 10.0, 1000).unwrap(),
            DataRecord::new(2, 20.0, 2000).unwrap(),
            DataRecord::new(3, 30.0, 3000).unwrap(),
        ];

        let processed = process_records(records, |r| r.transform(2.0)).unwrap();
        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0].get_value(), 20.0);
        assert_eq!(processed[1].get_value(), 40.0);
        assert_eq!(processed[2].get_value(), 60.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn validate(&self) -> bool {
        self.valid && self.id > 0
    }
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
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();

            let record = DataRecord::new(id, value, category);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.validate())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.validate() {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string());
        assert!(valid_record.validate());

        let invalid_record = DataRecord::new(0, -5.0, "".to_string());
        assert!(!invalid_record.validate());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,15.3,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeA").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 12.9).abs() < 0.0001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("TypeA").unwrap().len(), 1);
        assert_eq!(groups.get("TypeB").unwrap().len(), 1);
    }
}