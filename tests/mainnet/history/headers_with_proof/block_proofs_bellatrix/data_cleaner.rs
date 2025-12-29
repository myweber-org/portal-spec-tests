use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn clean_entry(&mut self, input: &str) -> Option<String> {
        let trimmed = input.trim().to_lowercase();
        
        if trimmed.is_empty() {
            return None;
        }

        if self.dedupe_set.contains(&trimmed) {
            return None;
        }

        self.dedupe_set.insert(trimmed.clone());
        Some(trimmed)
    }

    pub fn batch_clean(&mut self, inputs: Vec<&str>) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.clean_entry(input))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        
        let result1 = cleaner.clean_entry("  TEST  ");
        let result2 = cleaner.clean_entry("test");
        let result3 = cleaner.clean_entry("new data");
        
        assert_eq!(result1, Some("test".to_string()));
        assert_eq!(result2, None);
        assert_eq!(result3, Some("new data".to_string()));
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_empty_input() {
        let mut cleaner = DataCleaner::new();
        assert_eq!(cleaner.clean_entry("   "), None);
        assert_eq!(cleaner.clean_entry(""), None);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let inputs = vec!["apple", "APPLE", "banana", "  Banana  ", "cherry"];
        
        let cleaned = cleaner.batch_clean(inputs);
        
        assert_eq!(cleaned.len(), 3);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
        assert!(cleaned.contains(&"cherry".to_string()));
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn clean_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let mut writer = WriterBuilder::new().has_headers(true).from_writer(output_file);
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.age >= 18 && record.active {
            writer.serialize(&record)?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_data("input.csv", "output.csv")?;
    println!("Data cleaning completed successfully");
    Ok(())
}