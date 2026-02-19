
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(DataRecord { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
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

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(e) => eprintln!("Skipping invalid record at line {}: {}", line_num + 1, e),
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

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.calculate_total_value() / self.records.len() as f64)
        }
    }

    pub fn find_max_value_record(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test".to_string()).is_err());
        assert!(DataRecord::new(1, 5.0, "".to_string()).is_err());
    }

    #[test]
    fn test_calculate_adjusted_value() {
        let record = DataRecord::new(1, 10.0, "test".to_string()).unwrap();
        assert_eq!(record.calculate_adjusted_value(2.0), 20.0);
    }

    #[test]
    fn test_load_from_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.0,beta").unwrap();
        writeln!(temp_file, "3,invalid,gamma").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.records.len(), 2);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(
            DataRecord::new(1, 10.0, "alpha".to_string()).unwrap()
        );
        processor.records.push(
            DataRecord::new(2, 20.0, "beta".to_string()).unwrap()
        );
        processor.records.push(
            DataRecord::new(3, 30.0, "alpha".to_string()).unwrap()
        );

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
    }

    #[test]
    fn test_calculate_total_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(
            DataRecord::new(1, 10.0, "test".to_string()).unwrap()
        );
        processor.records.push(
            DataRecord::new(2, 20.0, "test".to_string()).unwrap()
        );

        assert_eq!(processor.calculate_total_value(), 30.0);
        assert_eq!(processor.get_average_value(), Some(15.0));
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.get_average_value(), None);
        assert_eq!(processor.find_max_value_record(), None);
    }
}
use csv;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
    
    fn process(&mut self) {
        self.name = self.name.to_uppercase();
        self.value = (self.value * 100.0).round() / 100.0;
    }
}

pub fn load_and_process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let mut records: Vec<Record> = Vec::new();
    
    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        if record.is_valid() {
            record.process();
            records.push(record);
        }
    }
    
    let output_file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(output_file);
    
    for record in records {
        wtr.serialize(record)?;
    }
    
    wtr.flush()?;
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            active: true,
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            active: false,
        };
        
        assert!(valid_record.is_valid());
        assert!(!invalid_record.is_valid());
    }
    
    #[test]
    fn test_record_processing() {
        let mut record = Record {
            id: 1,
            name: "test".to_string(),
            value: 12.3456,
            active: true,
        };
        
        record.process();
        
        assert_eq!(record.name, "TEST");
        assert_eq!(record.value, 12.35);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyCategory => write!(f, "Category cannot be empty"),
            DataError::TransformationError(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(DataError::EmptyCategory);
        }
        
        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<Self, DataError> {
        if multiplier <= 0.0 {
            return Err(DataError::TransformationError(
                "Multiplier must be positive".to_string()
            ));
        }
        
        let new_value = self.value * multiplier;
        
        if new_value > 1000.0 {
            return Err(DataError::TransformationError(
                format!("Transformed value {} exceeds maximum limit", new_value)
            ));
        }
        
        Ok(Self {
            id: self.id,
            value: new_value,
            category: self.category.clone(),
        })
    }
    
    pub fn normalize(&self, max_value: f64) -> Result<f64, DataError> {
        if max_value <= 0.0 {
            return Err(DataError::TransformationError(
                "Maximum value must be positive".to_string()
            ));
        }
        
        if self.value > max_value {
            return Err(DataError::TransformationError(
                format!("Value {} exceeds normalization maximum {}", self.value, max_value)
            ));
        }
        
        Ok(self.value / max_value)
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
    
    pub fn process_all(&self, multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
        let mut results = Vec::new();
        
        for record in &self.records {
            match record.transform(multiplier) {
                Ok(transformed) => results.push(transformed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.iter()
            .filter(|record| record.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 100.0, "test".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 100.0, "test".to_string());
        assert!(matches!(record, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_invalid_value() {
        let record = DataRecord::new(1, -10.0, "test".to_string());
        assert!(matches!(record, Err(DataError::InvalidValue)));
        
        let record = DataRecord::new(1, 1500.0, "test".to_string());
        assert!(matches!(record, Err(DataError::InvalidValue)));
    }
    
    #[test]
    fn test_empty_category() {
        let record = DataRecord::new(1, 100.0, "".to_string());
        assert!(matches!(record, Err(DataError::EmptyCategory)));
        
        let record = DataRecord::new(1, 100.0, "   ".to_string());
        assert!(matches!(record, Err(DataError::EmptyCategory)));
    }
    
    #[test]
    fn test_record_transformation() {
        let record = DataRecord::new(1, 100.0, "test".to_string()).unwrap();
        let transformed = record.transform(2.0).unwrap();
        assert_eq!(transformed.value, 200.0);
    }
    
    #[test]
    fn test_invalid_transformation() {
        let record = DataRecord::new(1, 600.0, "test".to_string()).unwrap();
        let result = record.transform(2.0);
        assert!(matches!(result, Err(DataError::TransformationError(_))));
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, 100.0, "A".to_string()).unwrap();
        let record2 = DataRecord::new(2, 200.0, "B".to_string()).unwrap();
        let record3 = DataRecord::new(3, 300.0, "A".to_string()).unwrap();
        
        processor.add_record(record1);
        processor.add_record(record2);
        processor.add_record(record3);
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        
        let (mean, variance, std_dev) = processor.calculate_statistics();
        assert_eq!(mean, 200.0);
        assert_eq!(variance, 6666.666666666667);
        assert_eq!(std_dev, 81.64965809277261);
    }
}