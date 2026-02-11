use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
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
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].trim();

            let record = DataRecord::new(id, value, category);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.valid)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "A");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
        assert!(record.valid);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "");
        assert!(!record.valid);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.3,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeC").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A"));
        processor.records.push(DataRecord::new(2, 20.0, "B"));
        processor.records.push(DataRecord::new(3, 30.0, "C"));

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }

    #[test]
    fn test_empty_average() {
        let processor = DataProcessor::new();
        let average = processor.calculate_average();
        assert_eq!(average, None);
    }

    #[test]
    fn test_grouping() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "Group1"));
        processor.records.push(DataRecord::new(2, 20.0, "Group1"));
        processor.records.push(DataRecord::new(3, 30.0, "Group2"));

        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("Group1").unwrap().len(), 2);
        assert_eq!(groups.get("Group2").unwrap().len(), 1);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    fn new(msg: &str) -> ProcessingError {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<DataRecord, ProcessingError> {
        if value < 0.0 {
            return Err(ProcessingError::new("Value cannot be negative"));
        }
        if timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<f64> {
    records
        .into_iter()
        .filter(|r| r.value > 10.0)
        .map(|r| r.value * 1.5)
        .collect()
}

pub fn calculate_statistics(values: &[f64]) -> (f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = values.iter().sum();
    let count = values.len() as f64;
    let mean = sum / count;

    let variance: f64 = values
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 25.5, 1625097600).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 25.5);
        assert_eq!(record.timestamp, 1625097600);
    }

    #[test]
    fn test_invalid_record_negative_value() {
        let result = DataRecord::new(1, -5.0, 1625097600);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 5.0, 1000).unwrap(),
            DataRecord::new(2, 15.0, 2000).unwrap(),
            DataRecord::new(3, 25.0, 3000).unwrap(),
        ];

        let processed = process_records(records);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0], 22.5);
        assert_eq!(processed[1], 37.5);
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (mean, variance, std_dev) = calculate_statistics(&values);
        
        assert!((mean - 5.0).abs() < 0.001);
        assert!((variance - 4.0).abs() < 0.001);
        assert!((std_dev - 2.0).abs() < 0.001);
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    EmptyCategory,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor { min_value, max_value }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::EmptyCategory);
        }

        Ok(())
    }

    pub fn normalize_value(&self, record: &DataRecord) -> f64 {
        (record.value - self.min_value) / (self.max_value - self.min_value)
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<(u64, f64)>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;
            let normalized = self.normalize_value(&record);
            results.push((record.id, normalized));
        }

        if results.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "No valid records to process".to_string(),
            ));
        }

        Ok(results)
    }

    pub fn filter_by_category(
        records: Vec<DataRecord>,
        category_filter: &str,
    ) -> Vec<DataRecord> {
        records
            .into_iter()
            .filter(|r| r.category == category_filter)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record_valid() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
            category: "test".to_string(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_invalid_value() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
            category: "test".to_string(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalize_value() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 75.0,
            timestamp: 1234567890,
            category: "test".to_string(),
        };

        let normalized = processor.normalize_value(&record);
        assert!((normalized - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1000,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 2000,
                category: "B".to_string(),
            },
            DataRecord {
                id: 3,
                value: 30.0,
                timestamp: 3000,
                category: "A".to_string(),
            },
        ];

        let filtered = DataProcessor::filter_by_category(records, "A");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
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
            
            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[2].trim().to_string();
            
            self.records.push(DataRecord {
                id,
                value,
                category,
            });
            
            count += 1;
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

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);
        
        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 10.5);
        assert_eq!(stats.1, 20.3);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut wtr = Writer::from_path(output_path)?;
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }
}

