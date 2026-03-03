
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
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,-5.0,false").unwrap();
        writeln!(temp_file, "3,Test3,15.0,true").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.0, 25.5);
        assert_eq!(stats.1, 12.75);
        assert_eq!(stats.2, 2);
    }

    #[test]
    fn test_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 0);
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.0, 0.0);
        assert_eq!(stats.1, 0.0);
        assert_eq!(stats.2, 0);
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
    pub timestamp: String,
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

            let record = DataRecord {
                id,
                value,
                category: parts[2].to_string(),
                timestamp: parts[3].to_string(),
            };

            self.records.push(record);
            count += 1;
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

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
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
    fn test_data_processor_creation() {
        let processor = DataProcessor::new();
        assert_eq!(processor.get_records().len(), 0);
    }

    #[test]
    fn test_load_from_csv() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,45.5,electronics,2023-01-01").unwrap();
        writeln!(temp_file, "2,78.9,furniture,2023-01-02").unwrap();
        writeln!(temp_file, "3,invalid,furniture,2023-01-03").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.get_records().len(), 2);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            value: 45.5,
            category: "electronics".to_string(),
            timestamp: "2023-01-01".to_string(),
        });
        processor.records.push(DataRecord {
            id: 2,
            value: 78.9,
            category: "furniture".to_string(),
            timestamp: "2023-01-02".to_string(),
        });

        let filtered = processor.filter_by_category("electronics");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.calculate_average(), None);

        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "test".to_string(),
            timestamp: "2023-01-01".to_string(),
        });
        processor.records.push(DataRecord {
            id: 2,
            value: 20.0,
            category: "test".to_string(),
            timestamp: "2023-01-02".to_string(),
        });

        assert_eq!(processor.calculate_average(), Some(15.0));
    }

    #[test]
    fn test_find_max_value() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "test".to_string(),
            timestamp: "2023-01-01".to_string(),
        });
        processor.records.push(DataRecord {
            id: 2,
            value: 30.0,
            category: "test".to_string(),
            timestamp: "2023-01-02".to_string(),
        });
        processor.records.push(DataRecord {
            id: 3,
            value: 20.0,
            category: "test".to_string(),
            timestamp: "2023-01-03".to_string(),
        });

        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
        assert_eq!(max_record.unwrap().value, 30.0);
    }

    #[test]
    fn test_clear() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "test".to_string(),
            timestamp: "2023-01-01".to_string(),
        });

        assert_eq!(processor.get_records().len(), 1);
        processor.clear();
        assert_eq!(processor.get_records().len(), 0);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.value < 0.0 || record.value > 1000.0 {
        return Err(ValidationError::InvalidValue);
    }
    
    if record.category.trim().is_empty() {
        return Err(ValidationError::EmptyCategory);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord, multiplier: f64) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * multiplier,
        category: record.category.to_uppercase(),
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, ValidationError>> {
    records
        .into_iter()
        .map(|record| {
            validate_record(&record)?;
            Ok(transform_record(&record, 1.5))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            value: 100.0,
            category: "test".to_string(),
        };
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 100.0,
            category: "test".to_string(),
        };
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 100.0,
            category: "test".to_string(),
        };
        let transformed = transform_record(&record, 2.0);
        assert_eq!(transformed.value, 200.0);
        assert_eq!(transformed.category, "TEST");
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 100.0,
                category: "a".to_string(),
            },
            DataRecord {
                id: 0,
                value: 200.0,
                category: "b".to_string(),
            },
        ];
        
        let results = process_records(records);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }
}