
use std::collections::HashSet;

pub struct DataCleaner {
    processed_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            processed_items: HashSet::new(),
        }
    }

    pub fn normalize_string(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn is_duplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_string(item);
        if self.processed_items.contains(&normalized) {
            true
        } else {
            self.processed_items.insert(normalized);
            false
        }
    }

    pub fn clean_data(&mut self, data: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        
        for item in data {
            if !self.is_duplicate(&item) {
                cleaned.push(item);
            }
        }
        
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.processed_items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_string() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_duplicate_detection() {
        let mut cleaner = DataCleaner::new();
        assert!(!cleaner.is_duplicate("test"));
        assert!(cleaner.is_duplicate("TEST"));
        assert!(cleaner.is_duplicate("  test  "));
    }

    #[test]
    fn test_clean_data() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "APPLE".to_string(),
            "banana".to_string(),
            "  banana  ".to_string(),
            "cherry".to_string(),
        ];
        
        let cleaned = cleaner.clean_data(data);
        assert_eq!(cleaned.len(), 3);
        assert_eq!(cleaner.get_unique_count(), 3);
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
    category: String,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_lowercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        writer.serialize(&record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() &&
    record.value >= 0.0 &&
    !record.category.is_empty()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    clean_csv(input_file, output_file)?;
    
    let validation_file = File::open(output_file)?;
    let mut validation_reader = Reader::from_reader(validation_file);
    let mut valid_count = 0;
    let mut total_count = 0;
    
    for result in validation_reader.deserialize() {
        let record: Record = result?;
        total_count += 1;
        
        if validate_record(&record) {
            valid_count += 1;
        }
    }
    
    println!("Processed {} records", total_count);
    println!("Valid records: {}", valid_count);
    
    Ok(())
}