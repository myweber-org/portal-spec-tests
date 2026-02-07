
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            for value in line.split(',') {
                if let Ok(num) = value.trim().parse::<f64>() {
                    self.data.push(num);
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

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            count: self.data.len(),
            mean: self.calculate_mean(),
            std_dev: self.calculate_standard_deviation(),
            min: self.data.iter().copied().reduce(f64::min),
            max: self.data.iter().copied().reduce(f64::max),
        }
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&x| x >= threshold)
            .copied()
            .collect()
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
        writeln!(f, "  Mean: {:.4}", self.mean.unwrap_or(f64::NAN))?;
        writeln!(f, "  Std Dev: {:.4}", self.std_dev.unwrap_or(f64::NAN))?;
        writeln!(f, "  Min: {:.4}", self.min.unwrap_or(f64::NAN))?;
        write!(f, "  Max: {:.4}", self.max.unwrap_or(f64::NAN))
    }
}
use std::error::Error;
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

    pub fn process_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        filter_predicate: Option<Box<dyn Fn(&[String]) -> bool>>,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        let mut records = Vec::new();

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if let Some(ref predicate) = filter_predicate {
                if predicate(&fields) {
                    records.push(fields);
                }
            } else {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn calculate_column_average(
        &self,
        data: &[Vec<String>],
        column_index: usize,
    ) -> Result<f64, Box<dyn Error>> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in data {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Err("No valid numeric data found in specified column".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path(), None).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_process_file_with_filter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let processor = DataProcessor::new(',', true);
        let filter = Box::new(|fields: &[String]| {
            if let Ok(age) = fields[1].parse::<i32>() {
                age > 30
            } else {
                false
            }
        });

        let result = processor
            .process_file(temp_file.path(), Some(filter))
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["Charlie", "35", "60000"]);
    }

    #[test]
    fn test_calculate_column_average() {
        let data = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let average = processor.calculate_column_average(&data, 0).unwrap();

        assert!((average - 12.666666).abs() < 0.0001);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: String) -> Self {
        Self {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.timestamp.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
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
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, min_value: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= min_value)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,item1,10.5,2024-01-01").unwrap();
        writeln!(temp_file, "2,item2,20.0,2024-01-02").unwrap();
        writeln!(temp_file, "3,item3,5.0,2024-01-03").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_record_count(), 3);

        let filtered = processor.filter_by_value(10.0);
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average().unwrap();
        assert!((average - 11.83333).abs() < 0.0001);

        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if let Some(value_str) = parts.get(0) {
                if let Ok(value) = value_str.parse::<f64>() {
                    self.data.push(value);
                }
            }
            
            if let Some(category) = parts.get(1) {
                *self.frequency_map.entry(category.to_string()).or_insert(0) += 1;
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

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
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
        writeln!(temp_file, "10.5,category_a").unwrap();
        writeln!(temp_file, "20.3,category_b").unwrap();
        writeln!(temp_file, "15.7,category_a").unwrap();
        writeln!(temp_file, "25.1,category_c").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(17.9));
        
        let median = processor.calculate_median();
        assert!(median.is_some());
        
        let distribution = processor.get_frequency_distribution();
        assert_eq!(distribution.get("category_a"), Some(&2));
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 3);
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            let _ = lines.next();
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

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn filter_valid_records(&self, records: Vec<Vec<String>>) -> Vec<Vec<String>> {
        records
            .into_iter()
            .filter(|record| self.validate_record(record))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_record(&["test".to_string(), "data".to_string()]));
        assert!(!processor.validate_record(&[]));
        assert!(!processor.validate_record(&["".to_string(), "valid".to_string()]));
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
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() && !record.iter().all(|field| field.is_empty()) {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.len() < 2 {
                invalid_indices.push(index);
                continue;
            }
            
            let numeric_fields = record.iter().skip(1).filter(|s| {
                s.parse::<f64>().is_ok()
            }).count();
            
            if numeric_fields < record.len() - 1 {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        if column_index == 0 {
            return None;
        }

        let values: Vec<f64> = records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,value1,value2").unwrap();
        writeln!(temp_file, "item1,10.5,20.3").unwrap();
        writeln!(temp_file, "item2,15.2,25.7").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["item1", "10.5", "20.3"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["valid".to_string(), "10.5".to_string(), "20.3".to_string()],
            vec!["invalid".to_string(), "abc".to_string()],
            vec!["partial".to_string(), "15.2".to_string(), "def".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1, 2]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["item1".to_string(), "10.0".to_string(), "20.0".to_string()],
            vec!["item2".to_string(), "20.0".to_string(), "30.0".to_string()],
            vec!["item3".to_string(), "30.0".to_string(), "40.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 1);
        
        assert!(stats.is_some());
        let (mean, variance, std_dev) = stats.unwrap();
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line_content.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].to_string();

        let record = DataRecord { id, value, category };
        if record.is_valid() {
            records.push(record);
        } else {
            eprintln!("Warning: Invalid record at line {}", line_number);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            category: "test".to_string(),
        };
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord {
            id: 0,
            value: 42.5,
            category: "test".to_string(),
        };
        assert!(!record1.is_valid());

        let record2 = DataRecord {
            id: 1,
            value: f64::NAN,
            category: "test".to_string(),
        };
        assert!(!record2.is_valid());
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,42.5,category_a")?;
        writeln!(temp_file, "2,37.8,category_b")?;
        writeln!(temp_file, "# This is a comment")?;
        writeln!(temp_file, "")?;

        let records = process_csv_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].category, "category_b");
        Ok(())
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, category: "a".to_string() },
            DataRecord { id: 2, value: 20.0, category: "b".to_string() },
            DataRecord { id: 3, value: 30.0, category: "c".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
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
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub total_value: f64,
    pub record_count: usize,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ValidationError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let valid_categories = ["A", "B", "C"];
        if !valid_categories.contains(&record.category.as_str()) {
            return Err(ValidationError::InvalidCategory);
        }
        
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                total_value: 0.0,
                record_count: 0,
                average_value: 0.0,
            });
        
        stats.total_value += record.value;
        stats.record_count += 1;
        stats.average_value = stats.total_value / stats.record_count as f64;
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn filter_by_value_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_record_count(&self) -> usize {
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
    fn test_valid_record_addition() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_record_count(), 1);
    }

    #[test]
    fn test_invalid_record_rejection() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -50.0,
            category: "D".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
        assert_eq!(processor.get_record_count(), 0);
    }

    #[test]
    fn test_category_stats_calculation() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "Record 1".to_string(), value: 100.0, category: "A".to_string() },
            DataRecord { id: 2, name: "Record 2".to_string(), value: 200.0, category: "A".to_string() },
            DataRecord { id: 3, name: "Record 3".to_string(), value: 150.0, category: "B".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let stats_a = processor.get_category_stats("A").unwrap();
        assert_eq!(stats_a.record_count, 2);
        assert_eq!(stats_a.total_value, 300.0);
        assert_eq!(stats_a.average_value, 150.0);
        
        let stats_b = processor.get_category_stats("B").unwrap();
        assert_eq!(stats_b.record_count, 1);
        assert_eq!(stats_b.total_value, 150.0);
        assert_eq!(stats_b.average_value, 150.0);
    }

    #[test]
    fn test_value_filtering() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "Low".to_string(), value: 50.0, category: "A".to_string() },
            DataRecord { id: 2, name: "Medium".to_string(), value: 150.0, category: "A".to_string() },
            DataRecord { id: 3, name: "High".to_string(), value: 250.0, category: "B".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let filtered = processor.filter_by_value_threshold(100.0);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.value >= 100.0));
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn add_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, data: &str, validator_name: &str) -> Result<(), String> {
        match self.validators.get(validator_name) {
            Some(validator) => {
                if validator(data) {
                    Ok(())
                } else {
                    Err(format!("Validation failed for: {}", data))
                }
            }
            None => Err(format!("Validator '{}' not found", validator_name)),
        }
    }

    pub fn transform(&self, data: String, transformer_name: &str) -> Result<String, String> {
        match self.transformers.get(transformer_name) {
            Some(transformer) => Ok(transformer(data)),
            None => Err(format!("Transformer '{}' not found", transformer_name)),
        }
    }

    pub fn process_data(
        &self,
        data: &str,
        validator_name: &str,
        transformer_name: &str,
    ) -> Result<String, String> {
        self.validate(data, validator_name)?;
        let transformed = self.transform(data.to_string(), transformer_name)?;
        Ok(transformed)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validator("email", Box::new(|s| s.contains('@')));
    processor.add_validator("numeric", Box::new(|s| s.chars().all(|c| c.is_numeric())));

    processor.add_transformer("uppercase", Box::new(|s| s.to_uppercase()));
    processor.add_transformer("reverse", Box::new(|s| s.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("test@example.com", "email").is_ok());
        assert!(processor.validate("invalid-email", "email").is_err());
    }

    #[test]
    fn test_numeric_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("12345", "numeric").is_ok());
        assert!(processor.validate("123abc", "numeric").is_err());
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = create_default_processor();
        let result = processor.transform("hello".to_string(), "uppercase");
        assert_eq!(result.unwrap(), "HELLO");
    }

    #[test]
    fn test_reverse_transformation() {
        let processor = create_default_processor();
        let result = processor.transform("rust".to_string(), "reverse");
        assert_eq!(result.unwrap(), "tsur");
    }

    #[test]
    fn test_full_processing() {
        let processor = create_default_processor();
        let result = processor.process_data("test@example.com", "email", "uppercase");
        assert_eq!(result.unwrap(), "TEST@EXAMPLE.COM");
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing timeout")]
    Timeout,
    #[error("Serialization error")]
    SerializationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) -> Result<(), DataError> {
        if !value.is_finite() {
            return Err(DataError::InvalidInput(
                "Value must be finite number".to_string(),
            ));
        }
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidInput("ID cannot be zero".to_string()));
        }
        if self.timestamp < 0 {
            return Err(DataError::InvalidInput(
                "Timestamp cannot be negative".to_string(),
            ));
        }
        if self.values.is_empty() {
            return Err(DataError::InvalidInput(
                "Record must contain at least one value".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct DataProcessor {
    max_records: usize,
    processed_count: u64,
}

impl DataProcessor {
    pub fn new(max_records: usize) -> Self {
        Self {
            max_records,
            processed_count: 0,
        }
    }

    pub fn process_record(&mut self, record: &DataRecord) -> Result<DataRecord, DataError> {
        if self.processed_count as usize >= self.max_records {
            return Err(DataError::Timeout);
        }

        record.validate()?;

        let mut processed = record.clone();
        
        // Normalize values
        let sum: f64 = processed.values.values().sum();
        if sum.abs() > f64::EPSILON {
            for value in processed.values.values_mut() {
                *value /= sum;
            }
        }

        // Add processing tag
        processed.add_tag("processed");
        processed.add_tag(&format!("batch_{}", self.processed_count / 1000));

        self.processed_count += 1;
        Ok(processed)
    }

    pub fn batch_process(
        &mut self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, DataError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            match self.process_record(&record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }

    pub fn get_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("processed_count".to_string(), self.processed_count as f64);
        stats.insert("remaining_capacity".to_string(), (self.max_records - self.processed_count as usize) as f64);
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.validate().is_err());

        record.add_value("temperature", 25.5).unwrap();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_value() {
        let mut record = DataRecord::new(1, 1234567890);
        let result = record.add_value("invalid", f64::INFINITY);
        assert!(matches!(result, Err(DataError::InvalidInput(_))));
    }

    #[test]
    fn test_processor_limits() {
        let mut processor = DataProcessor::new(2);
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("test", 1.0).unwrap();

        assert!(processor.process_record(&record).is_ok());
        assert!(processor.process_record(&record).is_ok());
        assert!(matches!(processor.process_record(&record), Err(DataError::Timeout)));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.timestamp.is_empty()
    }

    pub fn display(&self) {
        println!("Record {}: {} at {}", self.id, self.value, self.timestamp);
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let timestamp = parts[2].to_string();

        let record = DataRecord::new(id, value, timestamp);
        if record.is_valid() {
            records.push(record);
        } else {
            eprintln!("Warning: Invalid record at line {}", line_num + 1);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "2023-10-01T12:00:00Z".to_string());
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord::new(0, 42.5, "2023-10-01T12:00:00Z".to_string());
        let record2 = DataRecord::new(1, f64::INFINITY, "2023-10-01T12:00:00Z".to_string());
        let record3 = DataRecord::new(1, 42.5, "".to_string());
        
        assert!(!record1.is_valid());
        assert!(!record2.is_valid());
        assert!(!record3.is_valid());
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,42.5,2023-10-01T12:00:00Z")?;
        writeln!(temp_file, "2,37.8,2023-10-01T12:05:00Z")?;
        writeln!(temp_file, "# Comment line")?;
        writeln!(temp_file, "")?;
        writeln!(temp_file, "3,29.1,2023-10-01T12:10:00Z")?;

        let records = process_csv_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].value, 37.8);
        assert_eq!(records[2].timestamp, "2023-10-01T12:10:00Z");

        Ok(())
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "timestamp1".to_string()),
            DataRecord::new(2, 20.0, "timestamp2".to_string()),
            DataRecord::new(3, 30.0, "timestamp3".to_string()),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}