use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn filter_by_length(strings: Vec<String>, min_length: usize) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| s.len() >= min_length)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 1, 4];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_filter_by_length() {
        let input = vec!["a".to_string(), "ab".to_string(), "abc".to_string()];
        let result = filter_by_length(input, 2);
        assert_eq!(result, vec!["ab".to_string(), "abc".to_string()]);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct CleanRecord {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
        let raw_record: CleanRecord = result?;
        
        let cleaned_record = CleanRecord {
            id: raw_record.id,
            name: raw_record.name.trim().to_string(),
            age: raw_record.age.clamp(0, 120),
            active: raw_record.active,
        };

        wtr.serialize(cleaned_record)?;
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
}use std::collections::HashMap;

pub struct DataCleaner {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            filters: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn clean_data(&self, data: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        data.into_iter()
            .filter(|entry| self.filters.iter().all(|filter| filter(entry)))
            .collect()
    }
}

pub fn create_default_cleaner() -> DataCleaner {
    let mut cleaner = DataCleaner::new();
    
    cleaner.add_filter(|entry| {
        entry.contains_key("id") && !entry.get("id").unwrap().is_empty()
    });

    cleaner.add_filter(|entry| {
        entry.get("timestamp")
            .and_then(|ts| ts.parse::<u64>().ok())
            .map_or(false, |timestamp| timestamp > 0)
    });

    cleaner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_filters_invalid_data() {
        let cleaner = create_default_cleaner();
        
        let mut valid_entry = HashMap::new();
        valid_entry.insert("id".to_string(), "123".to_string());
        valid_entry.insert("timestamp".to_string(), "1672531200".to_string());
        
        let mut invalid_entry = HashMap::new();
        invalid_entry.insert("id".to_string(), "".to_string());
        invalid_entry.insert("timestamp".to_string(), "0".to_string());
        
        let data = vec![valid_entry.clone(), invalid_entry];
        let cleaned = cleaner.clean_data(data);
        
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].get("id").unwrap(), "123");
    }
}