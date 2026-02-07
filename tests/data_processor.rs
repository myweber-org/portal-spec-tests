
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field {} contains invalid values", rule.field_name));
            }

            if let Some(&value) = data.iter().find(|&&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!("Value {} out of range for field {}", value, rule.field_name));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&x| x * 2.0)
            .filter(|&x| x > 0.0)
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());

        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            Statistics {
                mean,
                variance,
                count: data.len(),
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            }
        })
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("temperature", -50.0, 100.0, true));
        
        let data = vec![10.0, 20.0, 30.0, 40.0];
        let result = processor.process_dataset("test_data", data);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![20.0, 40.0, 60.0, 80.0]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("pressure", 0.0, 100.0, true));
        
        let data = vec![50.0, 150.0, 30.0];
        let result = processor.process_dataset("invalid_data", data);
        
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, String>>,
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
        
        if let Some(header_line) = lines.next() {
            let headers: Vec<String> = header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            for line in lines {
                let line_content = line?;
                let values: Vec<&str> = line_content.split(',').map(|s| s.trim()).collect();
                
                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        record.insert(header.clone(), values[i].to_string());
                    }
                    self.records.push(record);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self, column_name: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;
        
        for record in &self.records {
            if let Some(value_str) = record.get(column_name) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    pub fn count_unique_values(&self, column_name: &str) -> usize {
        let mut unique_values = std::collections::HashSet::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_name) {
                unique_values.insert(value.clone());
            }
        }
        
        unique_values.len()
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_column_names(&self) -> Vec<String> {
        if let Some(first_record) = self.records.first() {
            first_record.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,55000").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_csv(file_path);
        assert!(result.is_ok());
        
        assert_eq!(processor.get_record_count(), 3);
        
        let avg_age = processor.calculate_average("age");
        assert_eq!(avg_age, Some(30.0));
        
        let unique_names = processor.count_unique_values("name");
        assert_eq!(unique_names, 3);
        
        let filtered = processor.filter_records(|record| {
            record.get("age").and_then(|a| a.parse::<i32>().ok()) > Some(30)
        });
        assert_eq!(filtered.len(), 1);
    }
}