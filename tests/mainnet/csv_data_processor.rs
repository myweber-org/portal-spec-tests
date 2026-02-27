
use std::error::Error;
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
        DataRecord {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            eprintln!("Warning: Invalid format at line {}", line_number);
            continue;
        }

        let id = match parts[0].parse::<u32>() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Warning: Invalid ID at line {}", line_number);
                continue;
            }
        };

        let name = parts[1].trim().to_string();
        let value = match parts[2].parse::<f64>() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Warning: Invalid value at line {}", line_number);
                continue;
            }
        };

        let category = parts[3].trim().to_string();
        let record = DataRecord::new(id, name, value, category);

        if record.is_valid() {
            records.push(record);
        } else {
            eprintln!("Warning: Invalid record at line {}", line_number);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[DataRecord], category: &str) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn process_data_pipeline(file_path: &str, target_category: &str) -> Result<(), Box<dyn Error>> {
    println!("Loading data from: {}", file_path);
    let records = load_csv_data(file_path)?;
    
    println!("Total records loaded: {}", records.len());
    
    let filtered = filter_by_category(&records, target_category);
    println!("Records in category '{}': {}", target_category, filtered.len());
    
    if let Some(avg) = calculate_average(&filtered) {
        println!("Average value for category '{}': {:.2}", target_category, avg);
    } else {
        println!("No records found for category '{}'", target_category);
    }
    
    let valid_count = records.iter().filter(|r| r.is_valid()).count();
    println!("Valid records: {}", valid_count);
    
    Ok(())
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
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0, "A".to_string());
        record.transform_value(2.5);
        assert_eq!(record.value, 25.0);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        
        let records = load_csv_data(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_filtering() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.0, "Cat1".to_string()),
            DataRecord::new(2, "B".to_string(), 20.0, "Cat2".to_string()),
            DataRecord::new(3, "C".to_string(), 30.0, "Cat1".to_string()),
        ];
        
        let filtered = filter_by_category(&records, "Cat1");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.0, "Cat1".to_string()),
            DataRecord::new(2, "B".to_string(), 20.0, "Cat1".to_string()),
            DataRecord::new(3, "C".to_string(), 30.0, "Cat1".to_string()),
        ];
        
        let avg = calculate_average(&records).unwrap();
        assert_eq!(avg, 20.0);
        
        let empty: Vec<DataRecord> = vec![];
        assert!(calculate_average(&empty).is_none());
    }
}