
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
        let mut lines = reader.lines();
        
        if self.has_header {
            lines.next();
        }
        
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }
        
        Ok(records)
    }
    
    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }
    
    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;
        
        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }
    
    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        
        let valid_record = vec!["data".to_string(), "123".to_string()];
        assert!(processor.validate_record(&valid_record));
        
        let invalid_record = vec!["".to_string(), "value".to_string()];
        assert!(!processor.validate_record(&invalid_record));
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "30.0".to_string()],
            vec!["invalid".to_string(), "40.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let average = processor.calculate_statistics(&records, 0).unwrap();
        
        assert!((average - 13.0).abs() < 0.001);
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn calculate_tax(&self, rate: f64) -> f64 {
        self.value * rate
    }
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
        let mut count = 0;
        for result in rdr.deserialize() {
            let record: Record = result?;
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }
        
        Ok(count)
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        
        for record in &self.records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn add_record(&mut self, record: Record) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn remove_record(&mut self, id: u32) -> bool {
        let original_len = self.records.len();
        self.records.retain(|r| r.id != id);
        self.records.len() < original_len
    }

    pub fn get_records(&self) -> &[Record] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -50.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = Record::new(1, "Item1".to_string(), 100.0, "CategoryA".to_string());
        let record2 = Record::new(2, "Item2".to_string(), 200.0, "CategoryB".to_string());
        
        processor.add_record(record1.clone());
        processor.add_record(record2.clone());
        
        assert_eq!(processor.get_records().len(), 2);
        assert_eq!(processor.calculate_total_value(), 300.0);
        
        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
        
        assert!(processor.remove_record(1));
        assert_eq!(processor.get_records().len(), 1);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(Record::new(1, "Test".to_string(), 123.45, "Sample".to_string()));
        
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        let count = new_processor.load_from_csv(path)?;
        
        assert_eq!(count, 1);
        assert_eq!(new_processor.calculate_total_value(), 123.45);
        
        Ok(())
    }
}