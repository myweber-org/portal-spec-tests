use std::collections::HashSet;

pub fn remove_duplicates<T: Eq + std::hash::Hash + Clone>(input: &[T]) -> Vec<T> {
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
    fn test_remove_duplicates() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(remove_duplicates(&input), expected);
    }

    #[test]
    fn test_remove_duplicates_strings() {
        let input = vec!["apple", "banana", "apple", "orange"];
        let expected = vec!["apple", "banana", "orange"];
        assert_eq!(remove_duplicates(&input), expected);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<i32> = vec![];
        let expected: Vec<i32> = vec![];
        assert_eq!(remove_duplicates(&input), expected);
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    seen: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> bool {
        let trimmed = record.trim().to_string();
        if trimmed.is_empty() {
            return false;
        }
        
        if self.seen.insert(trimmed.clone()) {
            self.records.push(trimmed);
            true
        } else {
            false
        }
    }

    pub fn get_unique_records(&self) -> &Vec<String> {
        &self.records
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

    pub fn clean_whitespace(&mut self) {
        self.records = self.records
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        self.seen.clear();
        for record in &self.records {
            self.seen.insert(record.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("test"));
        assert!(!cleaner.add_record("test"));
        assert!(cleaner.add_record("another"));
        assert_eq!(cleaner.get_unique_records().len(), 2);
    }

    #[test]
    fn test_email_validation() {
        let cleaner = DataCleaner::new();
        assert!(cleaner.validate_email("user@example.com"));
        assert!(!cleaner.validate_email("invalid-email"));
        assert!(!cleaner.validate_email("@domain.com"));
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<&str>) -> Vec<String> {
        data.iter()
            .filter(|&&item| self.deduplicate(item))
            .map(|&item| self.normalize_text(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "apple", "APPLE", "Banana", "banana"];
        let cleaned = cleaner.clean_dataset(data);
        
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
        assert_eq!(cleaner.normalize_text("TEST123"), "test123");
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
        record.age = record.age.clamp(0, 120);
        
        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "raw_data.csv";
    let output = "cleaned_data.csv";
    
    match clean_csv_data(input, output) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }
    
    Ok(())
}
use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;

pub fn filter_numeric_column(input_path: &str, output_path: &str, column_name: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(input_path)?;
    let mut wtr = Writer::from_path(output_path)?;

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    let column_index = headers.iter()
        .position(|h| h == column_name)
        .ok_or_else(|| format!("Column '{}' not found", column_name))?;

    for result in rdr.records() {
        let record = result?;
        if let Some(field) = record.get(column_index) {
            if field.parse::<f64>().is_ok() {
                wtr.write_record(&record)?;
            }
        }
    }

    wtr.flush()?;
    Ok(())
}