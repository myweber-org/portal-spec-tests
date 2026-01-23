use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn max_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).reduce(f64::max)
    }

    pub fn min_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).reduce(f64::min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,value,category").unwrap();
        writeln!(file, "1,10.5,A").unwrap();
        writeln!(file, "2,20.3,B").unwrap();
        writeln!(file, "3,15.7,A").unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(file.path().to_str().unwrap()).unwrap();

        assert_eq!(processor.calculate_mean(), Some(15.5));
        assert_eq!(processor.filter_by_category("A").len(), 2);
        assert_eq!(processor.max_value(), Some(20.3));
        assert_eq!(processor.min_value(), Some(10.5));
    }
}use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

#[derive(Debug, PartialEq)]
struct DataPoint {
    timestamp: i64,
    value: f64,
    category: String,
}

impl DataPoint {
    fn new(timestamp: i64, value: f64, category: &str) -> Result<Self, ValidationError> {
        if timestamp < 0 {
            return Err(ValidationError {
                message: "Timestamp cannot be negative".to_string(),
            });
        }
        
        if value.is_nan() || value.is_infinite() {
            return Err(ValidationError {
                message: "Value must be a finite number".to_string(),
            });
        }
        
        if category.trim().is_empty() {
            return Err(ValidationError {
                message: "Category cannot be empty".to_string(),
            });
        }
        
        Ok(DataPoint {
            timestamp,
            value,
            category: category.to_string(),
        })
    }
}

fn process_data_points(points: &[DataPoint]) -> Vec<DataPoint> {
    points
        .iter()
        .filter(|p| p.value > 0.0)
        .map(|p| DataPoint {
            value: p.value * 1.1,
            ..p.clone()
        })
        .collect()
}

fn calculate_statistics(points: &[DataPoint]) -> (f64, f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = points.iter().map(|p| p.value).sum();
    let count = points.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = points
        .iter()
        .map(|p| (p.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_datapoint_creation() {
        let point = DataPoint::new(1234567890, 42.5, "temperature").unwrap();
        assert_eq!(point.timestamp, 1234567890);
        assert_eq!(point.value, 42.5);
        assert_eq!(point.category, "temperature");
    }

    #[test]
    fn test_invalid_timestamp() {
        let result = DataPoint::new(-1, 42.5, "temperature");
        assert!(result.is_err());
    }

    #[test]
    fn test_process_data_points() {
        let points = vec![
            DataPoint::new(1, 10.0, "A").unwrap(),
            DataPoint::new(2, -5.0, "B").unwrap(),
            DataPoint::new(3, 20.0, "C").unwrap(),
        ];
        
        let processed = process_data_points(&points);
        assert_eq!(processed.len(), 2);
        assert!(processed.iter().all(|p| p.value > 11.0));
    }

    #[test]
    fn test_calculate_statistics() {
        let points = vec![
            DataPoint::new(1, 10.0, "A").unwrap(),
            DataPoint::new(2, 20.0, "B").unwrap(),
            DataPoint::new(3, 30.0, "C").unwrap(),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&points);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value < 0.0 {
            return Err(format!("Invalid value {} for record {}", record.value, record.id).into());
        }
        
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let average = if count > 0.0 { sum / count } else { 0.0 };
    
    let max = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);
    
    let min = records.iter()
        .map(|r| r.value)
        .fold(f64::INFINITY, f64::min);

    (average, min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();

        let result = process_csv_data(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, category: "C".to_string() },
        ];

        let (avg, min, max) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
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
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.len() < 2 {
                invalid_indices.push(index);
                continue;
            }
            
            if let Some(first_field) = record.first() {
                if first_field.is_empty() {
                    invalid_indices.push(index);
                }
            }
        }
        
        invalid_indices
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
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Jane,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["John".to_string(), "30".to_string()],
            vec!["".to_string(), "25".to_string()],
            vec!["Jane".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1, 2]);
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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_fields(&self, records: &[Vec<String>], field_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();
        
        for (row_num, record) in records.iter().enumerate() {
            if field_index >= record.len() {
                return Err(format!("Row {}: Field index {} out of bounds", row_num + 1, field_index).into());
            }
            
            match record[field_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", row_num + 1, record[field_index]).into()),
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.5").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let records = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
    }

    #[test]
    fn test_numeric_validation() {
        let records = vec![
            vec!["100.5".to_string(), "text".to_string()],
            vec!["200.0".to_string(), "more".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let numeric_values = processor.validate_numeric_fields(&records, 0).unwrap();
        
        assert_eq!(numeric_values, vec![100.5, 200.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let processor = DataProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert_eq!(mean, 30.0);
        assert_eq!(variance, 200.0);
        assert_eq!(std_dev, 200.0_f64.sqrt());
    }
}