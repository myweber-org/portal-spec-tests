use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
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

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Skip header
        if let Some(Ok(header)) = lines.next() {
            if !header.contains("id,value,category") {
                return Err("Invalid CSV format".into());
            }
        }

        for (line_num, line) in lines.enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() != 3 {
                return Err(format!("Invalid data at line {}", line_num + 2).into());
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2];

            match DataRecord::new(id, value, category) {
                Ok(record) => self.records.push(record),
                Err(e) => return Err(format!("Error at line {}: {}", line_num + 2, e).into()),
            }
        }

        Ok(())
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

    pub fn get_statistics(&self) -> (Option<f64>, Option<f64>, usize) {
        if self.records.is_empty() {
            return (None, None, 0);
        }

        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
        let max = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
        let count = self.records.len();

        (Some(min), Some(max), count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.calculate_average(), None);
        assert_eq!(processor.get_statistics().2, 0);
    }

    #[test]
    fn test_csv_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,value,category").unwrap();
        writeln!(file, "1,10.5,category_a").unwrap();
        writeln!(file, "2,20.0,category_b").unwrap();
        writeln!(file, "3,15.75,category_a").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_average(), Some(15.416666666666666));
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, Some(10.5));
        assert_eq!(stats.1, Some(20.0));
        assert_eq!(stats.2, 3);
    }
}