
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::DuplicateTag => write!(f, "Tags contain duplicates"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        if name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = HashMap::new();
        for tag in &tags {
            if seen_tags.contains_key(tag) {
                return Err(ValidationError::DuplicateTag);
            }
            seen_tags.insert(tag.clone(), true);
        }
        
        Ok(DataRecord {
            id,
            name,
            value,
            tags,
        })
    }
    
    pub fn transform(&self, multiplier: f64) -> Self {
        DataRecord {
            id: self.id,
            name: self.name.clone(),
            value: self.value * multiplier,
            tags: self.tags.clone(),
        }
    }
    
    pub fn add_tag(&mut self, tag: String) -> Result<(), ValidationError> {
        if self.tags.contains(&tag) {
            return Err(ValidationError::DuplicateTag);
        }
        self.tags.push(tag);
        Ok(())
    }
    
    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value * 100.0;
        let tag_bonus = self.tags.len() as f64 * 10.0;
        base_score + tag_bonus
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|r| r.value > 50.0)
        .map(|r| r.transform(1.1))
        .collect()
}

pub fn aggregate_values(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut result = HashMap::new();
    
    for record in records {
        let entry = result.entry(record.name.clone()).or_insert(0.0);
        *entry += record.value;
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        );
        
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Record");
        assert_eq!(record.value, 100.0);
        assert_eq!(record.tags.len(), 2);
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            100.0,
            vec![]
        );
        
        assert!(matches!(record, Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_calculate_score() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        ).unwrap();
        
        let score = record.calculate_score();
        assert_eq!(score, 100.0 * 100.0 + 2.0 * 10.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.timestamp.is_empty()
    }
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

            let name = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value > threshold)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|record| record.id == target_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,item1,10.5,2024-01-01").unwrap();
        writeln!(temp_file, "2,item2,20.0,2024-01-02").unwrap();
        writeln!(temp_file, "3,item3,invalid,2024-01-03").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.get_records().len(), 2);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "a".to_string(), 5.0, "t1".to_string()));
        processor.records.push(DataRecord::new(2, "b".to_string(), 15.0, "t2".to_string()));
        processor.records.push(DataRecord::new(3, "c".to_string(), 25.0, "t3".to_string()));

        let filtered = processor.filter_by_value(10.0);
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average();
        assert_eq!(average, Some(15.0));
    }
}