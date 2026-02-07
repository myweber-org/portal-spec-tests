
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
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, &'static str> {
        if value < 0.0 {
            return Err("Value cannot be negative");
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty");
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate.max(0.0)
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let raw_record: (u32, f64, String) = result?;
        match DataRecord::new(raw_record.0, raw_record.1, raw_record.2) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Skipping invalid record: {}", e),
        }
    }

    if records.is_empty() {
        return Err("No valid records found".into());
    }

    Ok(records)
}

pub fn process_records(records: &[DataRecord]) -> (f64, f64, usize) {
    let total: f64 = records.iter().map(|r| r.value).sum();
    let avg = total / records.len() as f64;
    let valid_count = records.len();
    (total, avg, valid_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 100.0, "Electronics".to_string());
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.calculate_tax(0.1), 10.0);
    }

    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(2, -50.0, "Books".to_string()).is_err());
        assert!(DataRecord::new(3, 50.0, "".to_string()).is_err());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.0,Electronics").unwrap();
        writeln!(temp_file, "2,50.5,Books").unwrap();
        
        let records = load_csv_data(temp_file.path().to_str().unwrap());
        assert!(records.is_ok());
        let records = records.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].category, "Electronics");
    }
}