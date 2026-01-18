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