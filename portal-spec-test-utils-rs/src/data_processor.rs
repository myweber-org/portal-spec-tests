
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }
            
            let id = parts[0].parse().unwrap_or(0);
            let value = parts[1].parse().unwrap_or(0.0);
            let category = parts[2].to_string();
            
            let record = DataRecord::new(id, value, category);
            self.add_record(record);
        }
        
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.total_value += record.get_value();
            self.valid_count += 1;
        }
        self.records.push(record);
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.valid_count > 0 {
            Some(self.total_value / self.valid_count as f64)
        } else {
            None
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.is_valid())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, usize, Option<f64>) {
        let total = self.records.len();
        let average = self.get_average_value();
        (total, self.valid_count, average)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "A".to_string());
        assert!(valid_record.is_valid());
        
        let invalid_value = DataRecord::new(2, -1.0, "B".to_string());
        assert!(!invalid_value.is_valid());
        
        let invalid_category = DataRecord::new(3, 10.0, "".to_string());
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, 10.0, "TypeA".to_string()));
        processor.add_record(DataRecord::new(2, 20.0, "TypeA".to_string()));
        processor.add_record(DataRecord::new(3, 30.0, "TypeB".to_string()));
        
        let filtered = processor.filter_by_category("TypeA");
        assert_eq!(filtered.len(), 2);
        
        let average = processor.get_average_value();
        assert!(average.is_some());
        assert_eq!(average.unwrap(), 20.0);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,Alpha").unwrap();
        writeln!(temp_file, "2,20.3,Beta").unwrap();
        writeln!(temp_file, "3,invalid,Gamma").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        
        let (total, valid, _) = processor.get_statistics();
        assert_eq!(total, 2);
        assert_eq!(valid, 2);
    }
}use std::error::Error;
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
            return Err(format!("Invalid value {} for record {}", value, id));
        }
        if category.is_empty() {
            return Err(format!("Empty category for record {}", id));
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;

        for line in reader.lines() {
            line_count += 1;
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_count).into());
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => self.records.push(record),
                Err(e) => eprintln!("Skipping record at line {}: {}", line_count, e),
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let count = self.records.len() as f64;
        let total = self.calculate_total();
        let mean = total / count;

        let variance: f64 = self
            .records
            .iter()
            .map(|record| (record.value - mean).powi(2))
            .sum::<f64>()
            / count;

        (total, mean, variance.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_record_creation() {
        let record = DataRecord::new(2, -5.0, "test".to_string());
        assert!(record.is_err());

        let record = DataRecord::new(3, 10.0, "".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "1,10.5,category_a\n2,20.0,category_b\n3,15.75,category_a"
        )
        .unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);

        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);

        let (total, mean, _) = processor.get_statistics();
        assert!((total - 46.25).abs() < 0.001);
        assert!((mean - 15.416666).abs() < 0.001);
    }
}