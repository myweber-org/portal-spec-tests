use csv::{Reader, Writer};
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

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_writer(File::create(output_path)?);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let values: Vec<f64> = records.iter().map(|r| r.value).collect();
    let count = values.len() as f64;
    
    let sum: f64 = values.iter().sum();
    let mean = sum / count;
    
    let variance: f64 = values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.id > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Valid".to_string(),
            value: 100.0,
            active: true,
        };
        
        let invalid_record = Record {
            id: 0,
            name: "".to_string(),
            value: 50.0,
            active: false,
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }
    
    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub fn load_csv_data(file_path: &Path) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();
    
    for result in rdr.records() {
        let record = result?;
        if record.len() >= 3 {
            let id: u32 = record[0].parse()?;
            let value: f64 = record[1].parse()?;
            let category = &record[2];
            
            match DataRecord::new(id, value, category) {
                Ok(data_record) => records.push(data_record),
                Err(e) => eprintln!("Skipping invalid record: {}", e),
            }
        }
    }
    
    Ok(records)
}

pub fn process_records(records: &[DataRecord]) -> (f64, f64, usize) {
    let total: f64 = records.iter().map(|r| r.value).sum();
    let average = if !records.is_empty() {
        total / records.len() as f64
    } else {
        0.0
    };
    let count = records.len();
    
    (total, average, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "analytics").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "analytics");
    }
    
    #[test]
    fn test_invalid_data_record() {
        let result = DataRecord::new(2, -5.0, "test");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_calculate_adjusted_value() {
        let record = DataRecord::new(3, 100.0, "finance").unwrap();
        assert_eq!(record.calculate_adjusted_value(1.5), 150.0);
    }
    
    #[test]
    fn test_load_csv_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,25.5,analytics").unwrap();
        writeln!(temp_file, "2,30.0,finance").unwrap();
        
        let records = load_csv_data(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].category, "analytics");
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 10.0, "a").unwrap(),
            DataRecord::new(2, 20.0, "b").unwrap(),
            DataRecord::new(3, 30.0, "c").unwrap(),
        ];
        
        let (total, average, count) = process_records(&records);
        assert_eq!(total, 60.0);
        assert_eq!(average, 20.0);
        assert_eq!(count, 3);
    }
}