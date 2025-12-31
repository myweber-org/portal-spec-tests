use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
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
                Ok(num) => num,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(num) => num,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(_) => continue,
            }
        }

        Ok(count)
    }

    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 100.0, "A".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "A");
    }

    #[test]
    fn test_invalid_record() {
        let result = DataRecord::new(1, -10.0, "A".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_tax_calculation() {
        let record = DataRecord::new(1, 100.0, "A".to_string()).unwrap();
        assert_eq!(record.calculate_tax(0.1), 10.0);
    }

    #[test]
    fn test_csv_loading() {
        let mut csv_data = Vec::new();
        writeln!(csv_data, "id,value,category").unwrap();
        writeln!(csv_data, "1,100.0,TypeA").unwrap();
        writeln!(csv_data, "2,200.0,TypeB").unwrap();
        writeln!(csv_data, "3,invalid,TypeC").unwrap();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&csv_data).unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.total_value(), 300.0);
    }
}