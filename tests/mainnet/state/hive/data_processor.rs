
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    config: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new(config: HashMap<String, String>) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Name cannot be empty".to_string()));
        }
        
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::ValidationError("Value must be a finite number".to_string()));
        }
        
        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        if let Some(prefix) = self.config.get("name_prefix") {
            transformed.name = format!("{}{}", prefix, transformed.name);
        }
        
        if let Some(factor_str) = self.config.get("value_multiplier") {
            if let Ok(factor) = factor_str.parse::<f64>() {
                transformed.value *= factor;
            } else {
                return Err(ProcessingError::TransformationError(
                    "Invalid multiplier in config".to_string()
                ));
            }
        }
        
        if let Some(tag_filter) = self.config.get("tag_filter") {
            transformed.tags.retain(|tag| tag.contains(tag_filter));
        }
        
        Ok(transformed)
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed.push(transformed);
        }
        
        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }
        
        let count = records.len() as f64;
        let sum: f64 = records.iter().map(|r| r.value).sum();
        let avg = sum / count;
        
        let variance: f64 = records.iter()
            .map(|r| (r.value - avg).powi(2))
            .sum::<f64>() / count;
        
        let max_value = records.iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);
        
        let min_value = records.iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        stats.insert("max".to_string(), max_value);
        stats.insert("min".to_string(), min_value);
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let config = HashMap::new();
        let processor = DataProcessor::new(config);
        
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 42.0,
            tags: vec!["tag1".to_string()],
        };
        
        assert!(processor.validate_record(&valid_record).is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: f64::NAN,
            tags: vec![],
        };
        
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_transformation() {
        let mut config = HashMap::new();
        config.insert("name_prefix".to_string(), "PREFIX_".to_string());
        config.insert("value_multiplier".to_string(), "2.0".to_string());
        
        let processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 1,
            name: "Original".to_string(),
            value: 10.0,
            tags: vec!["important".to_string(), "test".to_string()],
        };
        
        let transformed = processor.transform_record(&record).unwrap();
        
        assert_eq!(transformed.name, "PREFIX_Original");
        assert_eq!(transformed.value, 20.0);
    }

    #[test]
    fn test_statistics() {
        let config = HashMap::new();
        let processor = DataProcessor::new(config);
        
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, tags: vec![] },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, tags: vec![] },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, tags: vec![] },
        ];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&60.0));
        assert_eq!(stats.get("average"), Some(&20.0));
        assert_eq!(stats.get("min"), Some(&10.0));
        assert_eq!(stats.get("max"), Some(&30.0));
    }
}
use csv;
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

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    fn find_by_id(&self, id: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.id == id)
    }

    fn export_to_json(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &self.records)?;
        Ok(())
    }
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn process_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input.csv")?;
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    if let Some(avg) = processor.calculate_average() {
        println!("Average value: {:.2}", avg);
    }
    
    if let Some(record) = processor.find_by_id(42) {
        println!("Found record: {:?}", record);
    }
    
    for record in &processor.records {
        if validate_record(record) {
            println!("Valid record: {}", record.id);
        }
    }
    
    processor.export_to_json("output.json")?;
    
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            for field in record.iter() {
                if let Ok(value) = field.parse::<f64>() {
                    self.data.push(value);
                }
            }
        }
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn filter_outliers(&mut self, threshold: f64) {
        if let (Some(mean), Some(std_dev)) = (self.calculate_mean(), self.calculate_standard_deviation()) {
            self.data.retain(|&x| {
                let z_score = (x - mean).abs() / std_dev;
                z_score <= threshold
            });
        }
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Data points: {}, Mean: {:.2}, Std Dev: {:.2}",
            self.data.len(),
            self.calculate_mean().unwrap_or(0.0),
            self.calculate_standard_deviation().unwrap_or(0.0)
        )
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
        writeln!(temp_file, "10.5,20.3,30.7\n15.2,25.8,35.1").unwrap();
        
        assert!(processor.load_from_csv(temp_file.path()).is_ok());
        assert_eq!(processor.data.len(), 6);
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 22.93).abs() < 0.01);
        
        let original_len = processor.data.len();
        processor.filter_outliers(2.0);
        assert!(processor.data.len() <= original_len);
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
    category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Result<Self, ProcessingError> {
        if name.is_empty() {
            return Err(ProcessingError::InvalidData("Name cannot be empty".to_string()));
        }
        if value < 0.0 {
            return Err(ProcessingError::InvalidData("Value cannot be negative".to_string()));
        }
        if category.is_empty() {
            return Err(ProcessingError::InvalidData("Category cannot be empty".to_string()));
        }

        Ok(Self {
            id,
            name,
            value,
            category,
        })
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.name.len() > 100 {
            return Err(ProcessingError::ValidationError("Name too long".to_string()));
        }
        if self.value > 1_000_000.0 {
            return Err(ProcessingError::ValidationError("Value exceeds maximum".to_string()));
        }
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::TransformationError("Multiplier must be positive".to_string()));
        }
        self.value *= multiplier;
        Ok(())
    }

    pub fn get_normalized_value(&self, max_value: f64) -> f64 {
        if max_value <= 0.0 {
            return 0.0;
        }
        self.value / max_value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStatistics>,
}

#[derive(Debug, Clone)]
pub struct CategoryStatistics {
    total_value: f64,
    count: usize,
    average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        record.validate()?;
        self.records.push(record);
        self.update_statistics();
        Ok(())
    }

    pub fn process_records(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        for record in &mut self.records {
            record.transform(multiplier)?;
        }
        self.update_statistics();
        Ok(())
    }

    pub fn get_category_statistics(&self, category: &str) -> Option<&CategoryStatistics> {
        self.category_stats.get(category)
    }

    pub fn get_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    fn update_statistics(&mut self) {
        self.category_stats.clear();
        let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = category_totals
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        for (category, (total, count)) in category_totals {
            let average = if count > 0 { total / count as f64 } else { 0.0 };
            self.category_stats.insert(
                category,
                CategoryStatistics {
                    total_value: total,
                    count,
                    average_value: average,
                },
            );
        }
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
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
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, "Test".to_string(), 100.0, "CategoryA".to_string());
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_record_creation() {
        let record = DataRecord::new(1, "".to_string(), 100.0, "CategoryA".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0, "CategoryA".to_string()).unwrap();
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 200.0);
    }

    #[test]
    fn test_data_processor_statistics() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord::new(1, "Item1".to_string(), 100.0, "CategoryA".to_string()).unwrap();
        let record2 = DataRecord::new(2, "Item2".to_string(), 200.0, "CategoryA".to_string()).unwrap();
        let record3 = DataRecord::new(3, "Item3".to_string(), 300.0, "CategoryB".to_string()).unwrap();

        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        processor.add_record(record3).unwrap();

        let stats_a = processor.get_category_statistics("CategoryA").unwrap();
        assert_eq!(stats_a.total_value, 300.0);
        assert_eq!(stats_a.count, 2);
        assert_eq!(stats_a.average_value, 150.0);

        let stats_b = processor.get_category_statistics("CategoryB").unwrap();
        assert_eq!(stats_b.total_value, 300.0);
        assert_eq!(stats_b.count, 1);
        assert_eq!(stats_b.average_value, 300.0);
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

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_record_validation() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];

        assert!(processor.validate_record(&valid_record, 2));
        assert!(!processor.validate_record(&invalid_record, 2));
    }

    #[test]
    fn test_column_extraction() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 0);

        assert_eq!(column, vec!["a".to_string(), "c".to_string()]);
    }
}