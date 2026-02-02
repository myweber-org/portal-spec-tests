use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;
    
    let mut seen_ids = HashSet::new();
    let mut cleaned_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if seen_ids.contains(&record.id) {
            continue;
        }
        
        seen_ids.insert(record.id);
        
        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: if record.value.is_nan() { 0.0 } else { record.value },
            category: record.category.to_uppercase(),
        };
        
        writer.serialize(&cleaned_record)?;
        cleaned_count += 1;
    }
    
    writer.flush()?;
    println!("Cleaned {} records, removed duplicates", cleaned_count);
    
    Ok(())
}

fn main() {
    if let Err(e) = clean_csv("input.csv", "output.csv") {
        eprintln!("Error cleaning CSV: {}", e);
        std::process::exit(1);
    }
}
use std::collections::HashSet;

pub fn clean_and_sort_data(data: Vec<String>) -> Vec<String> {
    let unique_data: HashSet<String> = data.into_iter().collect();
    let mut sorted_data: Vec<String> = unique_data.into_iter().collect();
    sorted_data.sort();
    sorted_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort_data() {
        let input = vec![
            "banana".to_string(),
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
            "apple".to_string(),
        ];
        
        let result = clean_and_sort_data(input);
        let expected = vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];
        
        assert_eq!(result, expected);
    }
}
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

    pub fn normalize_text(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    pub fn is_duplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        !self.dedupe_set.insert(normalized)
    }

    pub fn clean_dataset(&mut self, data: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        
        for item in data {
            if !self.is_duplicate(&item) {
                cleaned.push(item);
            }
        }
        
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        let result = cleaner.normalize_text("  Hello World!  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "Banana".to_string(),
            "APPLE".to_string(),
        ];
        
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }
}use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T> {
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_duplicates(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item));
        self
    }

    pub fn filter<F>(&mut self, predicate: F) -> &mut Self
    where
        F: Fn(&T) -> bool,
    {
        self.data.retain(predicate);
        self
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

impl<T: PartialEq> DataCleaner<T> {
    pub fn remove_null_values(&mut self) -> &mut Self {
        self.filter(|item| item != &None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new(vec![1, 2, 2, 3, 4, 4, 5]);
        cleaner.remove_duplicates();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_filter() {
        let mut cleaner = DataCleaner::new(vec![1, 2, 3, 4, 5]);
        cleaner.filter(|&x| x % 2 == 0);
        assert_eq!(cleaner.get_data(), &vec![2, 4]);
    }
}