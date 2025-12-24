
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Statistics {
        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = if count > 0 { sum / count as f64 } else { 0.0 };
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Statistics {
            count,
            sum,
            mean,
            min,
            max,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, String>> {
    records
        .into_iter()
        .map(|mut record| {
            match record.validate() {
                Ok(_) => {
                    if record.values.len() > 10 {
                        record.values.truncate(10);
                    }
                    Ok(record)
                }
                Err(e) => Err(e),
            }
        })
        .collect()
}

pub fn filter_valid_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.validate().is_ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.5);
        assert!(record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0);
        record.add_value(20.0);
        record.add_value(30.0);

        let stats = record.calculate_statistics();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_process_records() {
        let mut valid_record = DataRecord::new(1, 1234567890);
        valid_record.add_value(42.5);

        let invalid_record = DataRecord::new(0, 1234567890);

        let records = vec![valid_record, invalid_record];
        let results = process_records(records);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;
        let mut count = 0;

        for result in reader.deserialize() {
            let record: DataRecord = result?;
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new().has_headers(true).from_path(path)?;

        for record in &self.records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, "Item1".to_string(), 10.0, "CategoryA".to_string()));
        processor.add_record(DataRecord::new(2, "Item2".to_string(), 20.0, "CategoryB".to_string()));
        processor.add_record(DataRecord::new(3, "Item3".to_string(), 30.0, "CategoryA".to_string()));

        assert_eq!(processor.get_records().len(), 3);
        
        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average_value();
        assert_eq!(avg, Some(20.0));
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "Test".to_string(), 15.5, "Sample".to_string()));

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        let count = new_processor.load_from_csv(path)?;
        
        assert_eq!(count, 1);
        assert_eq!(new_processor.get_records().len(), 1);
        
        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    pub fn process_file(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.has_header && index == 0 {
                continue;
            }

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

    pub fn calculate_average(&self, records: &[Vec<String>], column_index: usize) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path().to_str().unwrap());

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        
        let valid_record = vec!["test".to_string(), "123".to_string()];
        assert!(processor.validate_record(&valid_record));

        let invalid_record = vec!["".to_string(), "data".to_string()];
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "30.0".to_string()],
            vec!["invalid".to_string(), "40.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let average = processor.calculate_average(&records, 0);

        assert_eq!(average, Some(13.0));
    }
}