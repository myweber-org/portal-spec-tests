use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_fields(&self, records: &[Vec<String>], field_index: usize) -> Result<Vec<f64>, String> {
        let mut numeric_values = Vec::new();
        
        for (row_num, record) in records.iter().enumerate() {
            if field_index >= record.len() {
                return Err(format!("Row {}: Field index {} out of bounds", row_num + 1, field_index));
            }
            
            match record[field_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", row_num + 1, record[field_index])),
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.75").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.5"]);
    }

    #[test]
    fn test_numeric_validation() {
        let records = vec![
            vec!["10.5".to_string(), "text".to_string()],
            vec!["20.0".to_string(), "more".to_string()],
            vec!["15.75".to_string(), "data".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.validate_numeric_fields(&records, 0);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![10.5, 20.0, 15.75]);
    }

    #[test]
    fn test_statistics_calculation() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let processor = DataProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert_eq!(mean, 30.0);
        assert_eq!(variance, 200.0);
        assert_eq!(std_dev, 200.0_f64.sqrt());
    }
}