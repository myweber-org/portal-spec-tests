
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    let variance: f64 = records.iter().map(|r| (r.value - mean).powi(2)).sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (sum, mean, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records
        .into_iter()
        .filter(|r| r.category == category)
        .collect()
}use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn validate(&self) -> Result<(), DataError> {
        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::ValidationFailed(
                    format!("Invalid numeric value: {}", value)
                ));
            }
        }
        Ok(())
    }
    
    pub fn transform(&mut self, operation: fn(f64) -> f64) {
        for value in &mut self.values {
            *value = operation(*value);
        }
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        (sum, mean, variance.sqrt())
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<(u32, f64)>, DataError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        record.transform(|x| x.log10());
        
        let (sum, mean, _) = record.calculate_statistics();
        results.push((record.id, mean));
        
        record.add_metadata(
            "processed_sum".to_string(),
            format!("{:.4}", sum)
        );
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.values.len(), 3);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(matches!(result, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![2.0, 4.0, 6.0]).unwrap();
        let (sum, mean, std_dev) = record.calculate_statistics();
        
        assert_eq!(sum, 12.0);
        assert_eq!(mean, 4.0);
        assert!(std_dev - 1.63299 < 0.00001);
    }
}
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
    InvalidCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::InvalidCategory => write!(f, "Category must be one of: A, B, C, D"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, (f64, usize)>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.update_category_stats(&record, true);
        self.records.insert(record.id, record);
        
        Ok(())
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        if let Some(record) = self.records.remove(&id) {
            self.update_category_stats(&record, false);
            Some(record)
        } else {
            None
        }
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_average(&self, category: &str) -> Option<f64> {
        self.category_stats.get(category).map(|(sum, count)| sum / *count as f64)
    }

    pub fn transform_records<F>(&mut self, transform_fn: F) 
    where
        F: Fn(&DataRecord) -> DataRecord,
    {
        let transformed: Vec<DataRecord> = self.records
            .values()
            .map(|record| transform_fn(record))
            .collect();

        self.records.clear();
        self.category_stats.clear();

        for record in transformed {
            if self.validate_record(&record).is_ok() {
                self.update_category_stats(&record, true);
                self.records.insert(record.id, record);
            }
        }
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<&DataRecord>
    where
        F: Fn(&DataRecord) -> bool,
    {
        self.records
            .values()
            .filter(|record| predicate(record))
            .collect()
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

        let valid_categories = ["A", "B", "C", "D"];
        if !valid_categories.contains(&record.category.as_str()) {
            return Err(DataError::InvalidCategory);
        }

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord, is_add: bool) {
        let entry = self.category_stats
            .entry(record.category.clone())
            .or_insert((0.0, 0));

        if is_add {
            entry.0 += record.value;
            entry.1 += 1;
        } else {
            entry.0 -= record.value;
            entry.1 -= 1;
            
            if entry.1 == 0 {
                self.category_stats.remove(&record.category);
            }
        }
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.category_stats.clear();
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.total_records(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "X".to_string(),
        };

        assert!(processor.add_record(record).is_err());
        assert_eq!(processor.total_records(), 0);
    }

    #[test]
    fn test_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 50.0,
            category: "B".to_string(),
        };

        let record2 = DataRecord {
            id: 1,
            name: "Second".to_string(),
            value: 75.0,
            category: "C".to_string(),
        };

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
        assert_eq!(processor.total_records(), 1);
    }

    #[test]
    fn test_category_average() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 50.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 100.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 150.0, category: "A".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let avg = processor.get_category_average("A");
        assert_eq!(avg, Some(100.0));
    }

    #[test]
    fn test_transform_records() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord {
            id: 1,
            name: "Original".to_string(),
            value: 50.0,
            category: "A".to_string(),
        }).unwrap();

        processor.transform_records(|record| DataRecord {
            id: record.id,
            name: format!("{}_transformed", record.name),
            value: record.value * 2.0,
            category: record.category.clone(),
        });

        let record = processor.get_record(1).unwrap();
        assert_eq!(record.name, "Original_transformed");
        assert_eq!(record.value, 100.0);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "Low".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "High".to_string(), value: 500.0, category: "B".to_string() },
            DataRecord { id: 3, name: "Medium".to_string(), value: 250.0, category: "A".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let high_value_records = processor.filter_records(|r| r.value > 100.0);
        assert_eq!(high_value_records.len(), 2);
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.category.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.validate_records();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn get_category_summary(&self) -> Vec<(String, f64)> {
        use std::collections::HashMap;

        let mut category_totals: HashMap<String, f64> = HashMap::new();
        
        for record in &self.records {
            if record.value >= 0.0 {
                *category_totals.entry(record.category.clone()).or_insert(0.0) += record.value;
            }
        }

        let mut result: Vec<(String, f64)> = category_totals.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,15.3,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeA").unwrap();
        writeln!(temp_file, "4,20.0,").unwrap();

        let file_path = temp_file.path().to_str().unwrap();
        processor.load_from_csv(file_path).unwrap();

        assert_eq!(processor.records.len(), 4);
        
        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 2);
        
        let average = processor.calculate_average();
        assert_eq!(average, Some(12.9));
        
        let summary = processor.get_category_summary();
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0], ("TypeA".to_string(), 10.5));
        assert_eq!(summary[1], ("TypeB".to_string(), 15.3));
    }
}