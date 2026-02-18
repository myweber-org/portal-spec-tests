
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, f64>>,
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
        let mut lines = reader.lines();

        if let Some(header_result) = lines.next() {
            let header = header_result?;
            let columns: Vec<&str> = header.split(',').collect();

            for line_result in lines {
                let line = line_result?;
                let values: Vec<&str> = line.split(',').collect();
                
                if values.len() != columns.len() {
                    continue;
                }

                let mut record = HashMap::new();
                for (i, column) in columns.iter().enumerate() {
                    if let Ok(num) = values[i].parse::<f64>() {
                        record.insert(column.to_string(), num);
                    }
                }

                if !record.is_empty() {
                    self.records.push(record);
                }
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, column_name: &str) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column_name).copied())
            .collect();

        if values.is_empty() {
            return None;
        }

        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }

    pub fn filter_records(&self, column_name: &str, threshold: f64) -> Vec<HashMap<String, f64>> {
        self.records
            .iter()
            .filter(|record| {
                record.get(column_name)
                    .map(|&value| value > threshold)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn record_count(&self) -> usize {
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
        writeln!(temp_file, "id,value,score").unwrap();
        writeln!(temp_file, "1,10.5,0.8").unwrap();
        writeln!(temp_file, "2,15.2,0.9").unwrap();
        writeln!(temp_file, "3,8.7,0.7").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let stats = processor.calculate_statistics("value");
        assert!(stats.is_some());
        
        let (mean, _, std_dev) = stats.unwrap();
        assert!((mean - 11.466666).abs() < 0.001);
        assert!(std_dev > 0.0);
        
        let filtered = processor.filter_records("value", 10.0);
        assert_eq!(filtered.len(), 2);
    }
}use std::error::Error;
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
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
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

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&value| value > threshold)
            .cloned()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.data.len();
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        format!(
            "Data Summary:\n  Count: {}\n  Mean: {:.2}\n  Range: {:.2} to {:.2}",
            count, mean, min, max
        )
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
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        writeln!(temp_file, "C,8.9").unwrap();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.data.len(), 4);
        assert_eq!(processor.calculate_mean().unwrap(), 13.85);
        assert_eq!(processor.calculate_median().unwrap(), 13.1);
        
        let filtered = processor.filter_by_threshold(12.0);
        assert_eq!(filtered.len(), 2);
        
        let freq = processor.get_frequency_distribution();
        assert_eq!(freq.get("A"), Some(&2));
    }
}