fn process_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Record with maximum value: {:?}", max_record);
    }
    
    let filtered = processor.filter_by_category("premium");
    println!("Premium records found: {}", filtered.len());
    
    processor.save_filtered_to_csv("premium", "premium_records.csv")?;
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
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
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    next_id: u32,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_record(&mut self, name: String, value: f64, tags: Vec<String>) -> Result<u32, DataError> {
        if name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }

        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }

        let id = self.next_id;
        if self.records.contains_key(&id) {
            return Err(DataError::DuplicateRecord);
        }

        let record = DataRecord {
            id,
            name,
            value,
            tags,
        };

        self.records.insert(id, record);
        self.next_id += 1;

        Ok(id)
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn update_value(&mut self, id: u32, new_value: f64) -> Result<(), DataError> {
        if new_value < 0.0 || new_value > 1000.0 {
            return Err(DataError::InvalidValue);
        }

        if let Some(record) = self.records.get_mut(&id) {
            record.value = new_value;
            Ok(())
        } else {
            Err(DataError::InvalidId)
        }
    }

    pub fn filter_by_min_value(&self, min_value: f64) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|record| record.value >= min_value)
            .collect()
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.records.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.values().map(|r| r.value).sum();
        let mean = sum / count;

        let variance: f64 = self.records.values()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        self.records.remove(&id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.next_id = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let result = processor.add_record("Test Record".to_string(), 100.0, vec!["tag1".to_string()]);
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_add_invalid_value() {
        let mut processor = DataProcessor::new();
        let result = processor.add_record("Test".to_string(), -10.0, vec![]);
        assert!(matches!(result, Err(DataError::InvalidValue)));
    }

    #[test]
    fn test_empty_name() {
        let mut processor = DataProcessor::new();
        let result = processor.add_record("".to_string(), 50.0, vec![]);
        assert!(matches!(result, Err(DataError::EmptyName)));
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_record("Record 1".to_string(), 100.0, vec![]).unwrap();
        processor.add_record("Record 2".to_string(), 200.0, vec![]).unwrap();
        processor.add_record("Record 3".to_string(), 300.0, vec![]).unwrap();

        let (mean, variance, std_dev) = processor.calculate_statistics();
        assert_eq!(mean, 200.0);
        assert_eq!(variance, 6666.666666666667);
        assert_eq!(std_dev, 81.64965809277261);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = DataProcessor::new();
        processor.add_record("Low".to_string(), 50.0, vec![]).unwrap();
        processor.add_record("Medium".to_string(), 150.0, vec![]).unwrap();
        processor.add_record("High".to_string(), 250.0, vec![]).unwrap();

        let filtered = processor.filter_by_min_value(100.0);
        assert_eq!(filtered.len(), 2);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }

    writer.flush()?;
    
    println!("Processing complete. Valid records: {}, Invalid records: {}", 
             valid_count, invalid_count);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = Record {
            id: 2,
            name: "".to_string(),
            value: -50.0,
            category: "B".to_string(),
        };
        assert!(!record.is_valid());
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut input_file = NamedTempFile::new()?;
        writeln!(input_file, "id,name,value,category")?;
        writeln!(input_file, "1,Alice,100.0,A")?;
        writeln!(input_file, "2,Bob,-50.0,B")?;
        writeln!(input_file, "3,,200.0,C")?;

        let output_file = NamedTempFile::new()?;
        
        process_csv(input_file.path().to_str().unwrap(), 
                   output_file.path().to_str().unwrap())?;
        
        Ok(())
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor;

impl DataProcessor {
    pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<DataRecord>, Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        let mut records = Vec::new();
        for result in reader.deserialize() {
            let record: DataRecord = result?;
            record.validate()?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn save_to_csv<P: AsRef<Path>>(
        records: &[DataRecord],
        path: P,
    ) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        for record in records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn filter_by_category(records: &[DataRecord], category: &str) -> Vec<DataRecord> {
        records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(records: &[DataRecord]) -> f64 {
        if records.is_empty() {
            return 0.0;
        }
        let sum: f64 = records.iter().map(|r| r.value).sum();
        sum / records.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(2, "".to_string(), -50.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let records = vec![
            DataRecord::new(1, "Item1".to_string(), 10.5, "CategoryA".to_string()),
            DataRecord::new(2, "Item2".to_string(), 20.0, "CategoryB".to_string()),
            DataRecord::new(3, "Item3".to_string(), 15.0, "CategoryA".to_string()),
        ];

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        DataProcessor::save_to_csv(&records, path)?;
        let loaded_records = DataProcessor::load_from_csv(path)?;

        assert_eq!(records.len(), loaded_records.len());

        let filtered = DataProcessor::filter_by_category(&loaded_records, "CategoryA");
        assert_eq!(filtered.len(), 2);

        let average = DataProcessor::calculate_average(&records);
        assert!((average - 15.166666666666666).abs() < 0.0001);

        Ok(())
    }
}