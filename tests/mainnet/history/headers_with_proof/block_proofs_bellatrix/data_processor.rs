
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed = Self::normalize_data(data);
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn normalize_data(data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if data.is_empty() {
            return stats;
        }

        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        let mean = sum / count;

        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        let std_dev = variance.sqrt();

        let sorted_data = {
            let mut sorted = data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };

        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
        } else {
            sorted_data[count as usize / 2]
        };

        stats.insert("mean".to_string(), mean);
        stats.insert("median".to_string(), median);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), *sorted_data.first().unwrap());
        stats.insert("max".to_string(), *sorted_data.last().unwrap());
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);

        stats
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = DataProcessor::normalize_data(&data);
        
        assert_eq!(normalized.len(), 5);
        let sum: f64 = normalized.iter().sum();
        assert!(sum.abs() < 1e-10);
    }

    #[test]
    fn test_process_dataset() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let result = processor.process_dataset("test", &data);
        assert!(result.is_ok());
        
        let cached_result = processor.process_dataset("test", &data);
        assert!(cached_result.is_ok());
        assert_eq!(result.unwrap(), cached_result.unwrap());
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("empty", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let stats = processor.calculate_statistics(&data);
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("median").unwrap(), &3.0);
        assert_eq!(stats.get("min").unwrap(), &1.0);
        assert_eq!(stats.get("max").unwrap(), &5.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
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
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
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
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
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
                "Empty batch provided".to_string(),
            ));
        }

        Ok(results)
    }

    pub fn filter_by_threshold(&self, records: Vec<DataRecord>, threshold: f64) -> Vec<DataRecord> {
        records
            .into_iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_value() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
        };

        assert!(matches!(
            processor.validate_record(&record),
            Err(ProcessingError::InvalidValue)
        ));
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 1,
            value: 75.0,
            timestamp: 1234567890,
        };

        assert_eq!(processor.normalize_value(&record), 0.75);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(0.0, 100.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 25.0,
                timestamp: 1000,
            },
            DataRecord {
                id: 2,
                value: 75.0,
                timestamp: 2000,
            },
        ];

        let result = processor.process_batch(records);
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0].1, 0.25);
        assert_eq!(normalized[1].1, 0.75);
    }

    #[test]
    fn test_filter_by_threshold() {
        let processor = DataProcessor::new(0.0, 100.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 30.0,
                timestamp: 1000,
            },
            DataRecord {
                id: 2,
                value: 70.0,
                timestamp: 2000,
            },
            DataRecord {
                id: 3,
                value: 50.0,
                timestamp: 3000,
            },
        ];

        let filtered = processor.filter_by_threshold(records, 50.0);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.value >= 50.0));
    }
}
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

    pub fn is_valid(&self) -> bool {
        self.valid
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
            if parts.len() < 3 {
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
        self.records.iter().filter(|r| r.is_valid()).collect()
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
            if record.is_valid() {
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

    pub fn count_valid(&self) -> usize {
        self.filter_valid().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "");
        assert!(!record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,type_a").unwrap();
        writeln!(temp_file, "2,200.0,type_b").unwrap();
        writeln!(temp_file, "3,-50.0,type_c").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.count_valid(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "a"));
        processor.records.push(DataRecord::new(2, 20.0, "b"));
        processor.records.push(DataRecord::new(3, 30.0, "c"));

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }

    #[test]
    fn test_empty_average() {
        let processor = DataProcessor::new();
        let average = processor.calculate_average();
        assert_eq!(average, None);
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

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= 0.0 && record.value <= 1000.0)
            .collect()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "1,45.5,alpha").unwrap();
        writeln!(temp_file, "2,120.3,beta").unwrap();
        writeln!(temp_file, "3,890.7,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        
        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 3);
    }
}
use std::error::Error;
use std::fs::File;
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
        let mut rdr = csv::Reader::from_reader(file);
        let mut values = Vec::new();

        for result in rdr.records() {
            let record = result?;
            if let Some(field) = record.get(0) {
                if let Ok(value) = field.parse::<f64>() {
                    values.push(value);
                }
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn calculate_std_dev(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.values.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            count: self.values.len(),
            mean: self.calculate_mean(),
            std_dev: self.calculate_std_dev(),
            min: self.values.iter().copied().reduce(f64::min),
            max: self.values.iter().copied().reduce(f64::max),
        }
    }
}

pub struct DataSummary {
    pub count: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl std::fmt::Display for DataSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Summary:")?;
        writeln!(f, "  Count: {}", self.count)?;
        
        if let Some(mean) = self.mean {
            writeln!(f, "  Mean: {:.4}", mean)?;
        }
        
        if let Some(std_dev) = self.std_dev {
            writeln!(f, "  Standard Deviation: {:.4}", std_dev)?;
        }
        
        if let Some(min) = self.min {
            writeln!(f, "  Minimum: {:.4}", min)?;
        }
        
