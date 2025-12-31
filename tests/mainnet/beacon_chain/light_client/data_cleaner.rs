use std::collections::HashSet;
use std::io::{self, BufRead, Write};

fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = String::new();
    
    println!("Enter data (press Ctrl+D when finished):");
    for line in stdin.lock().lines() {
        input.push_str(&line?);
        input.push('\n');
    }
    
    let cleaned = clean_data(&input);
    
    let mut output_file = std::fs::File::create("cleaned_output.txt")?;
    output_file.write_all(cleaned.as_bytes())?;
    
    println!("Data cleaned and saved to cleaned_output.txt");
    println!("Original lines: {}", input.lines().count());
    println!("Unique lines: {}", cleaned.lines().count());
    
    Ok(())
}
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

    pub fn add_item(&mut self, item: &str) -> bool {
        let normalized = Self::normalize_string(item);
        self.unique_items.insert(normalized)
    }

    pub fn get_unique_items(&self) -> Vec<String> {
        let mut items: Vec<String> = self.unique_items.iter().cloned().collect();
        items.sort();
        items
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }

    pub fn count(&self) -> usize {
        self.unique_items.len()
    }

    fn normalize_string(s: &str) -> String {
        s.trim().to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_detection() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_item("Apple"));
        assert!(!cleaner.add_item("apple"));
        assert!(!cleaner.add_item("  APPLE  "));
        assert_eq!(cleaner.count(), 1);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_item("  Banana  ");
        cleaner.add_item("BANANA");
        assert_eq!(cleaner.count(), 1);
    }

    #[test]
    fn test_get_sorted_items() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_item("Orange");
        cleaner.add_item("Apple");
        cleaner.add_item("Banana");
        
        let items = cleaner.get_unique_items();
        assert_eq!(items, vec!["apple", "banana", "orange"]);
    }
}use std::collections::HashSet;

pub fn clean_and_sort_data(data: &[String]) -> Vec<String> {
    let unique_set: HashSet<_> = data.iter().collect();
    let mut unique_vec: Vec<String> = unique_set.into_iter().cloned().collect();
    unique_vec.sort();
    unique_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort() {
        let input = vec![
            "zebra".to_string(),
            "apple".to_string(),
            "zebra".to_string(),
            "banana".to_string(),
        ];
        let result = clean_and_sort_data(&input);
        assert_eq!(result, vec!["apple", "banana", "zebra"]);
    }
}