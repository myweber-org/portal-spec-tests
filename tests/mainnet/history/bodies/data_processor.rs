use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    pub records: Vec<Vec<String>>,
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

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                self.records.push(fields);
            }
        }

        Ok(())
    }

    pub fn validate_numeric_column(&self, column_index: usize) -> bool {
        if self.records.is_empty() {
            return false;
        }

        for record in &self.records {
            if column_index >= record.len() {
                return false;
            }
            if record[column_index].parse::<f64>().is_err() {
                return false;
            }
        }
        true
    }

    pub fn calculate_column_average(&self, column_index: usize) -> Option<f64> {
        if !self.validate_numeric_column(column_index) {
            return None;
        }

        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Ok(value) = record[column_index].parse::<f64>() {
                sum += value;
                count += 1;
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 2);
    }

    #[test]
    fn test_numeric_validation() {
        let mut processor = DataProcessor::new();
        processor.records.push(vec!["test".to_string(), "42.5".to_string()]);
        processor.records.push(vec!["test2".to_string(), "100".to_string()]);

        assert!(processor.validate_numeric_column(1));
        assert!(!processor.validate_numeric_column(0));
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(vec!["42.0".to_string()]);
        processor.records.push(vec!["58.0".to_string()]);

        let avg = processor.calculate_column_average(0);
        assert_eq!(avg, Some(50.0));
    }
}