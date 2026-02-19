
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationError("Values cannot be empty".to_string()));
        }

        for (i, &value) in self.values.iter().enumerate() {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationError(
                    format!("Invalid value at position {}: {}", i, value)
                ));
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) -> Result<(), ProcessingError> {
        self.validate()?;

        let sum: f64 = self.values.iter().sum();
        if sum == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Cannot normalize zero vector".to_string()
            ));
        }

        for value in self.values.iter_mut() {
            *value /= sum;
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> Result<HashMap<String, f64>, ProcessingError> {
        self.validate()?;

        let mut stats = HashMap::new();
        
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;

        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());

        Ok(stats)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, ProcessingError> {
    let mut results = Vec::new();

    for record in records.iter_mut() {
        record.normalize()?;
        let stats = record.calculate_statistics()?;
        results.push(stats);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.normalize().is_ok());
        
        let sum: f64 = record.values.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_statistics() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0]);
        let stats = record.calculate_statistics().unwrap();
        
        assert_eq!(stats["count"], 4.0);
        assert_eq!(stats["sum"], 10.0);
        assert_eq!(stats["mean"], 2.5);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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

            let category = parts[2].to_string();
            let valid = parts[3].parse::<bool>().unwrap_or(false);

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_creation() {
        let processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
    }

    #[test]
    fn test_load_from_csv() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,category_a,true").unwrap();
        writeln!(temp_file, "2,20.3,category_b,false").unwrap();
        writeln!(temp_file, "3,15.7,category_a,true").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_filter_valid() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(DataRecord {
            id: 1,
            value: 10.5,
            category: "test".to_string(),
            valid: true,
        });
        
        processor.records.push(DataRecord {
            id: 2,
            value: 20.3,
            category: "test".to_string(),
            valid: false,
        });
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 1);
        assert_eq!(valid_records[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "test".to_string(),
            valid: true,
        });
        
        processor.records.push(DataRecord {
            id: 2,
            value: 20.0,
            category: "test".to_string(),
            valid: true,
        });
        
        processor.records.push(DataRecord {
            id: 3,
            value: 30.0,
            category: "test".to_string(),
            valid: false,
        });
        
        let average = processor.calculate_average();
        assert_eq!(average, Some(15.0));
    }

    #[test]
    fn test_group_by_category() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(DataRecord {
            id: 1,
            value: 10.5,
            category: "category_a".to_string(),
            valid: true,
        });
        
        processor.records.push(DataRecord {
            id: 2,
            value: 20.3,
            category: "category_b".to_string(),
            valid: true,
        });
        
        processor.records.push(DataRecord {
            id: 3,
            value: 15.7,
            category: "category_a".to_string(),
            valid: true,
        });
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("category_a").unwrap().len(), 2);
        assert_eq!(groups.get("category_b").unwrap().len(), 1);
    }
}