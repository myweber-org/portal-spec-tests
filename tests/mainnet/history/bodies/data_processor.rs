use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_data = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if columns.len() > filter_column && columns[filter_column] == filter_value {
                filtered_data.push(columns);
            }
        }

        Ok(filtered_data)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut sum = 0.0;
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if let Some(value_str) = columns.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        writeln!(temp_file, "Bob,30,Paris").unwrap();
        writeln!(temp_file, "Charlie,25,New York").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv(1, "25").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][0], "Charlie");
    }

    #[test]
    fn test_calculate_average() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,score").unwrap();
        writeln!(temp_file, "Alice,85.5").unwrap();
        writeln!(temp_file, "Bob,92.0").unwrap();
        writeln!(temp_file, "Charlie,78.5").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(1).unwrap();

        assert!((average - 85.333).abs() < 0.001);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<Box<dyn Fn(&[f64]) -> bool>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                Box::new(|data: &[f64]| !data.is_empty()),
                Box::new(|data: &[f64]| data.iter().all(|&x| x.is_finite())),
                Box::new(|data: &[f64]| data.len() < 10000),
            ],
        }
    }

    pub fn process(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if !self.validate_data(data) {
            return Err("Data validation failed".to_string());
        }

        let processed: Vec<f64> = data
            .iter()
            .map(|&x| x.powi(2).sqrt().ln_1p())
            .collect();

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_cached(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    fn validate_data(&self, data: &[f64]) -> bool {
        self.validation_rules.iter().all(|rule| rule(data))
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&[f64]) -> bool + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }
}

pub fn calculate_statistics(data: &[f64]) -> (f64, f64, f64) {
    if data.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = data.iter().sum();
    let mean = sum / data.len() as f64;
    
    let variance: f64 = data
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / data.len() as f64;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process("test", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), data.len());
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (mean, variance, std_dev) = calculate_statistics(&data);
        
        assert!((mean - 5.0).abs() < 0.001);
        assert!((variance - 4.0).abs() < 0.001);
        assert!((std_dev - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![f64::INFINITY, 2.0, 3.0];
        
        let result = processor.process("invalid", &invalid_data);
        assert!(result.is_err());
    }
}