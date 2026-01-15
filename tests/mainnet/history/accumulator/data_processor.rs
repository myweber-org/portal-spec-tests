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
        Ok(Self { id, value, category })
    }

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
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
                Err(e) => eprintln!("Skipping line {}: {}", line_num + 1, e),
            }
        }

        Ok(count)
    }

    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 100.0, "A".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "A");
    }

    #[test]
    fn test_invalid_record() {
        let result = DataRecord::new(1, -10.0, "A".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_tax() {
        let record = DataRecord::new(1, 100.0, "A".to_string()).unwrap();
        assert_eq!(record.calculate_tax(0.1), 10.0);
    }

    #[test]
    fn test_load_csv() {
        let mut csv_content = "id,value,category\n".to_string();
        csv_content.push_str("1,100.0,A\n");
        csv_content.push_str("2,200.0,B\n");
        csv_content.push_str("3,150.0,A\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.total_value(), 450.0);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "A".to_string()).unwrap());
        processor.records.push(DataRecord::new(2, 200.0, "B".to_string()).unwrap());
        processor.records.push(DataRecord::new(3, 150.0, "A".to_string()).unwrap());

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);

        let avg = processor.average_value().unwrap();
        assert_eq!(avg, 150.0);
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
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: u64) -> Self {
        DataRecord {
            id,
            value,
            category,
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

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let timestamp = parts[3].parse::<u64>().unwrap_or(0);

            let record = DataRecord::new(id, value, category, timestamp);
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

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let avg = self.calculate_average().unwrap_or(0.0);
        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
        let max = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);

        Statistics {
            count,
            average: avg,
            minimum: min,
            maximum: max,
        }
    }

    pub fn export_valid_records<P: AsRef<Path>>(&self, path: P) -> Result<usize, Box<dyn Error>> {
        let mut valid_count = 0;
        let mut output = String::new();
        
        output.push_str("id,value,category,timestamp\n");
        
        for record in &self.records {
            if record.is_valid() {
                output.push_str(&format!("{},{},{},{}\n", 
                    record.id, record.value, record.category, record.timestamp));
                valid_count += 1;
            }
        }

        std::fs::write(path, output)?;
        Ok(valid_count)
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub minimum: f64,
    pub maximum: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Count: {}, Average: {:.2}, Min: {:.2}, Max: {:.2}", 
            self.count, self.average, self.minimum, self.maximum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), 1234567890);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, 42.5, "test".to_string(), 1234567890);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,alpha,1234567890").unwrap();
        writeln!(temp_file, "2,99.9,beta,1234567891").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.records.len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test".to_string(), 1));
        processor.records.push(DataRecord::new(2, 20.0, "test".to_string(), 2));
        processor.records.push(DataRecord::new(3, 30.0, "test".to_string(), 3));

        let stats = processor.get_statistics();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.average, 20.0);
        assert_eq!(stats.minimum, 10.0);
        assert_eq!(stats.maximum, 30.0);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidInput,
    TransformationFailed,
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidInput => write!(f, "Invalid input data"),
            ProcessingError::TransformationFailed => write!(f, "Data transformation failed"),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
    enabled: bool,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor {
            threshold,
            enabled: true,
        }
    }

    pub fn process_data(&self, input: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        if !self.enabled {
            return Err(ProcessingError::ValidationError("Processor disabled".to_string()));
        }

        if input.is_empty() {
            return Err(ProcessingError::InvalidInput);
        }

        let validated = self.validate_data(input)?;
        let transformed = self.transform_data(&validated)?;
        
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        let mut result = Vec::with_capacity(data.len());
        
        for &value in data {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationError(
                    format!("Invalid numeric value: {}", value)
                ));
            }
            
            if value.abs() > self.threshold {
                return Err(ProcessingError::ValidationError(
                    format!("Value {} exceeds threshold {}", value, self.threshold)
                ));
            }
            
            result.push(value);
        }
        
        Ok(result)
    }

    fn transform_data(&self, data: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        let mut transformed = Vec::with_capacity(data.len());
        
        for &value in data {
            let transformed_value = (value * 2.0).sin();
            
            if transformed_value.is_nan() {
                return Err(ProcessingError::TransformationFailed);
            }
            
            transformed.push(transformed_value);
        }
        
        Ok(transformed)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn update_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processing() {
        let processor = DataProcessor::new(100.0);
        let input = vec![1.0, 2.0, 3.0];
        
        let result = processor.process_data(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_invalid_input() {
        let processor = DataProcessor::new(100.0);
        let input = vec![];
        
        let result = processor.process_data(&input);
        assert!(matches!(result, Err(ProcessingError::InvalidInput)));
    }

    #[test]
    fn test_threshold_exceeded() {
        let processor = DataProcessor::new(10.0);
        let input = vec![5.0, 15.0, 3.0];
        
        let result = processor.process_data(&input);
        assert!(matches!(result, Err(ProcessingError::ValidationError(_))));
    }
}