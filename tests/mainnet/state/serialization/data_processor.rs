
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
pub enum ProcessingError {
    InvalidValue,
    MissingField,
    CategoryNotFound,
    DuplicateId,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value must be positive"),
            ProcessingError::MissingField => write!(f, "Required field is missing"),
            ProcessingError::CategoryNotFound => write!(f, "Category does not exist"),
            ProcessingError::DuplicateId => write!(f, "Duplicate record ID found"),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(allowed_categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            categories: allowed_categories,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::DuplicateId);
        }

        if record.value <= 0.0 {
            return Err(ProcessingError::InvalidValue);
        }

        if !self.categories.contains(&record.category) {
            return Err(ProcessingError::CategoryNotFound);
        }

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }

    pub fn get_records_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.category == category)
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

    pub fn validate_all(&self) -> Vec<ProcessingError> {
        let mut errors = Vec::new();

        for record in self.records.values() {
            if record.value <= 0.0 {
                errors.push(ProcessingError::InvalidValue);
            }

            if !self.categories.contains(&record.category) {
                errors.push(ProcessingError::CategoryNotFound);
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let categories = vec!["A".to_string(), "B".to_string()];
        let mut processor = DataProcessor::new(categories);
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.calculate_total(), 100.0);
    }

    #[test]
    fn test_duplicate_id() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };
        let record2 = DataRecord {
            id: 1,
            name: "Second".to_string(),
            value: 75.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record1).is_ok());
        assert!(matches!(
            processor.add_record(record2),
            Err(ProcessingError::DuplicateId)
        ));
    }

    #[test]
    fn test_transform_values() {
        let categories = vec!["X".to_string()];
        let mut processor = DataProcessor::new(categories);
        let record = DataRecord {
            id: 1,
            name: "Data".to_string(),
            value: 10.0,
            category: "X".to_string(),
        };

        processor.add_record(record).unwrap();
        processor.transform_values(|v| v * 2.0);

        assert_eq!(processor.calculate_total(), 20.0);
    }
}