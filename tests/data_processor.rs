
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
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
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            
            let record = DataRecord::new(id, value, category);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                *counts.entry(record.category.clone()).or_insert(0) += 1;
            }
        }
        
        counts
    }

    pub fn get_statistics(&self) -> Statistics {
        let valid_records = self.filter_valid();
        let count = valid_records.len();
        
        if count == 0 {
            return Statistics::empty();
        }

        let values: Vec<f64> = valid_records.iter().map(|r| r.value).collect();
        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|v| (v - avg).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();

        Statistics {
            count,
            average: avg,
            standard_deviation: std_dev,
            min: values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            max: values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub standard_deviation: f64,
    pub min: f64,
    pub max: f64,
}

impl Statistics {
    fn empty() -> Self {
        Statistics {
            count: 0,
            average: 0.0,
            standard_deviation: 0.0,
            min: 0.0,
            max: 0.0,
        }
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
        assert!(valid_record.valid);

        let invalid_value = DataRecord::new(2, -10.0, "B".to_string());
        assert!(!invalid_value.valid);

        let invalid_category = DataRecord::new(3, 15.0, "".to_string());
        assert!(!invalid_category.valid);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut csv_content = "id,value,category\n".to_string();
        csv_content.push_str("1,42.5,TypeA\n");
        csv_content.push_str("2,18.3,TypeB\n");
        csv_content.push_str("3,-5.0,TypeC\n");

        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", csv_content)?;

        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(temp_file.path())?;
        
        assert_eq!(count, 3);
        assert_eq!(processor.filter_valid().len(), 2);
        
        Ok(())
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "B".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "A".to_string()));

        let stats = processor.get_statistics();
        
        assert_eq!(stats.count, 3);
        assert_eq!(stats.average, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_category_counting() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "Alpha".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "Beta".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "Alpha".to_string()));
        processor.records.push(DataRecord::new(4, -5.0, "Gamma".to_string()));

        let counts = processor.count_by_category();
        
        assert_eq!(counts.get("Alpha"), Some(&2));
        assert_eq!(counts.get("Beta"), Some(&1));
        assert_eq!(counts.get("Gamma"), None);
    }
}