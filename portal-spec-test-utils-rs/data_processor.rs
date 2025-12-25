
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= 0.0 && !record.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|record| record.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|record| record.id == target_id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,Test1,10.5,2024-01-01").unwrap();
        writeln!(temp_file, "2,Test2,20.0,2024-01-02").unwrap();
        writeln!(temp_file, "3,Test3,-5.0,2024-01-03").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.25).abs() < 0.001);
        
        let record = processor.find_by_id(2);
        assert!(record.is_some());
        assert_eq!(record.unwrap().name, "Test2");
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    fn new(msg: &str) -> ProcessingError {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::new("ID cannot be zero"));
        }
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ProcessingError::new("Value must be a finite number"));
        }
        if self.timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<f64>, ProcessingError> {
    let mut results = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let transformed = transform_value(record.value)?;
        results.push(transformed);
    }
    
    Ok(results)
}

fn transform_value(value: f64) -> Result<f64, ProcessingError> {
    if value.abs() < 1e-10 {
        return Err(ProcessingError::new("Value too close to zero"));
    }
    
    let result = (value * 100.0).ln();
    if result.is_nan() || result.is_infinite() {
        return Err(ProcessingError::new("Transformation produced invalid result"));
    }
    
    Ok(result)
}

pub fn calculate_statistics(values: &[f64]) -> Option<(f64, f64)> {
    if values.is_empty() {
        return None;
    }
    
    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;
    
    let variance: f64 = values
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    
    Some((mean, variance.sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1000,
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 2000,
            },
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = calculate_statistics(&values);
        assert!(stats.is_some());
        
        let (mean, std_dev) = stats.unwrap();
        assert!((mean - 3.0).abs() < 1e-10);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }
}