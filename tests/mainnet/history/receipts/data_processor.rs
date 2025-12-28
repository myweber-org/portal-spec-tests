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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            for part in parts {
                if let Ok(value) = part.trim().parse::<f64>() {
                    self.data.push(value);
                } else {
                    self.frequency_map
                        .entry(part.trim().to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.data.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.data
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn get_top_categories(&self, limit: usize) -> Vec<(String, u32)> {
        let mut entries: Vec<_> = self.frequency_map.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        
        entries
            .into_iter()
            .take(limit)
            .map(|(key, value)| (key.clone(), *value))
            .collect()
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&value| value >= threshold)
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
        writeln!(temp_file, "10.5,20.3,15.7").unwrap();
        writeln!(temp_file, "category_a,category_b,category_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let (mean, variance, std_dev) = processor.calculate_statistics();
        assert!((mean - 15.5).abs() < 0.01);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
        
        let top_categories = processor.get_top_categories(2);
        assert_eq!(top_categories.len(), 2);
        assert_eq!(top_categories[0].0, "category_a");
        
        let filtered = processor.filter_data(15.0);
        assert_eq!(filtered.len(), 2);
    }
}