
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Self {
        Self { id, name, value, tags }
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".into());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".into());
        }
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        record.validate()?;
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn process_all(&mut self, multiplier: f64) {
        for record in self.records.values_mut() {
            record.transform(multiplier);
        }
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let count = self.records.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.values().map(|r| r.value).sum();
        let avg = sum / count;
        let max = self.records.values().map(|r| r.value).fold(f64::MIN, f64::max);
        let min = self.records.values().map(|r| r.value).fold(f64::MAX, f64::min);

        (avg, min, max)
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.0, vec![]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, vec![]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, "sample".to_string(), 5.0, vec!["tag1".to_string()]);
        
        assert!(processor.add_record(record).is_ok());
        processor.process_all(2.0);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 10.0);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn calculate_statistics(path: &str) -> Result<(f64, f64, usize), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut sum = 0.0;
    let mut count = 0;
    let mut max_value = f64::MIN;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.active {
            sum += record.value;
            count += 1;
            if record.value > max_value {
                max_value = record.value;
            }
        }
    }

    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    Ok((average, max_value, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_data() {
        let input_data = "id,name,value,active\n1,test1,10.5,true\n2,test2,5.0,false\n3,test3,15.0,true\n";
        let input_file = NamedTempFile::new().unwrap();
        std::fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        process_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            10.0
        ).unwrap();
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("test1"));
        assert!(!output_content.contains("test2"));
        assert!(output_content.contains("test3"));
    }
}