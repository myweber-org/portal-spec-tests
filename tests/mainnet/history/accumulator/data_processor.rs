
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: ValidationRules,
}

pub struct ValidationRules {
    min_value: f64,
    max_value: f64,
    required_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if !self.validation_rules.required_keys.contains(&key) {
            return Err(format!("Key '{}' is not in required keys list", key));
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!("Value {} is outside allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[count as usize / 2]
            };

            Statistics {
                mean,
                median,
                std_dev,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                count: values.len(),
            }
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<Vec<f64>, String> {
        if let Some(values) = self.data.get(key) {
            let stats = self.calculate_statistics(key).unwrap();
            let normalized: Vec<f64> = values.iter()
                .map(|&x| (x - stats.mean) / stats.std_dev)
                .collect();
            
            self.data.insert(key.to_string(), normalized.clone());
            Ok(normalized)
        } else {
            Err(format!("Key '{}' not found in dataset", key))
        }
    }

    pub fn merge_datasets(&self, other: &DataProcessor) -> DataProcessor {
        let mut merged_data = self.data.clone();
        
        for (key, values) in &other.data {
            merged_data.insert(key.clone(), values.clone());
        }

        DataProcessor {
            data: merged_data,
            validation_rules: self.validation_rules.clone(),
        }
    }
}

pub struct Statistics {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl ValidationRules {
    pub fn new(min_value: f64, max_value: f64, required_keys: Vec<String>) -> Self {
        ValidationRules {
            min_value,
            max_value,
            required_keys,
        }
    }
}

impl Clone for ValidationRules {
    fn clone(&self) -> Self {
        ValidationRules {
            min_value: self.min_value,
            max_value: self.max_value,
            required_keys: self.required_keys.clone(),
        }
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord {
            id,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_number + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].to_string();

        let record = DataRecord::new(id, value, category);
        if !record.is_valid() {
            return Err(format!("Invalid data at line {}", line_number + 1).into());
        }

        records.push(record);
    }

    Ok(records)
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord::new(0, 42.5, "test".to_string());
        assert!(!record1.is_valid());

        let record2 = DataRecord::new(1, -1.0, "test".to_string());
        assert!(!record2.is_valid());

        let record3 = DataRecord::new(1, 42.5, "".to_string());
        assert!(!record3.is_valid());
    }

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.0,category_b").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "3,30.5,category_c").unwrap();

        let records = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].value, 20.0);
        assert_eq!(records[2].category, "category_c");
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            DataRecord::new(1, 10.0, "a".to_string()),
            DataRecord::new(2, 20.0, "b".to_string()),
            DataRecord::new(3, 30.0, "c".to_string()),
        ];

        let avg = calculate_average(&records).unwrap();
        assert_eq!(avg, 20.0);

        let empty_avg = calculate_average(&[]);
        assert!(empty_avg.is_none());
    }
}