
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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
            if parts.len() != 3 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[2].trim().to_string();
            
            if !self.validate_record(id, value, &category) {
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
    
    fn validate_record(&self, id: u32, value: f64, category: &str) -> bool {
        if id == 0 {
            return false;
        }
        
        if value < 0.0 || value > 10000.0 {
            return false;
        }
        
        if category.is_empty() || category.len() > 50 {
            return false;
        }
        
        true
    }
    
    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_processor_creation() {
        let processor = DataProcessor::new();
        assert_eq!(processor.records.len(), 0);
    }
    
    #[test]
    fn test_load_valid_csv() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,42.5,TypeA").unwrap();
        writeln!(temp_file, "2,78.9,TypeB").unwrap();
        writeln!(temp_file, "3,15.3,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.records.len(), 3);
    }
    
    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "Test".to_string(),
        });
        
        processor.records.push(DataRecord {
            id: 2,
            value: 20.0,
            category: "Test".to_string(),
        });
        
        processor.records.push(DataRecord {
            id: 3,
            value: 30.0,
            category: "Test".to_string(),
        });
        
        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }
    
    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "TypeA".to_string(),
        });
        
        processor.records.push(DataRecord {
            id: 2,
            value: 20.0,
            category: "TypeB".to_string(),
        });
        
        processor.records.push(DataRecord {
            id: 3,
            value: 30.0,
            category: "TypeA".to_string(),
        });
        
        let filtered = processor.filter_by_category("TypeA");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "TypeA"));
    }
    
    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new();
        
        assert!(processor.validate_record(1, 50.0, "Valid"));
        assert!(!processor.validate_record(0, 50.0, "Valid"));
        assert!(!processor.validate_record(1, -10.0, "Valid"));
        assert!(!processor.validate_record(1, 50000.0, "Valid"));
        assert!(!processor.validate_record(1, 50.0, ""));
    }
}