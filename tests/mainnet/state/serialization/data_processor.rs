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

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if let Some(value) = parts.get(0) {
                if let Ok(num) = value.parse::<f64>() {
                    self.data.push(num);
                }
            }
            
            if let Some(category) = parts.get(1) {
                *self.frequency_map.entry(category.to_string()).or_insert(0) += 1;
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

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
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
        writeln!(temp_file, "15.5,CategoryA").unwrap();
        writeln!(temp_file, "22.3,CategoryB").unwrap();
        writeln!(temp_file, "18.7,CategoryA").unwrap();
        writeln!(temp_file, "25.1,CategoryC").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 20.4).abs() < 0.01);
        
        let median = processor.calculate_median().unwrap();
        assert!((median - 20.5).abs() < 0.01);
        
        let distribution = processor.get_frequency_distribution();
        assert_eq!(distribution.get("CategoryA"), Some(&2));
        assert_eq!(distribution.get("CategoryB"), Some(&1));
        
        let filtered = processor.filter_data(20.0);
        assert_eq!(filtered.len(), 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn process_with_filter<F>(&self, filter_fn: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if filter_fn(&columns) {
                results.push(columns);
            }
        }

        Ok(results)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        Ok(count)
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
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,apple,100").unwrap();
        writeln!(temp_file, "2,banana,200").unwrap();
        writeln!(temp_file, "3,cherry,300").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        
        let filtered = processor
            .process_with_filter(|cols| cols.len() > 1 && cols[2].parse::<i32>().unwrap_or(0) > 150)
            .unwrap();

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][1], "banana");
        assert_eq!(filtered[1][1], "cherry");

        let count = processor.count_records().unwrap();
        assert_eq!(count, 3);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    MissingField(String),
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingField(field) => write!(f, "Missing required field: {}", field),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    validation_rules: HashMap<String, Box<dyn Fn(&DataRecord) -> Result<(), DataError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    pub fn add_validation_rule<F>(&mut self, name: &str, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), DataError> + 'static,
    {
        self.validation_rules.insert(name.to_string(), Box::new(rule));
    }

    pub fn process_records(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        
        for record in &self.records {
            let category = record.category.clone();
            let entry = results.entry(category).or_insert(0.0);
            *entry += record.value;
        }
        
        results
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transformer(record.value);
        }
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::MissingField("name".to_string()));
        }
        
        for (rule_name, rule) in &self.validation_rules {
            if let Err(e) = rule(record) {
                return Err(DataError::ValidationFailed(
                    format!("Rule '{}' failed: {}", rule_name, e)
                ));
            }
        }
        
        Ok(())
    }
}

pub fn create_sample_record() -> DataRecord {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "manual".to_string());
    metadata.insert("version".to_string(), "1.0".to_string());
    
    DataRecord {
        id: 1001,
        name: "Sample Data".to_string(),
        value: 42.5,
        category: "analytics".to_string(),
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = create_sample_record();
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_process_records() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Test 1".to_string(),
            value: 10.0,
            category: "A".to_string(),
            metadata: HashMap::new(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Test 2".to_string(),
            value: 20.0,
            category: "A".to_string(),
            metadata: HashMap::new(),
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let results = processor.process_records();
        assert_eq!(results.get("A"), Some(&30.0));
    }

    #[test]
    fn test_validation_rule() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule("positive_value", |record| {
            if record.value >= 0.0 {
                Ok(())
            } else {
                Err(DataError::ValidationFailed("Value must be positive".to_string()))
            }
        });
        
        let negative_record = DataRecord {
            id: 1,
            name: "Negative".to_string(),
            value: -5.0,
            category: "test".to_string(),
            metadata: HashMap::new(),
        };
        
        assert!(processor.add_record(negative_record).is_err());
    }
}