
use std::collections::HashMap;

pub struct DataProcessor {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
    transformers: Vec<Box<dyn Fn(HashMap<String, String>) -> HashMap<String, String>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            filters: Vec::new(),
            transformers: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn add_transformer<F>(&mut self, transformer: F)
    where
        F: Fn(HashMap<String, String>) -> HashMap<String, String> + 'static,
    {
        self.transformers.push(Box::new(transformer));
    }

    pub fn process(&self, data: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        data.into_iter()
            .filter(|record| self.filters.iter().all(|filter| filter(record)))
            .map(|record| {
                let mut transformed = record;
                for transformer in &self.transformers {
                    transformed = transformer(transformed);
                }
                transformed
            })
            .collect()
    }
}

pub fn create_sample_data() -> Vec<HashMap<String, String>> {
    let mut data = Vec::new();
    
    let mut record1 = HashMap::new();
    record1.insert("id".to_string(), "1".to_string());
    record1.insert("name".to_string(), "Alice".to_string());
    record1.insert("age".to_string(), "30".to_string());
    record1.insert("active".to_string(), "true".to_string());
    data.push(record1);
    
    let mut record2 = HashMap::new();
    record2.insert("id".to_string(), "2".to_string());
    record2.insert("name".to_string(), "Bob".to_string());
    record2.insert("age".to_string(), "25".to_string());
    record2.insert("active".to_string(), "false".to_string());
    data.push(record2);
    
    let mut record3 = HashMap::new();
    record3.insert("id".to_string(), "3".to_string());
    record3.insert("name".to_string(), "Charlie".to_string());
    record3.insert("age".to_string(), "35".to_string());
    record3.insert("active".to_string(), "true".to_string());
    data.push(record3);
    
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_filter(|record| {
            record.get("active").map(|v| v == "true").unwrap_or(false)
        });
        
        processor.add_transformer(|mut record| {
            if let Some(age_str) = record.get("age") {
                if let Ok(age) = age_str.parse::<i32>() {
                    let age_group = if age < 30 { "young" } else { "adult" };
                    record.insert("age_group".to_string(), age_group.to_string());
                }
            }
            record
        });
        
        let data = create_sample_data();
        let result = processor.process(data);
        
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|r| r.get("active") == Some(&"true".to_string())));
        assert!(result.iter().all(|r| r.contains_key("age_group")));
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        groups
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        let total = self.records.len();
        let valid = self.validate_records().len();
        (total, valid)
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,15.0,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,-5.0,Category1").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.validate_records().len(), 2);
        assert_eq!(processor.calculate_average(), Some(12.75));
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("Category1").unwrap().len(), 2);
        assert_eq!(groups.get("Category2").unwrap().len(), 1);
        
        let stats = processor.get_statistics();
        assert_eq!(stats, (3, 2));
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
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: i64) -> Self {
        DataRecord {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && self.timestamp > 0
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

            let timestamp = match parts[3].parse::<i64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, min_value: f64, max_value: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= min_value && r.value <= max_value)
            .cloned()
            .collect()
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
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
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, 1234567890);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, 0);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,record1,10.5,1234567890").unwrap();
        writeln!(temp_file, "2,record2,20.0,1234567891").unwrap();
        writeln!(temp_file, "3,,15.0,1234567892").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.get_records().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "a".to_string(), 10.0, 1000));
        processor.records.push(DataRecord::new(2, "b".to_string(), 20.0, 1001));
        processor.records.push(DataRecord::new(3, "c".to_string(), 30.0, 1002));

        let (mean, variance, std_dev) = processor.calculate_statistics();
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}