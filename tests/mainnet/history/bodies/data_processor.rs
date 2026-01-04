
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

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn normalize_data(&mut self, field_name: &str) -> Result<Vec<f64>, String> {
        if let Some(values) = self.data.get_mut(field_name) {
            if values.is_empty() {
                return Ok(Vec::new());
            }

            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            if (max - min).abs() < f64::EPSILON {
                return Ok(vec![0.0; values.len()]);
            }

            let normalized: Vec<f64> = values
                .iter()
                .map(|&v| (v - min) / (max - min))
                .collect();

            self.data.insert(field_name.to_string(), normalized.clone());
            Ok(normalized)
        } else {
            Err(format!("Field '{}' not found in dataset", field_name))
        }
    }

    pub fn calculate_statistics(&self, field_name: &str) -> Result<Statistics, String> {
        if let Some(values) = self.data.get(field_name) {
            if values.is_empty() {
                return Ok(Statistics::default());
            }

            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;

            let variance: f64 = values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;

            let sorted_values = {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                sorted
            };

            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[count as usize / 2]
            };

            Ok(Statistics {
                mean,
                median,
                variance,
                std_dev: variance.sqrt(),
                min: *values.iter().fold(&f64::INFINITY, |a, b| a.min(b)),
                max: *values.iter().fold(&f64::NEG_INFINITY, |a, b| a.max(b)),
                count: values.len(),
            })
        } else {
            Err(format!("Field '{}' not found in dataset", field_name))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Statistics {
    pub mean: f64,
    pub median: f64,
    pub variance: f64,
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
        assert_eq!(stats.variance, 2.0);
        assert_eq!(stats.std_dev, 2.0_f64.sqrt());
    }
}
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if data.is_empty() {
            return Err("No data available for statistics calculation".into());
        }

        let mut values = Vec::new();
        for record in data {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }
            
            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at column {}", column_index).into()),
            }
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();

        Ok((mean, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,55000").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&data, 0);
        
        assert!(stats.is_ok());
        let (mean, std_dev) = stats.unwrap();
        assert!((mean - 12.666).abs() < 0.001);
        assert!(std_dev > 0.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub active: bool,
}

#[derive(Debug)]
pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            self.validate_record(&record)?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn add_record(&mut self, record: Record) -> Result<(), Box<dyn Error>> {
        self.validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    pub fn remove_record(&mut self, id: u32) -> bool {
        let original_len = self.records.len();
        self.records.retain(|r| r.id != id);
        self.records.len() != original_len
    }

    pub fn get_records(&self) -> &[Record] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    fn validate_record(&self, record: &Record) -> Result<(), Box<dyn Error>> {
        if record.name.is_empty() {
            return Err("Record name cannot be empty".into());
        }

        if record.value < 0.0 {
            return Err("Record value cannot be negative".into());
        }

        if self.records.iter().any(|r| r.id == record.id) {
            return Err(format!("Duplicate record ID: {}", record.id).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();

        let record1 = Record {
            id: 1,
            name: "Test Record 1".to_string(),
            value: 100.5,
            category: "A".to_string(),
            active: true,
        };

        let record2 = Record {
            id: 2,
            name: "Test Record 2".to_string(),
            value: 200.0,
            category: "B".to_string(),
            active: false,
        };

        assert!(processor.add_record(record1.clone()).is_ok());
        assert!(processor.add_record(record2.clone()).is_ok());

        assert_eq!(processor.get_records().len(), 2);
        assert_eq!(processor.filter_by_category("A").len(), 1);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 150.25).abs() < 0.001);

        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);

        assert!(processor.remove_record(1));
        assert_eq!(processor.get_records().len(), 1);

        processor.clear();
        assert!(processor.get_records().is_empty());
    }

    #[test]
    fn test_csv_serialization() {
        let mut processor = DataProcessor::new();

        let records = vec![
            Record {
                id: 1,
                name: "Item 1".to_string(),
                value: 10.5,
                category: "Category1".to_string(),
                active: true,
            },
            Record {
                id: 2,
                name: "Item 2".to_string(),
                value: 20.0,
                category: "Category2".to_string(),
                active: false,
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        assert!(processor.save_to_csv(path).is_ok());

        let mut new_processor = DataProcessor::new();
        assert!(new_processor.load_from_csv(path).is_ok());
        assert_eq!(new_processor.get_records().len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();

        let invalid_record = Record {
            id: 1,
            name: "".to_string(),
            value: 50.0,
            category: "Test".to_string(),
            active: true,
        };

        assert!(processor.add_record(invalid_record).is_err());

        let valid_record = Record {
            id: 1,
            name: "Valid".to_string(),
            value: 50.0,
            category: "Test".to_string(),
            active: true,
        };

        assert!(processor.add_record(valid_record.clone()).is_ok());

        let duplicate_record = Record {
            id: 1,
            name: "Duplicate".to_string(),
            value: 100.0,
            category: "Test".to_string(),
            active: true,
        };

        assert!(processor.add_record(duplicate_record).is_err());

        let negative_record = Record {
            id: 2,
            name: "Negative".to_string(),
            value: -10.0,
            category: "Test".to_string(),
            active: true,
        };

        assert!(processor.add_record(negative_record).is_err());
    }
}