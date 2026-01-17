use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    records: Vec<String>,
    unique_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            unique_set: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> Result<(), Box<dyn Error>> {
        let trimmed = record.trim();
        
        if trimmed.is_empty() {
            return Err("Empty record not allowed".into());
        }

        if trimmed.len() > 1000 {
            return Err("Record exceeds maximum length".into());
        }

        if self.unique_set.contains(trimmed) {
            return Err("Duplicate record detected".into());
        }

        self.unique_set.insert(trimmed.to_string());
        self.records.push(trimmed.to_string());
        Ok(())
    }

    pub fn validate_email(&self, email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2 && 
        !parts[0].is_empty() && 
        !domain_parts.iter().any(|part| part.is_empty())
    }

    pub fn deduplicate(&mut self) -> usize {
        let original_count = self.records.len();
        self.unique_set.clear();
        
        let mut deduped = Vec::new();
        for record in &self.records {
            if !self.unique_set.contains(record) {
                self.unique_set.insert(record.clone());
                deduped.push(record.clone());
            }
        }
        
        self.records = deduped;
        original_count - self.records.len()
    }

    pub fn get_records(&self) -> &[String] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.unique_set.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_record() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("test@example.com").is_ok());
        assert_eq!(cleaner.get_records().len(), 1);
    }

    #[test]
    fn test_duplicate_rejection() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("duplicate").unwrap();
        assert!(cleaner.add_record("duplicate").is_err());
    }

    #[test]
    fn test_email_validation() {
        let cleaner = DataCleaner::new();
        assert!(cleaner.validate_email("user@domain.com"));
        assert!(!cleaner.validate_email("invalid-email"));
    }
}
use std::collections::HashSet;

pub fn clean_data<T: Eq + std::hash::Hash + Clone>(data: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in data {
        if !seen.contains(item) {
            seen.insert(item.clone());
            result.push(item.clone());
        }
    }
    
    result
}

pub fn filter_by_predicate<T, F>(data: &[T], predicate: F) -> Vec<T>
where
    T: Clone,
    F: Fn(&T) -> bool,
{
    data.iter()
        .filter(|item| predicate(item))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data_removes_duplicates() {
        let data = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let cleaned = clean_data(&data);
        assert_eq!(cleaned, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_filter_by_predicate() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let filtered = filter_by_predicate(&data, |&x| x % 2 == 0);
        assert_eq!(filtered, vec![2, 4, 6, 8, 10]);
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
    age: u32,
    email: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.email = record.email.trim().to_lowercase();
        
        if record.age > 120 {
            eprintln!("Warning: Invalid age {} for record {}", record.age, record.id);
            record.age = 0;
        }
        
        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    println!("Data cleaning completed successfully");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "cleaned_data.csv";
    
    match clean_csv_data(input_file, output_file) {
        Ok(_) => println!("Processing finished"),
        Err(e) => eprintln!("Error occurred: {}", e),
    }
    
    Ok(())
}use std::collections::HashSet;
use std::iter::FromIterator;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) {
        self.records.push(record.to_string());
    }

    pub fn deduplicate(&self) -> Vec<String> {
        let unique_set: HashSet<String> = HashSet::from_iter(self.records.iter().cloned());
        let mut result: Vec<String> = unique_set.into_iter().collect();
        result.sort();
        result
    }

    pub fn normalize_whitespace(&self) -> Vec<String> {
        self.records
            .iter()
            .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
            .collect()
    }

    pub fn to_lowercase(&self) -> Vec<String> {
        self.records.iter().map(|s| s.to_lowercase()).collect()
    }

    pub fn clean_pipeline(&self) -> Vec<String> {
        let normalized = self.normalize_whitespace();
        let lowercased: Vec<String> = normalized.iter().map(|s| s.to_lowercase()).collect();
        let unique_set: HashSet<String> = HashSet::from_iter(lowercased.into_iter());
        let mut result: Vec<String> = unique_set.into_iter().collect();
        result.sort();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("apple");
        cleaner.add_record("banana");
        cleaner.add_record("apple");
        cleaner.add_record("cherry");
        
        let result = cleaner.deduplicate();
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_normalize_whitespace() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  multiple   spaces   here  ");
        cleaner.add_record("tabs\tand\nnewlines");
        
        let result = cleaner.normalize_whitespace();
        assert_eq!(result[0], "multiple spaces here");
        assert_eq!(result[1], "tabs and newlines");
    }

    #[test]
    fn test_clean_pipeline() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  Apple  ");
        cleaner.add_record("apple");
        cleaner.add_record("BANANA  ");
        cleaner.add_record("  banana  ");
        
        let result = cleaner.clean_pipeline();
        assert_eq!(result, vec!["apple", "banana"]);
    }
}