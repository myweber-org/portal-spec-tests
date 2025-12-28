
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    DuplicateRecord,
    ProcessingError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidName => write!(f, "Invalid record name"),
            DataError::InvalidValue => write!(f, "Invalid record value"),
            DataError::DuplicateRecord => write!(f, "Duplicate record detected"),
            DataError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    name_index: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            name_index: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }

        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        if self.name_index.contains_key(&record.name) {
            return Err(DataError::DuplicateRecord);
        }

        self.records.insert(record.id, record.clone());
        self.name_index.insert(record.name, record.id);
        Ok(())
    }

    pub fn get_record_by_id(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_record_by_name(&self, name: &str) -> Option<&DataRecord> {
        self.name_index.get(name).and_then(|id| self.records.get(id))
    }

    pub fn calculate_statistics(&self) -> Result<Statistics, DataError> {
        if self.records.is_empty() {
            return Err(DataError::ProcessingError("No records available".to_string()));
        }

        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let average = sum / count as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Ok(Statistics {
            count,
            sum,
            average,
            min,
            max,
        })
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) -> Result<(), DataError>
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            let new_value = transform_fn(record.value);
            if new_value.is_nan() || new_value.is_infinite() {
                return Err(DataError::InvalidValue);
            }
            record.value = new_value;
        }
        Ok(())
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Statistics: count={}, sum={:.2}, avg={:.2}, min={:.2}, max={:.2}",
            self.count, self.sum, self.average, self.min, self.max
        )
    }
}