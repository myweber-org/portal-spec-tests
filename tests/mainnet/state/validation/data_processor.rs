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

    pub fn process_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        filter_func: Option<Box<dyn Fn(&[String]) -> bool>>,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        let mut result = Vec::new();

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if let Some(ref filter) = filter_func {
                if filter(&fields) {
                    result.push(fields);
                }
            } else {
                result.push(fields);
            }
        }

        Ok(result)
    }

    pub fn filter_numeric_greater_than(
        &self,
        data: &[Vec<String>],
        column_index: usize,
        threshold: f64,
    ) -> Vec<Vec<String>> {
        data.iter()
            .filter(|row| {
                if let Some(value) = row.get(column_index) {
                    if let Ok(num) = value.parse::<f64>() {
                        return num > threshold;
                    }
                }
                false
            })
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
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,85.5").unwrap();
        writeln!(temp_file, "Bob,30,92.0").unwrap();
        writeln!(temp_file, "Charlie,22,78.5").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path(), None).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Alice", "25", "85.5"]);
    }

    #[test]
    fn test_filter_numeric() {
        let data = vec![
            vec!["A".to_string(), "10.5".to_string()],
            vec!["B".to_string(), "5.2".to_string()],
            vec!["C".to_string(), "15.8".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let filtered = processor.filter_numeric_greater_than(&data, 1, 10.0);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|row| row[0] == "A"));
        assert!(filtered.iter().any(|row| row[0] == "C"));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    ValidationError(String),
    #[error("Transformation failed: {0}")]
    TransformationError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            metadata: None,
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(DataError::ValidationError(
                "Values map cannot be empty".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationError(
                    "Key cannot be empty or whitespace".to_string(),
                ));
            }
            
            if !value.is_finite() {
                return Err(DataError::ValidationError(
                    format!("Value for key '{}' must be finite", key),
                ));
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if multiplier == 0.0 {
            return Err(DataError::TransformationError(
                "Multiplier cannot be zero".to_string(),
            ));
        }

        for value in self.values.values_mut() {
            *value *= multiplier;
            
            if !value.is_finite() {
                return Err(DataError::TransformationError(
                    "Transformation resulted in non-finite value".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn to_json(&self) -> Result<String, DataError> {
        serde_json::to_string_pretty(self).map_err(DataError::from)
    }

    pub fn from_json(json_str: &str) -> Result<Self, DataError> {
        serde_json::from_str(json_str).map_err(DataError::from)
    }
}

pub fn process_records(
    records: &mut [DataRecord],
    multiplier: f64,
) -> Result<Vec<String>, DataError> {
    let mut results = Vec::with_capacity(records.len());

    for record in records {
        record.validate()?;
        record.transform(multiplier)?;
        
        let json_output = record.to_json()?;
        results.push(json_output);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let mut record = DataRecord::new(1, 1625097600);
        record.values.insert("temperature".to_string(), 25.5);
        record.values.insert("humidity".to_string(), 60.0);

        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, 1625097600);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_transform() {
        let mut record = DataRecord::new(1, 1625097600);
        record.values.insert("value".to_string(), 10.0);

        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("value"), Some(&20.0));
    }

    #[test]
    fn test_serialization() {
        let mut record = DataRecord::new(42, 1625097600);
        record.values.insert("metric".to_string(), 99.9);
        
        let json = record.to_json();
        assert!(json.is_ok());
        
        let parsed = DataRecord::from_json(&json.unwrap());
        assert!(parsed.is_ok());
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

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if self.has_header && line_number == 0 {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();

        for (index, record) in records.iter().enumerate() {
            if record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }

        invalid_indices
    }

    pub fn calculate_column_averages(&self, records: &[Vec<String>]) -> Result<Vec<f64>, Box<dyn Error>> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        let column_count = records[0].len();
        let mut sums = vec![0.0; column_count];
        let mut counts = vec![0; column_count];

        for record in records {
            for (i, field) in record.iter().enumerate() {
                if let Ok(value) = field.parse::<f64>() {
                    sums[i] += value;
                    counts[i] += 1;
                }
            }
        }

        let averages: Vec<f64> = sums
            .iter()
            .zip(counts.iter())
            .map(|(&sum, &count)| if count > 0 { sum / count as f64 } else { 0.0 })
            .collect();

        Ok(averages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "col1,col2,col3").unwrap();
        writeln!(temp_file, "1,2,3").unwrap();
        writeln!(temp_file, "4,5,6").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "2", "3"]);
    }

    #[test]
    fn test_validate_records() {
        let processor = DataProcessor::new(',', false);
        let records = vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ];

        let invalid_indices = processor.validate_records(&records);
        assert_eq!(invalid_indices, vec![1]);
    }

    #[test]
    fn test_calculate_column_averages() {
        let processor = DataProcessor::new(',', false);
        let records = vec![
            vec!["1.0".to_string(), "2.0".to_string()],
            vec!["3.0".to_string(), "4.0".to_string()],
            vec!["5.0".to_string(), "6.0".to_string()],
        ];

        let averages = processor.calculate_column_averages(&records).unwrap();
        assert_eq!(averages, vec![3.0, 4.0]);
    }
}