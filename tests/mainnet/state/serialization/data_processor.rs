
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
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::UnknownCategory => write!(f, "Category not recognized"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    valid_categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(valid_categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            valid_categories,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        self.records.remove(&id)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        if !self.valid_categories.contains(&record.category) {
            return Err(DataError::UnknownCategory);
        }
        
        Ok(())
    }
}

pub fn create_sample_processor() -> DataProcessor {
    let categories = vec![
        "analytics".to_string(),
        "monitoring".to_string(),
        "reporting".to_string(),
    ];
    
    DataProcessor::new(categories)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = create_sample_processor();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "analytics".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = create_sample_processor();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "unknown".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = create_sample_processor();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 50.0,
                category: "analytics".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 100.0,
                category: "monitoring".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 150.0,
                category: "reporting".to_string(),
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.calculate_average(), 100.0);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = create_sample_processor();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Analytics 1".to_string(),
                value: 50.0,
                category: "analytics".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Analytics 2".to_string(),
                value: 75.0,
                category: "analytics".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Monitoring 1".to_string(),
                value: 100.0,
                category: "monitoring".to_string(),
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let analytics_records = processor.filter_by_category("analytics");
        assert_eq!(analytics_records.len(), 2);
    }

    #[test]
    fn test_transform_values() {
        let mut processor = create_sample_processor();
        
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 50.0,
            category: "analytics".to_string(),
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 2.0);
        
        let updated_record = processor.get_record(1).unwrap();
        assert_eq!(updated_record.value, 100.0);
    }
}