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
            if parts.len() != 4 {
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
            let valid = parts[3].parse::<bool>().unwrap_or(false);

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
        let valid_records: Vec<&DataRecord> = self.records.iter().filter(|r| r.valid).collect();
        
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

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
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
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,category_a,true").unwrap();
        writeln!(temp_file, "2,20.3,category_b,true").unwrap();
        writeln!(temp_file, "3,invalid,category_c,false").unwrap();
        writeln!(temp_file, "4,15.7,category_a,false").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.4).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 3);
        assert_eq!(groups.get("category_a").unwrap().len(), 2);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidName,
    InvalidValue,
    MissingMetadata,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidName => write!(f, "Name cannot be empty"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::MissingMetadata => write!(f, "Required metadata fields are missing"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        DataRecord {
            id,
            name,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::InvalidName);
        }
        
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if self.metadata.is_empty() {
            return Err(ValidationError::MissingMetadata);
        }
        
        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), Box<dyn Error>> {
        if multiplier <= 0.0 {
            return Err("Multiplier must be positive".into());
        }
        
        self.value *= multiplier;
        Ok(())
    }

    pub fn get_normalized_value(&self, scale: f64) -> f64 {
        self.value / scale
    }

    pub fn merge_metadata(&mut self, other: HashMap<String, String>) {
        for (key, value) in other {
            self.metadata.entry(key).or_insert(value);
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<f64>, Box<dyn Error>> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        
        record.transform_value(1.5)?;
        
        let normalized = record.get_normalized_value(100.0);
        results.push(normalized);
    }
    
    Ok(results)
}

pub fn create_sample_record() -> DataRecord {
    let mut record = DataRecord::new(1, "Sample".to_string(), 50.0);
    record.add_metadata("category".to_string(), "test".to_string());
    record.add_metadata("version".to_string(), "1.0".to_string());
    record
}