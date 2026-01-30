
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    UnknownCategory,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::UnknownCategory => write!(f, "Category not recognized"),
            DataError::TransformationError(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_weights: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_weights: HashMap::from([
                ("standard".to_string(), 1.0),
                ("premium".to_string(), 1.5),
                ("economy".to_string(), 0.8),
            ]),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if !(0.0..=1000.0).contains(&record.value) {
            return Err(DataError::InvalidValue);
        }
        
        if !self.category_weights.contains_key(&record.category) {
            return Err(DataError::UnknownCategory);
        }
        
        Ok(())
    }

    pub fn calculate_weighted_values(&self) -> Result<Vec<f64>, DataError> {
        let mut results = Vec::with_capacity(self.records.len());
        
        for record in &self.records {
            let weight = self.category_weights
                .get(&record.category)
                .ok_or_else(|| DataError::TransformationError(
                    format!("Missing weight for category: {}", record.category)
                ))?;
            
            let weighted_value = record.value * weight;
            results.push(weighted_value);
        }
        
        Ok(results)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn transform_records<F>(&mut self, transform_fn: F) -> Result<(), DataError>
    where
        F: Fn(&mut DataRecord) -> Result<(), DataError>,
    {
        for record in &mut self.records {
            transform_fn(record)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_addition() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "standard".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_invalid_id() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 50.0,
            category: "standard".to_string(),
        };
        
        assert!(matches!(processor.add_record(record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_weighted_calculation() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Premium Item".to_string(),
            value: 200.0,
            category: "premium".to_string(),
        };
        
        processor.add_record(record).unwrap();
        let weighted = processor.calculate_weighted_values().unwrap();
        
        assert_eq!(weighted[0], 300.0); // 200 * 1.5
    }

    #[test]
    fn test_category_filter() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, category: "standard".to_string() },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, category: "premium".to_string() },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, category: "standard".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let standard_records = processor.filter_by_category("standard");
        assert_eq!(standard_records.len(), 2);
    }
}