        if let Some(max) = self.max {
            writeln!(f, "  Maximum: {:.4}", max)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let dataset = DataSet::new();
        assert_eq!(dataset.calculate_mean(), None);
        assert_eq!(dataset.calculate_std_dev(), None);
    }

    #[test]
    fn test_basic_statistics() {
        let mut dataset = DataSet::new();
        dataset.add_value(1.0);
        dataset.add_value(2.0);
        dataset.add_value(3.0);
        
        assert_eq!(dataset.calculate_mean(), Some(2.0));
        assert!(dataset.calculate_std_dev().unwrap() - 1.0 < 0.0001);
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "value")?;
        writeln!(temp_file, "10.5")?;
        writeln!(temp_file, "20.3")?;
        writeln!(temp_file, "15.7")?;
        
        let dataset = DataSet::from_csv(temp_file.path())?;
        assert_eq!(dataset.values.len(), 3);
        assert!(dataset.calculate_mean().unwrap() - 15.5 < 0.0001);
        
        Ok(())
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }
            
            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();
            
            match DataRecord::new(id, value, category) {
                Ok(record) => self.records.push(record),
                Err(e) => eprintln!("Skipping invalid record at line {}: {}", line_num + 1, e),
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());
        
        let record = DataRecord::new(2, -10.0, "test".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.total_records(), 0);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.0,alpha").unwrap();
        writeln!(temp_file, "2,200.0,beta").unwrap();
        writeln!(temp_file, "3,300.0,alpha").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(processor.total_records(), 3);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 200.0).abs() < 0.001);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
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

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();
            if category.is_empty() {
                continue;
            }

            self.records.push(DataRecord {
                id,
                value,
                category,
            });

            count += 1;
        }

        Ok(count)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.3,TypeB").unwrap();
        writeln!(temp_file, "3,15.7,TypeA").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.record_count(), 3);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.5).abs() < 0.001);

        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);

        let max_record = processor.get_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
    }
}use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
    EmptyDataset,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp must be non-negative"),
            ValidationError::EmptyDataset => write!(f, "Dataset cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.value < 0.0 || record.value > 1000.0 {
        return Err(ValidationError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(ValidationError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    if max == min {
        return 0.0;
    }
    (value - min) / (max - min)
}

pub fn process_dataset(records: &[DataRecord]) -> Result<Vec<f64>, ValidationError> {
    if records.is_empty() {
        return Err(ValidationError::EmptyDataset);
    }
    
    for record in records {
        validate_record(record)?;
    }
    
    let min_value = records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
    let max_value = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
    
    let normalized: Vec<f64> = records
        .iter()
        .map(|record| normalize_value(record.value, min_value, max_value))
        .collect();
    
    Ok(normalized)
}

pub fn calculate_statistics(records: &[DataRecord]) -> Option<(f64, f64, f64)> {
    if records.is_empty() {
        return None;
    }
    
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    Some((mean, variance, std_dev))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            value: 500.0,
            timestamp: 1234567890,
        };
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 500.0,
            timestamp: 1234567890,
        };
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_normalize_value() {
        assert_eq!(normalize_value(50.0, 0.0, 100.0), 0.5);
        assert_eq!(normalize_value(0.0, 0.0, 100.0), 0.0);
        assert_eq!(normalize_value(100.0, 0.0, 100.0), 1.0);
    }
    
    #[test]
    fn test_process_dataset() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1000 },
            DataRecord { id: 2, value: 20.0, timestamp: 2000 },
            DataRecord { id: 3, value: 30.0, timestamp: 3000 },
        ];
        
        let result = process_dataset(&records);
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized.len(), 3);
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[2], 1.0);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1000 },
            DataRecord { id: 2, value: 20.0, timestamp: 2000 },
            DataRecord { id: 3, value: 30.0, timestamp: 3000 },
        ];
        
        let stats = calculate_statistics(&records);
        assert!(stats.is_some());
        let (mean, variance, std_dev) = stats.unwrap();
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data array provided".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = values
            .iter()
            .map(|&x| x * 2.0)
            .collect();

        self.cache.insert(key.to_string(), processed.clone());

        Ok(processed)
    }

    pub fn get_cached_result(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|values| {
            let count = values.len() as f64;
            let sum: f64 = values.iter().sum();
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            (mean, variance, std_dev)
        })
    }
}

pub fn validate_input_range(values: &[f64], min: f64, max: f64) -> bool {
    values.iter().all(|&x| x >= min && x <= max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_numeric_data("invalid", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.process_numeric_data("stats", &data).unwrap();
        let stats = processor.calculate_statistics("stats");
        
        assert!(stats.is_some());
        let (mean, variance, std_dev) = stats.unwrap();
        
        assert!((mean - 6.0).abs() < 0.001);
        assert!((variance - 8.0).abs() < 0.001);
        assert!((std_dev - 2.828).abs() < 0.001);
    }
}