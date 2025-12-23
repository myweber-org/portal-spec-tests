
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
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

            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() != 4 {
                continue;
            }

            let id = fields[0].parse::<u32>().unwrap_or(0);
            let name = fields[1].to_string();
            let value = fields[2].parse::<f64>().unwrap_or(0.0);
            let active = fields[3].parse::<bool>().unwrap_or(false);

            let record = Record::new(id, name, value, active);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.id == target_id)
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = Record::new(1, "Alpha".to_string(), 100.0, true);
        let record2 = Record::new(2, "Beta".to_string(), 200.0, false);
        
        processor.records.push(record1);
        processor.records.push(record2);

        assert_eq!(processor.total_records(), 2);
        assert_eq!(processor.filter_active().len(), 1);
        assert_eq!(processor.calculate_average(), Some(150.0));
        assert!(processor.find_by_id(1).is_some());
        assert!(processor.find_by_id(3).is_none());
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
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
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>], expected_columns: usize) -> Result<(), String> {
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                return Err(format!(
                    "Record {} has {} columns, expected {}",
                    index + 1,
                    record.len(),
                    expected_columns
                ));
            }
        }
        Ok(())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, String> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        let mut column_data = Vec::with_capacity(records.len());
        
        for (row_index, record) in records.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!(
                    "Column index {} out of bounds for record {} (max index: {})",
                    column_index,
                    row_index + 1,
                    record.len() - 1
                ));
            }
            column_data.push(record[column_index].clone());
        }
        
        Ok(column_data)
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
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_validate_records_valid() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records, 3).is_ok());
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let names = processor.extract_column(&records, 0).unwrap();
        assert_eq!(names, vec!["Alice", "Bob"]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
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

            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
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

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.75,CategoryA").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_records().len(), 3);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "A".to_string(), 10.0, "X".to_string()));
        processor.records.push(DataRecord::new(2, "B".to_string(), 20.0, "Y".to_string()));
        processor.records.push(DataRecord::new(3, "C".to_string(), 30.0, "X".to_string()));

        let filtered = processor.filter_by_category("X");
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(20.0));
    }
}use csv::Reader;
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

pub fn process_data_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 && record.category.len() <= 50 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let avg = if count > 0 { sum / count as f64 } else { 0.0 };
    let max_value = records.iter().map(|r| r.value).fold(0.0, f64::max);

    (avg, max_value, count)
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
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

            let active = match parts[3].to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                _ => false,
            };

            let record = Record::new(id, name, value, active);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records
            .iter()
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Record> {
        self.records
            .iter()
            .find(|r| r.name.to_lowercase() == name.to_lowercase())
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
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Alice,100.5,true").unwrap();
        writeln!(temp_file, "2,Bob,75.3,false").unwrap();
        writeln!(temp_file, "3,Charlie,50.0,true").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.filter_active().len(), 2);
        assert_eq!(processor.calculate_total(), 225.8);
        
        assert!(processor.find_by_name("Alice").is_some());
        assert!(processor.find_by_name("Nonexistent").is_none());
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
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

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.data.len();
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        format!(
            "Data Summary:\n  Count: {}\n  Mean: {:.2}\n  Min: {:.2}\n  Max: {:.2}\n  Unique Categories: {}",
            count, mean, min, max, self.frequency_map.len()
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
        writeln!(temp_file, "10.5,CategoryA").unwrap();
        writeln!(temp_file, "20.3,CategoryB").unwrap();
        writeln!(temp_file, "15.7,CategoryA").unwrap();
        writeln!(temp_file, "25.1,CategoryC").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(17.9));
        assert_eq!(processor.data.len(), 4);
        assert_eq!(processor.get_frequency_distribution().get("CategoryA"), Some(&2));
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 3);
    }
}