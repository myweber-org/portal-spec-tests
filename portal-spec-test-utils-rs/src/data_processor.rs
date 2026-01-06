
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    fn new(msg: &str) -> ProcessingError {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<DataRecord, ProcessingError> {
        if value < 0.0 || value > 1000.0 {
            return Err(ProcessingError::new("Value out of valid range (0-1000)"));
        }
        if timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<f64> {
    records
        .iter()
        .map(|r| r.value * 1.1)
        .filter(|&v| v <= 900.0)
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, 1234567890);
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.timestamp, 1234567890);
    }

    #[test]
    fn test_invalid_value_record() {
        let record = DataRecord::new(1, 1500.0, 1234567890);
        assert!(record.is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 100.0, 1000).unwrap(),
            DataRecord::new(2, 200.0, 2000).unwrap(),
            DataRecord::new(3, 900.0, 3000).unwrap(),
        ];
        let processed = process_records(&records);
        assert_eq!(processed.len(), 3);
        assert!((processed[0] - 110.0).abs() < 0.001);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 10.0, 1000).unwrap(),
            DataRecord::new(2, 20.0, 2000).unwrap(),
            DataRecord::new(3, 30.0, 3000).unwrap(),
        ];
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert!((mean - 20.0).abs() < 0.001);
        assert!((variance - 66.666).abs() < 0.1);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
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

    pub fn process_dataset(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<HashMap<String, f64>>, String> {
        let mut processed = Vec::new();

        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(validated_record) => {
                    let transformed = self.transform_record(&validated_record);
                    self.cache_record(index, &transformed);
                    processed.push(transformed);
                }
                Err(err) => {
                    return Err(format!("Validation failed at record {}: {}", index, err));
                }
            }
        }

        Ok(processed)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<HashMap<String, f64>, String> {
        for rule in &self.validation_rules {
            match record.get(&rule.field_name) {
                Some(&value) => {
                    if value < rule.min_value || value > rule.max_value {
                        return Err(format!(
                            "Field '{}' value {} out of range [{}, {}]",
                            rule.field_name, value, rule.min_value, rule.max_value
                        ));
                    }
                }
                None => {
                    if rule.required {
                        return Err(format!("Required field '{}' missing", rule.field_name));
                    }
                }
            }
        }
        Ok(record.clone())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = record.clone();
        
        for (key, value) in transformed.iter_mut() {
            if key.starts_with("normalized_") {
                *value = (*value * 100.0).round() / 100.0;
            }
        }

        transformed
    }

    fn cache_record(&mut self, index: usize, record: &HashMap<String, f64>) {
        let cache_key = format!("record_{}", index);
        let values: Vec<f64> = record.values().cloned().collect();
        self.cache.insert(cache_key, values);
    }

    pub fn get_cached_record(&self, index: usize) -> Option<&Vec<f64>> {
        let key = format!("record_{}", index);
        self.cache.get(&key)
    }

    pub fn calculate_statistics(&self, field_name: &str, dataset: &[HashMap<String, f64>]) -> Option<Statistics> {
        let values: Vec<f64> = dataset
            .iter()
            .filter_map(|record| record.get(field_name))
            .cloned()
            .collect();

        if values.is_empty() {
            return None;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        Some(Statistics {
            field_name: field_name.to_string(),
            count: values.len(),
            mean,
            variance,
            std_dev,
            min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub field_name: String,
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn validate_records(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.trim().is_empty() {
                errors.push(format!("Record {} has empty name", index));
            }

            if record.value < 0.0 {
                errors.push(format!("Record {} has negative value: {}", index, record.value));
            }

            if record.category.trim().is_empty() {
                errors.push(format!("Record {} has empty category", index));
            }
        }

        errors
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.find_max_value().map(|r| r.value);

        (count, avg, max)
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,100.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,200.0,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,150.75,Category1").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let category1_items = processor.filter_by_category("Category1");
        assert_eq!(category1_items.len(), 2);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 150.416).abs() < 0.001);

        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().name, "ItemB");

        let errors = processor.validate_records();
        assert!(errors.is_empty());

        let (count, avg, max) = processor.get_statistics();
        assert_eq!(count, 3);
        assert!(avg.is_some());
        assert!(max.is_some());
    }
}