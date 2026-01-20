use csv::Reader;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if !["A", "B", "C"].contains(&self.category.as_str()) {
            return Err("Category must be A, B, or C".to_string());
        }
        Ok(())
    }
}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (index, result) in reader.deserialize().enumerate() {
        match result {
            Ok(record) => {
                let record: Record = record;
                if let Err(e) = record.validate() {
                    errors.push(format!("Row {}: {}", index + 1, e));
                } else {
                    records.push(record);
                }
            }
            Err(e) => errors.push(format!("Row {}: Parse error - {}", index + 1, e)),
        }
    }

    if !errors.is_empty() {
        return Err(format!("Validation errors:\n{}", errors.join("\n")).into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_record() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            category: "D".to_string(),
        };
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,Item1,100.0,A").unwrap();
        writeln!(file, "2,Item2,200.0,B").unwrap();
        
        let result = process_csv_file(file.path());
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        DataRecord {
            id,
            value,
            category: category.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), &'static str> {
        if multiplier.is_finite() && multiplier != 0.0 {
            self.value *= multiplier;
            Ok(())
        } else {
            Err("Invalid multiplier provided")
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<(), &'static str>> {
    records
        .iter_mut()
        .map(|record| {
            if record.is_valid() {
                record.transform(1.5)
            } else {
                Err("Invalid record detected")
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 10.5, "category_a");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, f64::NAN, "");
        assert!(!record.is_valid());
    }

    #[test]
    fn test_transform_operation() {
        let mut record = DataRecord::new(1, 10.0, "test");
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 20.0);
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, 5.0, "test");
        record.add_metadata("source", "generated");
        assert_eq!(record.get_metadata("source"), Some(&"generated".to_string()));
    }
}