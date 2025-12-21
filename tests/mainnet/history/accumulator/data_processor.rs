
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let active_count = records.iter().filter(|r| r.active).count();
    
    (sum, average, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_data_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,-5.0,false").unwrap();
        writeln!(temp_file, "3,Test3,20.0,true").unwrap();
        
        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].name, "Test3");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, avg, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
    }
}
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

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) {
        self.data.insert(name, values);
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for rule in &self.validation_rules {
            if let Some(data) = self.data.get(&rule.field_name) {
                if rule.required && data.is_empty() {
                    errors.push(format!("Field '{}' is required but empty", rule.field_name));
                }

                for &value in data {
                    if value < rule.min_value || value > rule.max_value {
                        errors.push(format!(
                            "Value {} in field '{}' is outside valid range [{}, {}]",
                            value, rule.field_name, rule.min_value, rule.max_value
                        ));
                    }
                }
            } else if rule.required {
                errors.push(format!("Required field '{}' not found", rule.field_name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn normalize_data(&mut self, field_name: &str) -> Option<Vec<f64>> {
        if let Some(data) = self.data.get_mut(field_name) {
            if data.is_empty() {
                return None;
            }

            let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            if (max - min).abs() < f64::EPSILON {
                return Some(vec![0.0; data.len()]);
            }

            let normalized: Vec<f64> = data
                .iter()
                .map(|&x| (x - min) / (max - min))
                .collect();

            self.data.insert(field_name.to_string(), normalized.clone());
            Some(normalized)
        } else {
            None
        }
    }

    pub fn calculate_statistics(&self, field_name: &str) -> Option<DataStatistics> {
        if let Some(data) = self.data.get(field_name) {
            if data.is_empty() {
                return None;
            }

            let sum: f64 = data.iter().sum();
            let mean = sum / data.len() as f64;
            
            let variance: f64 = data
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / data.len() as f64;
            
            let std_dev = variance.sqrt();

            let sorted_data = {
                let mut sorted = data.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                sorted
            };

            let median = if data.len() % 2 == 0 {
                let mid = data.len() / 2;
                (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
            } else {
                sorted_data[data.len() / 2]
            };

            Some(DataStatistics {
                mean,
                median,
                std_dev,
                min: sorted_data[0],
                max: sorted_data[sorted_data.len() - 1],
                count: data.len(),
            })
        } else {
            None
        }
    }
}

pub struct DataStatistics {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl ValidationRule {
    pub fn new(field_name: String, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name,
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
    fn test_data_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperature".to_string(), vec![20.5, 22.3, 18.7, 25.1]);
        
        let rule = ValidationRule::new("temperature".to_string(), 15.0, 30.0, true);
        processor.add_validation_rule(rule);

        assert!(processor.validate_all().is_ok());
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("scores".to_string(), vec![10.0, 20.0, 30.0, 40.0]);
        
        let normalized = processor.normalize_data("scores").unwrap();
        assert_eq!(normalized, vec![0.0, 1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        
        let stats = processor.calculate_statistics("values").unwrap();
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.std_dev, (2.0_f64).sqrt());
    }
}