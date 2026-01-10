
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: u64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    RecordNotFound,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid data value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::RecordNotFound => write!(f, "Record not found"),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: u64) -> Result<Self, ProcessingError> {
        if value < 0.0 || value > 1000.0 {
            return Err(ProcessingError::InvalidValue);
        }
        
        if timestamp == 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }
        
        Ok(Self {
            id,
            value,
            timestamp,
        })
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::InvalidValue);
        }
        
        self.value *= multiplier;
        Ok(())
    }
    
    pub fn validate(&self) -> bool {
        self.value >= 0.0 && self.value <= 1000.0 && self.timestamp > 0
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
    
    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }
    
    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.id == id)
    }
    
    pub fn process_records(&mut self) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::new();
        
        for record in &mut self.records {
            if !record.validate() {
                return Err(ProcessingError::InvalidValue);
            }
            
            record.transform(1.5)?;
            results.push(record.value);
        }
        
        Ok(results)
    }
    
    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value > threshold)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 100.0, 1234567890);
        assert!(record.is_ok());
        
        let invalid_record = DataRecord::new(2, -10.0, 1234567890);
        assert!(invalid_record.is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 100.0, 1234567890).unwrap();
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 200.0);
    }
    
    #[test]
    fn test_processor_functionality() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, 50.0, 1234567890).unwrap();
        processor.add_record(record);
        
        let result = processor.process_records();
        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0], 75.0);
    }
}use std::error::Error;
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 && self.has_header {
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
            .map(|(&sum, &count)| {
                if count > 0 {
                    sum / count as f64
                } else {
                    0.0
                }
            })
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["x".to_string(), "".to_string(), "z".to_string()],
            vec!["".to_string(), "".to_string(), "".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);

        assert_eq!(invalid, vec![1, 2]);
    }

    #[test]
    fn test_calculate_averages() {
        let records = vec![
            vec!["10".to_string(), "20".to_string()],
            vec!["20".to_string(), "30".to_string()],
            vec!["30".to_string(), "40".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let averages = processor.calculate_column_averages(&records).unwrap();

        assert_eq!(averages, vec![20.0, 30.0]);
    }
}