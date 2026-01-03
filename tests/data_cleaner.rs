use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new(data: Vec<Vec<Option<String>>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self) {
        self.data.retain(|row| {
            row.iter().all(|cell| cell.is_some())
        });
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| {
            let row_string: String = row
                .iter()
                .map(|cell| cell.as_ref().unwrap_or(&"NULL".to_string()))
                .collect::<Vec<_>>()
                .join("|");
            seen.insert(row_string)
        });
    }

    pub fn get_data(&self) -> &Vec<Vec<Option<String>>> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_null_rows() {
        let data = vec![
            vec![Some("A".to_string()), Some("B".to_string())],
            vec![Some("C".to_string()), None],
            vec![Some("E".to_string()), Some("F".to_string())],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows();
        assert_eq!(cleaner.get_data().len(), 2);
    }

    #[test]
    fn test_deduplicate() {
        let data = vec![
            vec![Some("X".to_string()), Some("Y".to_string())],
            vec![Some("X".to_string()), Some("Y".to_string())],
            vec![Some("Z".to_string()), Some("W".to_string())],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data().len(), 2);
    }
}use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, items: Vec<T>) -> Vec<T> {
        let mut result = Vec::new();
        for item in items {
            if self.seen.insert(item.clone()) {
                result.push(item);
            }
        }
        result
    }

    pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
        strings
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        let data = vec![1, 2, 2, 3, 1, 4];
        let result = cleaner.deduplicate(data);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "".to_string(),
            "  TEST  ".to_string(),
        ];
        let result = DataCleaner::normalize_strings(input);
        assert_eq!(result, vec!["hello", "world", "test"]);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;

pub fn filter_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let out_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new().from_writer(out_file);

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        if let Some(value_str) = record.get(2) {
            if let Ok(value) = value_str.parse::<f64>() {
                if value >= min_value {
                    wtr.write_record(&record)?;
                }
            }
        }
    }

    wtr.flush()?;
    Ok(())
}use regex::Regex;
use std::collections::HashSet;

pub fn clean_text(input: &str, remove_stopwords: bool) -> String {
    let stopwords: HashSet<&str> = [
        "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
    ]
    .iter()
    .cloned()
    .collect();

    let re = Regex::new(r"[^\w\s]").unwrap();
    let mut cleaned = re.replace_all(input, "").to_lowercase();

    if remove_stopwords {
        cleaned = cleaned
            .split_whitespace()
            .filter(|word| !stopwords.contains(word))
            .collect::<Vec<&str>>()
            .join(" ");
    }

    cleaned.trim().to_string()
}

pub fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let input = "Hello, World! This is a test.";
        let expected = "hello world this is a test";
        assert_eq!(clean_text(input, false), expected);
    }

    #[test]
    fn test_clean_text_with_stopwords() {
        let input = "The quick brown fox jumps over a lazy dog";
        let result = clean_text(input, true);
        assert!(!result.contains("the"));
        assert!(!result.contains("a"));
    }

    #[test]
    fn test_normalize_whitespace() {
        let input = "  Multiple   spaces   and\t tabs  ";
        let expected = "Multiple spaces and tabs";
        assert_eq!(normalize_whitespace(input), expected);
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
        let mut cleaned = Vec::new();
        for item in data {
            if self.deduplicate(item) {
                cleaned.push(self.normalize_text(item));
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
    }
}