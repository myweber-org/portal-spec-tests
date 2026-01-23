
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Self {
        DataRecord { id, value, timestamp }
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::InvalidData("ID cannot be zero".to_string()));
        }
        if !self.value.is_finite() {
            return Err(ProcessingError::InvalidData("Value must be finite".to_string()));
        }
        if self.timestamp < 0 {
            return Err(ProcessingError::InvalidData("Timestamp cannot be negative".to_string()));
        }
        Ok(())
    }

    pub fn transform(&mut self, factor: f64) -> Result<(), ProcessingError> {
        if factor <= 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Transformation factor must be positive".to_string(),
            ));
        }
        self.value *= factor;
        self.timestamp += 3600;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(factor)?;
        processed.push(DataRecord::new(record.id, record.value, record.timestamp));
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, 1672531200);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 42.5, 1672531200);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_transformation() {
        let mut record = DataRecord::new(1, 10.0, 1672531200);
        assert!(record.transform(2.5).is_ok());
        assert_eq!(record.value, 25.0);
        assert_eq!(record.timestamp, 1672534800);
    }

    #[test]
    fn test_batch_processing() {
        let mut records = vec![
            DataRecord::new(1, 10.0, 1672531200),
            DataRecord::new(2, 20.0, 1672531200),
        ];
        
        let result = process_records(&mut records, 3.0);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed[0].value, 30.0);
        assert_eq!(processed[1].value, 60.0);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err(format!("Empty name for record ID {}", record.id));
    }
    if record.value < 0.0 {
        return Err(format!("Negative value for record ID {}", record.id));
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err(format!("Invalid category for record ID {}", record.id));
    }
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_data_processing() {
        let data = "id,name,value,category\n1,Test1,10.5,A\n2,Test2,20.0,B\n";
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(data.as_bytes()).unwrap();
        
        let result = process_data_file(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_invalid_category() {
        let data = "id,name,value,category\n1,Test1,10.5,Invalid\n";
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(data.as_bytes()).unwrap();
        
        let result = process_data_file(file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}