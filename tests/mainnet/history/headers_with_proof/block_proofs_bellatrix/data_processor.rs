
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut result = Vec::with_capacity(data.len());
        
        for &value in data {
            if value.is_nan() || value.is_infinite() {
                return Err(format!("Invalid numeric value detected: {}", value));
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < 2 {
            return data.to_vec();
        }

        let mean: f64 = data.iter().sum::<f64>() / data.len() as f64;
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < f64::EPSILON {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("valid", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), data.len());
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let first_result = processor.process_dataset("cached", &data).unwrap();
        let second_result = processor.process_dataset("cached", &data).unwrap();
        
        assert_eq!(first_result, second_result);
        
        let (unique_keys, total_values) = processor.cache_stats();
        assert_eq!(unique_keys, 1);
        assert_eq!(total_values, 3);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    details: String,
}

impl ValidationError {
    fn new(msg: &str) -> ValidationError {
        ValidationError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::new("ID cannot be zero"));
        }
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(ValidationError::new("Value must be between 0 and 1000"));
        }
        if self.timestamp == 0 {
            return Err(ValidationError::new("Timestamp cannot be zero"));
        }
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, ValidationError>> {
    records
        .into_iter()
        .map(|mut record| {
            record.validate()?;
            record.value = transform_value(record.value);
            Ok(record)
        })
        .collect()
}

fn transform_value(value: f64) -> f64 {
    (value * 1.05).round()
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
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 500.0,
            timestamp: 1234567890,
        };
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 500.0,
            timestamp: 1234567890,
        };
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let result = transform_value(100.0);
        assert_eq!(result, 105.0);
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 100,
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 200,
            },
            DataRecord {
                id: 3,
                value: 30.0,
                timestamp: 300,
            },
        ];
        let avg = calculate_average(&records).unwrap();
        assert_eq!(avg, 20.0);
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

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|f| !f.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        if data.is_empty() {
            return Err("No data available".into());
        }

        let mut values = Vec::new();
        for record in data {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }

            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at column {}", column_index).into()),
            }
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&data, 0).unwrap();
        
        let expected_mean = (10.5 + 15.5 + 12.0) / 3.0;
        assert!((stats.0 - expected_mean).abs() < 0.0001);
    }
}