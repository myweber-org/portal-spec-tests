use csv::Reader;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    timestamp: String,
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

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value > threshold)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.id == target_id)
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self.records)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        let csv_data = "id,name,value,timestamp\n1,test1,10.5,2023-01-01\n2,test2,20.3,2023-01-02";
        
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), csv_data).unwrap();
        
        assert!(processor.load_from_csv(temp_file.path()).is_ok());
        assert_eq!(processor.records.len(), 2);
        
        let filtered = processor.filter_by_value(15.0);
        assert_eq!(filtered.len(), 1);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.4).abs() < 0.01);
        
        let found = processor.find_by_id(1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test1");
    }
}use std::error::Error;
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
        
        if let Some(header_line) = lines.next() {
            let headers: Vec<String> = header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            for line in lines {
                let line = line?;
                let values: Vec<String> = line
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                
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

    pub fn calculate_statistics(&self, column: &str) -> Option<Statistics> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column).and_then(|v| v.parse::<f64>().ok()))
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len();
        let mean = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        Some(Statistics {
            count,
            mean,
            variance,
            min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        })
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

    pub fn get_column_unique_values(&self, column: &str) -> Vec<String> {
        let mut values: Vec<String> = self.records
            .iter()
            .filter_map(|record| record.get(column).cloned())
            .collect();
        
        values.sort();
        values.dedup();
        values
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Count: {}, Mean: {:.2}, Variance: {:.2}, Min: {:.2}, Max: {:.2}",
            self.count, self.mean, self.variance, self.min, self.max
        )
    }
}