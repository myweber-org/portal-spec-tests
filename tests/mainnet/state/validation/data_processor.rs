
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
    valid_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
            valid_count: 0,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();

            let record = DataRecord::new(id, value, category);
            if record.is_valid() {
                self.total_value += record.get_value();
                self.valid_count += 1;
            }
            self.records.push(record);
        }

        Ok(())
    }

    pub fn get_average_value(&self) -> f64 {
        if self.valid_count == 0 {
            0.0
        } else {
            self.total_value / self.valid_count as f64
        }
    }

    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn count_records(&self) -> usize {
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
        writeln!(temp_file, "1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,-5.0,CategoryB").unwrap();
        writeln!(temp_file, "3,20.0,").unwrap();
        writeln!(temp_file, "# Comment line").unwrap();
        writeln!(temp_file, "4,15.0,CategoryC").unwrap();

        processor.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.get_valid_records().len(), 2);
        assert_eq!(processor.get_average_value(), 12.75);
    }

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 10.0, "Test".to_string());
        assert!(valid_record.is_valid());

        let invalid_value = DataRecord::new(2, -1.0, "Test".to_string());
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(3, 5.0, "".to_string());
        assert!(!invalid_category.is_valid());
    }
}