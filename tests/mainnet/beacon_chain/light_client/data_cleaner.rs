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
}use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
    normalized: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
            normalized: Vec::new(),
        }
    }

    pub fn add(&mut self, item: T) -> bool {
        if self.seen.insert(item.clone()) {
            self.normalized.push(item);
            true
        } else {
            false
        }
    }

    pub fn normalize(&mut self) -> &[T] {
        self.normalized.sort();
        &self.normalized
    }

    pub fn deduplicate(&mut self) -> Vec<T> {
        let mut result = Vec::new();
        std::mem::swap(&mut result, &mut self.normalized);
        self.seen.clear();
        result
    }

    pub fn is_empty(&self) -> bool {
        self.normalized.is_empty()
    }

    pub fn len(&self) -> usize {
        self.normalized.len()
    }
}

impl<T> Default for DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add("apple"));
        assert!(cleaner.add("banana"));
        assert!(!cleaner.add("apple"));
        assert_eq!(cleaner.len(), 2);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add("zebra");
        cleaner.add("apple");
        cleaner.add("banana");
        
        let normalized = cleaner.normalize();
        assert_eq!(normalized, &["apple", "banana", "zebra"]);
    }

    #[test]
    fn test_deduplicate_output() {
        let mut cleaner = DataCleaner::new();
        cleaner.add("duplicate");
        cleaner.add("duplicate");
        cleaner.add("unique");
        
        let result = cleaner.deduplicate();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"duplicate"));
        assert!(result.contains(&"unique"));
    }
}