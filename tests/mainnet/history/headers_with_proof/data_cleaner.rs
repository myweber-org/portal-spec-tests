
use std::collections::HashSet;

pub struct DataCleaner {
    unique_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_items: HashSet::new(),
        }
    }

    pub fn process_string(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.unique_items.contains(&normalized) {
            return None;
        }

        self.unique_items.insert(normalized.clone());
        Some(normalized)
    }

    pub fn process_batch(&mut self, inputs: &[&str]) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_string(input))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.unique_items.len()
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cleaning() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process_string("  HELLO  "), Some("hello".to_string()));
        assert_eq!(cleaner.process_string("hello"), None);
        assert_eq!(cleaner.process_string(""), None);
        assert_eq!(cleaner.process_string("   "), None);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let inputs = vec!["Apple", "apple", "BANANA", "  banana  ", "Cherry"];
        
        let result = cleaner.process_batch(&inputs);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u32,
    active: bool,
}

fn clean_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;

        record.name = record.name.trim().to_string();

        if record.age > 120 {
            record.age = 120;
        }

        if record.name.is_empty() {
            continue;
        }

        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_data("input.csv", "cleaned_output.csv")?;
    println!("Data cleaning completed successfully.");
    Ok(())
}