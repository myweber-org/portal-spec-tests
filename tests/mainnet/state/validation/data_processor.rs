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