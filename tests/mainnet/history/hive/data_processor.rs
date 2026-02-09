
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, values: Vec<f64>) -> Self {
        Self {
            id,
            timestamp,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".into());
        }
        
        if self.values.is_empty() {
            return Err("Values vector cannot be empty".into());
        }
        
        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }
        
        Ok(())
    }

    pub fn transform_values<F>(&mut self, transformer: F) 
    where
        F: Fn(f64) -> f64,
    {
        self.values = self.values.iter().map(|&v| transformer(v)).collect();
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        processed_record.transform_values(|v| v * 2.0);
        processed_record.add_metadata("processed".to_string(), "true".to_string());
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let all_values: Vec<f64> = records.iter()
        .flat_map(|r| r.values.iter().copied())
        .collect();
    
    let sum: f64 = all_values.iter().sum();
    let count = all_values.len() as f64;
    
    if count > 0.0 {
        let mean = sum / count;
        let variance: f64 = all_values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1234567890, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 1234567890, vec![1.0, 2.0, 3.0]);
        record.transform_values(|v| v * 2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 1000, vec![1.0, 2.0]),
            DataRecord::new(2, 2000, vec![3.0, 4.0]),
        ];
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.get("mean").unwrap(), &2.5);
        assert_eq!(stats.get("total_records").unwrap(), &2.0);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value < 0.0 {
            return Err("Negative value found in data".into());
        }
        
        if !is_valid_category(&record.category) {
            return Err(format!("Invalid category: {}", record.category).into());
        }
        
        records.push(record);
    }

    Ok(records)
}

fn is_valid_category(category: &str) -> bool {
    let valid_categories = ["A", "B", "C", "D"];
    valid_categories.contains(&category)
}

pub fn calculate_total(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use csv::{Reader, Writer};
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

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records
            .iter()
            .map(|record| record.value)
            .sum()
    }

    pub fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);
        
        for record in &self.records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, id: u32, name: String, value: f64, active: bool) {
        self.records.push(Record {
            id,
            name,
            value,
            active,
        });
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let total = self.calculate_total();
        let average = if count > 0 {
            total / count as f64
        } else {
            0.0
        };
        
        (count, total, average)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(1, "Test1".to_string(), 10.5, true);
        processor.add_record(2, "Test2".to_string(), 20.0, false);
        processor.add_record(3, "Test3".to_string(), 30.5, true);
        
        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 2);
        
        let total = processor.calculate_total();
        assert_eq!(total, 61.0);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        assert_eq!(stats.1, 61.0);
        assert_eq!(stats.2, 61.0 / 3.0);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(1, "Alpha".to_string(), 100.0, true);
        processor.add_record(2, "Beta".to_string(), 200.0, false);
        
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();
        
        processor.save_to_csv(temp_path)?;
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(temp_path)?;
        
        assert_eq!(new_processor.records.len(), 2);
        Ok(())
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
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
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let active = parts[3].parse::<bool>()?;

            self.records.push(Record {
                id,
                name,
                value,
                active,
            });
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records.iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter()
            .map(|record| record.value)
            .sum();
        
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter()
            .find(|record| record.id == target_id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,20.0,false").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.record_count(), 2);
    }

    #[test]
    fn test_filter_active() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "Active".to_string(),
            value: 10.0,
            active: true,
        });
        processor.records.push(Record {
            id: 2,
            name: "Inactive".to_string(),
            value: 20.0,
            active: false,
        });

        let active = processor.filter_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "Test1".to_string(),
            value: 10.0,
            active: true,
        });
        processor.records.push(Record {
            id: 2,
            name: "Test2".to_string(),
            value: 20.0,
            active: true,
        });

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(15.0));
    }

    #[test]
    fn test_find_by_id() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 42,
            name: "Target".to_string(),
            value: 99.9,
            active: true,
        });

        let found = processor.find_by_id(42);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Target");
        
        let not_found = processor.find_by_id(999);
        assert!(not_found.is_none());
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::DuplicateTag => write!(f, "Tags contain duplicates"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Self {
        DataRecord {
            id,
            name,
            value,
            tags,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &self.tags {
            if !seen_tags.insert(tag) {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ValidationError> {
        record.validate()?;
        
        if self.records.contains_key(&record.id) {
            return Err(ValidationError::InvalidId);
        }
        
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn process_records(&mut self, multiplier: f64) {
        for record in self.records.values_mut() {
            record.transform(multiplier);
        }
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let count = self.records.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.values().map(|r| r.value).sum();
        let avg = sum / count;
        
        let variance: f64 = self.records.values()
            .map(|r| (r.value - avg).powi(2))
            .sum::<f64>() / count;
        
        (sum, avg, variance.sqrt())
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(
            1,
            "test".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()],
        );
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "test".to_string(),
            100.0,
            vec!["tag1".to_string()],
        );
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_duplicate_tags() {
        let record = DataRecord::new(
            1,
            "test".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag1".to_string()],
        );
        assert!(matches!(record.validate(), Err(ValidationError::DuplicateTag)));
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(
            1,
            "record1".to_string(),
            10.0,
            vec!["important".to_string()],
        );
        
        let record2 = DataRecord::new(
            2,
            "record2".to_string(),
            20.0,
            vec!["important".to_string(), "urgent".to_string()],
        );
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        processor.process_records(2.0);
        
        let (sum, avg, _) = processor.get_statistics();
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 30.0);
        
        let important_records = processor.filter_by_tag("important");
        assert_eq!(important_records.len(), 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                values.push(value);
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn variance(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.mean().unwrap();
        let sum_sq_diff: f64 = self.values.iter().map(|&x| (x - mean).powi(2)).sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn min(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let ds = DataSet::new();
        assert_eq!(ds.count(), 0);
        assert!(ds.mean().is_none());
        assert!(ds.variance().is_none());
    }

    #[test]
    fn test_basic_statistics() {
        let mut ds = DataSet::new();
        ds.add_value(1.0);
        ds.add_value(2.0);
        ds.add_value(3.0);
        ds.add_value(4.0);
        ds.add_value(5.0);

        assert_eq!(ds.count(), 5);
        assert_eq!(ds.mean(), Some(3.0));
        assert_eq!(ds.variance(), Some(2.5));
        assert_eq!(ds.std_dev(), Some(2.5_f64.sqrt()));
        assert_eq!(ds.min(), Some(1.0));
        assert_eq!(ds.max(), Some(5.0));
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "10.5")?;
        writeln!(temp_file, "20.3")?;
        writeln!(temp_file, "15.7")?;
        writeln!(temp_file, "invalid")?;
        writeln!(temp_file, "25.1")?;

        let ds = DataSet::from_csv(temp_file.path())?;
        assert_eq!(ds.count(), 4);
        assert!((ds.mean().unwrap() - 17.9).abs() < 0.001);
        Ok(())
    }
}