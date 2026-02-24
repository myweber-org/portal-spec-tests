
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: ValidationRules,
}

pub struct ValidationRules {
    min_value: f64,
    max_value: f64,
    required_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if !self.validation_rules.required_keys.contains(&key) {
            return Err(format!("Key '{}' is not in required keys list", key));
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!("Value {} is outside allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, Stats> {
        let mut stats_map = HashMap::new();
        
        for (key, values) in &self.data {
            if values.is_empty() {
                continue;
            }

            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[count as usize / 2]
            };

            stats_map.insert(key.clone(), Stats {
                mean,
                median,
                std_dev,
                min: *sorted_values.first().unwrap(),
                max: *sorted_values.last().unwrap(),
                count: values.len(),
            });
        }
        
        stats_map
    }

    pub fn normalize_data(&mut self) {
        for values in self.data.values_mut() {
            if values.is_empty() {
                continue;
            }

            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            if (max - min).abs() > f64::EPSILON {
                for value in values.iter_mut() {
                    *value = (*value - min) / (max - min);
                }
            }
        }
    }

    pub fn get_data_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn get_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.data.get(key)
    }
}

pub struct Stats {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl ValidationRules {
    pub fn new(min_value: f64, max_value: f64, required_keys: Vec<String>) -> Self {
        ValidationRules {
            min_value,
            max_value,
            required_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["temperature".to_string(), "humidity".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        
        assert!(processor.add_dataset(
            "temperature".to_string(),
            vec![20.0, 25.0, 30.0, 35.0]
        ).is_ok());
        
        assert!(processor.add_dataset(
            "pressure".to_string(),
            vec![100.0, 200.0]
        ).is_err());
        
        assert!(processor.add_dataset(
            "temperature".to_string(),
            vec![-10.0, 150.0]
        ).is_err());
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats.len(), 1);
        assert!(stats.contains_key("temperature"));
        
        processor.normalize_data();
        let normalized_data = processor.get_data("temperature").unwrap();
        assert!(normalized_data.iter().all(|&x| x >= 0.0 && x <= 1.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_result) = lines.next() {
            let header_line = header_result?;
            let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();

            for line_result in lines {
                let line = line_result?;
                let values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
                
                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        record.insert(header.clone(), values[i].clone());
                    }
                    self.records.push(record);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self, column_name: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.get(column_name) {
                if let Ok(value) = value_str.parse::<f64>() {
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

    pub fn count_unique_values(&self, column_name: &str) -> HashMap<String, usize> {
        let mut frequency_map = HashMap::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_name) {
                *frequency_map.entry(value.clone()).or_insert(0) += 1;
            }
        }
        
        frequency_map
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_column_names(&self) -> Vec<String> {
        if let Some(first_record) = self.records.first() {
            first_record.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let file_path = temp_file.path().to_str().unwrap();
        
        let mut processor = DataProcessor::new();
        let load_result = processor.load_csv(file_path);
        assert!(load_result.is_ok());
        
        assert_eq!(processor.get_record_count(), 3);
        
        let avg_age = processor.calculate_average("age");
        assert_eq!(avg_age, Some(30.0));
        
        let avg_salary = processor.calculate_average("salary");
        assert_eq!(avg_salary, Some(51666.666666666664));
        
        let filtered = processor.filter_records(|record| {
            record.get("age").and_then(|a| a.parse::<i32>().ok()).unwrap_or(0) > 30
        });
        assert_eq!(filtered.len(), 1);
        
        let column_names = processor.get_column_names();
        assert_eq!(column_names, vec!["name", "age", "salary"]);
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
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    (mean, variance.sqrt())
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        if let Some(header_result) = lines.next() {
            let header_line = header_result?;
            let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();
            
            for line_result in lines {
                let line = line_result?;
                let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                
                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        if let Ok(num) = values[i].parse::<f64>() {
                            record.insert(header.clone(), num);
                        }
                    }
                    self.records.push(record);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self, column_name: &str) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column_name).copied())
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

    pub fn filter_records(&self, column_name: &str, threshold: f64) -> Vec<HashMap<String, f64>> {
        self.records
            .iter()
            .filter(|record| {
                record.get(column_name)
                    .map(|&value| value > threshold)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
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
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,score").unwrap();
        writeln!(temp_file, "1,10.5,0.8").unwrap();
        writeln!(temp_file, "2,15.2,0.9").unwrap();
        writeln!(temp_file, "3,8.7,0.6").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let stats = processor.calculate_statistics("value");
        assert!(stats.is_some());
        
        let filtered = processor.filter_records("score", 0.7);
        assert_eq!(filtered.len(), 2);
    }
}
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u32, name: &str, value: f64) -> Self {
        Self {
            id,
            name: name.to_string(),
            value,
            tags: Vec::new(),
        }
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), String> {
        if !record.is_valid() {
            return Err("Invalid record data".to_string());
        }

        if self.records.contains_key(&record.id) {
            return Err(format!("Record with id {} already exists", record.id));
        }

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn filter_by_value(&self, min_value: f64) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.value >= min_value)
            .collect()
    }

    pub fn calculate_statistics(&self) -> (f64, f64, usize) {
        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0);
        }

        let sum: f64 = values.iter().sum();
        let count = values.len();
        let average = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|v| (v - average).powi(2))
            .sum::<f64>() / count as f64;
        
        (average, variance, count)
    }

    pub fn transform_records<F>(&mut self, transform_fn: F) 
    where
        F: Fn(&mut DataRecord),
    {
        for record in self.records.values_mut() {
            transform_fn(record);
        }
    }
}

pub fn process_data_batch(records: Vec<DataRecord>) -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    for record in records {
        if let Err(e) = processor.add_record(record) {
            eprintln!("Failed to add record: {}", e);
        }
    }
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "test", 10.5);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "", -5.0);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, "sample", 25.0);
        
        assert!(processor.add_record(record).is_ok());
        assert!(processor.get_record(1).is_some());
        assert!(processor.get_record(2).is_none());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "a", 10.0)).unwrap();
        processor.add_record(DataRecord::new(2, "b", 20.0)).unwrap();
        processor.add_record(DataRecord::new(3, "c", 30.0)).unwrap();
        
        let (avg, var, count) = processor.calculate_statistics();
        assert_eq!(count, 3);
        assert!((avg - 20.0).abs() < 0.001);
        assert!(var > 0.0);
    }
}