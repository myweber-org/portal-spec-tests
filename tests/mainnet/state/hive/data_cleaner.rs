use std::collections::HashSet;

pub fn deduplicate_vector<T: Eq + std::hash::Hash + Clone>(input: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in input {
        if seen.insert(item) {
            result.push(item.clone());
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_vector() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(deduplicate_vector(&input), expected);
    }

    #[test]
    fn test_deduplicate_vector_strings() {
        let input = vec!["apple", "banana", "apple", "orange"];
        let expected = vec!["apple", "banana", "orange"];
        assert_eq!(deduplicate_vector(&input), expected);
    }

    #[test]
    fn test_deduplicate_vector_empty() {
        let input: Vec<i32> = vec![];
        let expected: Vec<i32> = vec![];
        assert_eq!(deduplicate_vector(&input), expected);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: Option<u8>,
    active: bool,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                continue;
            }
        };

        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            age: record.age.filter(|&a| a <= 120),
            active: record.active,
        };

        wtr.serialize(cleaned_record)?;
    }

    wtr.flush()?;
    println!("Successfully cleaned data from {} to {}", input_path, output_path);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_csv("input.csv", "output.csv")?;
    Ok(())
}use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                unique_records.push(record);
            }
        }

        self.records = unique_records.clone();
        unique_records
    }

    pub fn validate_records(&self) -> Result<(), String> {
        for (index, record) in self.records.iter().enumerate() {
            if record.trim().is_empty() {
                return Err(format!("Empty record found at index {}", index));
            }
            if record.len() > 1000 {
                return Err(format!("Record too long at index {}", index));
            }
        }
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());

        let deduped = cleaner.deduplicate();
        assert_eq!(deduped.len(), 2);
        assert_eq!(cleaner.get_record_count(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid record".to_string());
        assert!(cleaner.validate_records().is_ok());

        cleaner.add_record("".to_string());
        assert!(cleaner.validate_records().is_err());
    }
}