use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let valid = value > 0.0 && !category.is_empty();
            
            let record = DataRecord {
                id,
                value,
                category,
                valid,
            };
            
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.valid)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,0.0,TypeB").unwrap();
        writeln!(temp_file, "3,15.2,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 12.85).abs() < 0.01);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("TypeA").unwrap().len(), 2);
        assert_eq!(groups.get("TypeB").unwrap().len(), 1);
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
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: HashMap<String, Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, name: &str, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.insert(name.to_string(), Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), Vec<ProcessingError>> {
        let mut errors = Vec::new();

        for (rule_name, rule) in &self.validation_rules {
            if let Err(err) = rule(record) {
                errors.push(err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }
        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter()
            .map(|record| self.process_record(record))
            .collect()
    }
}

fn create_default_validation_rules() -> HashMap<String, Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>> {
    let mut rules = HashMap::new();

    rules.insert("id_positive".to_string(), Box::new(|record: &DataRecord| {
        if record.id == 0 {
            Err(ProcessingError::ValidationError("ID must be positive".to_string()))
        } else {
            Ok(())
        }
    }));

    rules.insert("name_not_empty".to_string(), Box::new(|record: &DataRecord| {
        if record.name.trim().is_empty() {
            Err(ProcessingError::ValidationError("Name cannot be empty".to_string()))
        } else {
            Ok(())
        }
    }));

    rules.insert("value_range".to_string(), Box::new(|record: &DataRecord| {
        if record.value < 0.0 || record.value > 1000.0 {
            Err(ProcessingError::ValidationError("Value must be between 0 and 1000".to_string()))
        } else {
            Ok(())
        }
    }));

    rules
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    let rules = create_default_validation_rules();
    for (name, rule) in rules {
        processor.add_validation_rule(&name, rule);
    }

    processor.add_transformation(|mut record: DataRecord| {
        record.name = record.name.to_uppercase();
        Ok(record)
    });

    processor.add_transformation(|mut record: DataRecord| {
        record.value = (record.value * 100.0).round() / 100.0;
        Ok(record)
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 500.0,
            tags: vec!["tag1".to_string()],
        };

        let result = processor.validate_record(&record);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            tags: vec![],
        };

        let result = processor.validate_record(&record);
        assert!(result.is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 123.456,
            tags: vec!["sample".to_string()],
        };

        let result = processor.process_record(record);
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed.name, "TEST");
        assert_eq!(transformed.value, 123.46);
    }
}