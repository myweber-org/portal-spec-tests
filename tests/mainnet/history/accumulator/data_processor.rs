
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
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
            data: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_dataset(&mut self, name: &str, values: Vec<f64>) -> Result<(), String> {
        if name.is_empty() {
            return Err("Dataset name cannot be empty".to_string());
        }
        
        if self.data.contains_key(name) {
            return Err(format!("Dataset '{}' already exists", name));
        }
        
        self.data.insert(name.to_string(), values);
        Ok(())
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_data(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for rule in &self.validation_rules {
            if let Some(data_values) = self.data.get(&rule.field_name) {
                if rule.required && data_values.is_empty() {
                    errors.push(format!("Field '{}' is required but empty", rule.field_name));
                    continue;
                }
                
                for (index, &value) in data_values.iter().enumerate() {
                    if value < rule.min_value || value > rule.max_value {
                        errors.push(format!(
                            "Value {} at index {} in field '{}' is outside valid range [{}, {}]",
                            value, index, rule.field_name, rule.min_value, rule.max_value
                        ));
                    }
                }
            } else if rule.required {
                errors.push(format!("Required field '{}' not found in dataset", rule.field_name));
            }
        }
        
        errors
    }

    pub fn calculate_statistics(&self, field_name: &str) -> Option<Statistics> {
        self.data.get(field_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = if count > 0 { sum / count as f64 } else { 0.0 };
            
            let variance = if count > 1 {
                let squared_diff_sum: f64 = values.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum();
                squared_diff_sum / (count - 1) as f64
            } else {
                0.0
            };
            
            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev: variance.sqrt(),
            }
        })
    }

    pub fn normalize_data(&mut self, field_name: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(field_name) {
            if values.is_empty() {
                return Ok(());
            }
            
            let stats = self.calculate_statistics(field_name).unwrap();
            
            if stats.std_dev > 0.0 {
                for value in values.iter_mut() {
                    *value = (*value - stats.mean) / stats.std_dev;
                }
            }
            Ok(())
        } else {
            Err(format!("Field '{}' not found in dataset", field_name))
        }
    }

    pub fn get_data(&self, field_name: &str) -> Option<&Vec<f64>> {
        self.data.get(field_name)
    }

    pub fn list_datasets(&self) -> Vec<&String> {
        self.data.keys().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
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
    fn test_add_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("temperatures", vec![20.5, 22.1, 19.8, 23.4]);
        assert!(result.is_ok());
        assert_eq!(processor.list_datasets().len(), 1);
    }

    #[test]
    fn test_duplicate_dataset() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperatures", vec![20.5]).unwrap();
        let result = processor.add_dataset("temperatures", vec![22.1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("scores", vec![85.0, 92.0, 78.0, 105.0]).unwrap();
        
        let rule = ValidationRule::new("scores", 0.0, 100.0, true);
        processor.add_validation_rule(rule);
        
        let errors = processor.validate_data();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("outside valid range"));
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values", vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("values").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("data", vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        processor.normalize_data("data").unwrap();
        let normalized_data = processor.get_data("data").unwrap();
        
        let stats = processor.calculate_statistics("data").unwrap();
        assert!(stats.mean.abs() < 1e-10);
        assert!((stats.std_dev - 1.0).abs() < 1e-10);
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Self {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor;

impl DataProcessor {
    pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        let mut records = Vec::new();

        for result in reader.deserialize() {
            let record: Record = result?;
            record.validate()?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn save_to_csv<P: AsRef<Path>>(records: &[Record], path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(path)?;

        for record in records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn filter_active(records: &[Record]) -> Vec<&Record> {
        records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_total(records: &[Record]) -> f64 {
        records.iter().map(|r| r.value).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_name = Record::new(2, "".to_string(), 50.0, false);
        assert!(invalid_name.validate().is_err());

        let invalid_value = Record::new(3, "Test".to_string(), -10.0, true);
        assert!(invalid_value.validate().is_err());
    }

    #[test]
    fn test_csv_roundtrip() -> Result<(), Box<dyn Error>> {
        let records = vec![
            Record::new(1, "Alpha".to_string(), 10.5, true),
            Record::new(2, "Beta".to_string(), 20.0, false),
            Record::new(3, "Gamma".to_string(), 30.75, true),
        ];

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        DataProcessor::save_to_csv(&records, path)?;
        let loaded = DataProcessor::load_from_csv(path)?;

        assert_eq!(records.len(), loaded.len());
        assert_eq!(records[0].name, loaded[0].name);
        assert_eq!(records[1].value, loaded[1].value);

        Ok(())
    }

    #[test]
    fn test_filter_and_calculate() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, false),
            Record::new(3, "C".to_string(), 30.0, true),
        ];

        let active = DataProcessor::filter_active(&records);
        assert_eq!(active.len(), 2);

        let total = DataProcessor::calculate_total(&records);
        assert_eq!(total, 60.0);
    }
}