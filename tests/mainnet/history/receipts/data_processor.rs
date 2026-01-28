use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut result = Vec::with_capacity(data.len());
        
        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < 2 {
            return data.to_vec();
        }

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;

        if range.abs() < f64::EPSILON {
            return vec![0.5; data.len()];
        }

        data.iter()
            .map(|&x| (x - min) / range)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.sqrt().abs())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_keys = self.cache.len();
        let total_values: usize = self.cache.values().map(|v| v.len()).sum();
        (total_keys, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = processor.process_dataset("normal", &data).unwrap();
        
        assert_eq!(result.len(), 5);
        assert!(result[0] >= 0.0 && result[0] <= 1.0);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let first_result = processor.process_dataset("cached", &data).unwrap();
        let second_result = processor.process_dataset("cached", &data).unwrap();
        
        assert_eq!(first_result, second_result);
        
        let (keys, values) = processor.cache_stats();
        assert_eq!(keys, 1);
        assert_eq!(values, 3);
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

    pub fn process_data(&mut self, dataset: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, String> {
        if dataset.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        let mut processed = Vec::with_capacity(dataset.len());
        
        for (index, row) in dataset.iter().enumerate() {
            match self.validate_row(row) {
                Ok(_) => {
                    let transformed = self.transform_row(row);
                    processed.push(transformed);
                    self.cache.insert(format!("row_{}", index), row.clone());
                }
                Err(e) => return Err(format!("Validation failed at row {}: {}", index, e)),
            }
        }

        Ok(processed)
    }

    fn validate_row(&self, row: &[f64]) -> Result<(), String> {
        for rule in &self.validation_rules {
            let field_index = match self.get_field_index(&rule.field_name) {
                Some(idx) => idx,
                None => continue,
            };

            if field_index >= row.len() {
                if rule.required {
                    return Err(format!("Required field '{}' missing", rule.field_name));
                }
                continue;
            }

            let value = row[field_index];
            if value < rule.min_value || value > rule.max_value {
                return Err(format!(
                    "Field '{}' value {} outside valid range [{}, {}]",
                    rule.field_name, value, rule.min_value, rule.max_value
                ));
            }
        }
        Ok(())
    }

    fn transform_row(&self, row: &[f64]) -> Vec<f64> {
        let mut transformed = Vec::with_capacity(row.len());
        
        for &value in row {
            let transformed_value = if value < 0.0 {
                value.abs()
            } else if value > 100.0 {
                100.0
            } else {
                value
            };
            transformed.push(transformed_value);
        }
        
        transformed
    }

    fn get_field_index(&self, field_name: &str) -> Option<usize> {
        match field_name {
            "temperature" => Some(0),
            "pressure" => Some(1),
            "humidity" => Some(2),
            "velocity" => Some(3),
            _ => None,
        }
    }

    pub fn get_cached_row(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        for (key, values) in &self.cache {
            if !values.is_empty() {
                let sum: f64 = values.iter().sum();
                let avg = sum / values.len() as f64;
                stats.insert(key.clone(), avg);
            }
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 150.0,
            required: true,
        });

        let test_data = vec![
            vec![25.0, 1013.0, 65.0, 10.0],
            vec![-10.0, 1000.0, 70.0, 5.0],
        ];

        let result = processor.process_data(&test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0][0], 25.0);
        assert_eq!(processed[1][0], 10.0);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: 0.0,
            max_value: 100.0,
            required: true,
        });

        let invalid_data = vec![vec![120.0, 1013.0, 65.0, 10.0]];
        let result = processor.process_data(&invalid_data);
        assert!(result.is_err());
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut map = std::collections::HashMap::new();
        for record in &self.records {
            map.entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        map
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(file, "3,ItemC,15.7,Category1").unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(file.path()).unwrap();

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_total(), 46.5);

        let valid = processor.validate_records();
        assert_eq!(valid.len(), 3);

        let groups = processor.group_by_category();
        assert_eq!(groups.get("Category1").unwrap().len(), 2);
        assert_eq!(groups.get("Category2").unwrap().len(), 1);

        let stats = processor.get_statistics();
        assert!((stats.0 - 15.5).abs() < 0.001);
    }
}