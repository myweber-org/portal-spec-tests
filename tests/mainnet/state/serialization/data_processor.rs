use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if let Some(value) = parts.get(0) {
                if let Ok(num) = value.parse::<f64>() {
                    self.data.push(num);
                }
            }
            
            if let Some(category) = parts.get(1) {
                *self.frequency_map.entry(category.to_string()).or_insert(0) += 1;
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "15.5,CategoryA").unwrap();
        writeln!(temp_file, "22.3,CategoryB").unwrap();
        writeln!(temp_file, "18.7,CategoryA").unwrap();
        writeln!(temp_file, "25.1,CategoryC").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 20.4).abs() < 0.01);
        
        let median = processor.calculate_median().unwrap();
        assert!((median - 20.5).abs() < 0.01);
        
        let distribution = processor.get_frequency_distribution();
        assert_eq!(distribution.get("CategoryA"), Some(&2));
        assert_eq!(distribution.get("CategoryB"), Some(&1));
        
        let filtered = processor.filter_data(20.0);
        assert_eq!(filtered.len(), 2);
    }
}