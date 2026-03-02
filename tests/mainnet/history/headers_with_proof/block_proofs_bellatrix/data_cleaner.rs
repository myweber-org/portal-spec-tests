use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

pub fn process_from_stdin() -> io::Result<()> {
    let stdin = io::stdin();
    let mut buffer = String::new();
    
    for line in stdin.lock().lines() {
        buffer.push_str(&line?);
        buffer.push('\n');
    }
    
    let cleaned = clean_data(&buffer);
    io::stdout().write_all(cleaned.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\napple\nbanana";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + Hash + Clone,
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

    pub fn merge_cleaners(mut self, other: DataCleaner<T>) -> Self {
        self.seen.extend(other.seen);
        self
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
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        let data = vec![1, 2, 2, 3, 4, 4, 5];
        let result = cleaner.deduplicate(data);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "".to_string(),
            "  ".to_string(),
        ];
        let result = DataCleaner::normalize_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_merge_cleaners() {
        let mut cleaner1 = DataCleaner::new();
        cleaner1.deduplicate(vec![1, 2, 3]);

        let mut cleaner2 = DataCleaner::new();
        cleaner2.deduplicate(vec![3, 4, 5]);

        let merged = cleaner1.merge_cleaners(cleaner2);
        let result = merged.deduplicate(vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(result, vec![6]);
    }
}
use csv::{ReaderBuilder, WriterBuilder};
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

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.age > 120 {
        return Err("Age must be reasonable".to_string());
    }
    Ok(())
}

pub fn clean_csv_data(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        match validate_record(&record) {
            Ok(_) => {
                wtr.serialize(&record)?;
                valid_count += 1;
            }
            Err(err) => {
                eprintln!("Invalid record {}: {}", record.id, err);
                invalid_count += 1;
            }
        }
    }

    wtr.flush()?;
    println!("Cleaning complete. Valid: {}, Invalid: {}", valid_count, invalid_count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "John".to_string(),
            age: 30,
            active: true,
        };
        assert!(validate_record(&valid_record).is_ok());

        let invalid_record = Record {
            id: 2,
            name: "   ".to_string(),
            age: 30,
            active: true,
        };
        assert!(validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_clean_csv_data() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,age,active\n1,Alice,25,true\n2,,30,false\n3,Bob,150,true";
        
        let input_file = NamedTempFile::new()?;
        std::fs::write(&input_file, input_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        clean_csv_data(input_file.path(), output_file.path())?;
        
        let output_content = std::fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("Alice"));
        assert!(!output_content.contains("Bob"));
        
        Ok(())
    }
}use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T: Clone + Eq + std::hash::Hash> DataCleaner<T> {
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_values(&mut self, null_value: &T) {
        self.data.retain(|item| item != null_value);
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_null_values() {
        let mut cleaner = DataCleaner::new(vec![Some(1), None, Some(2), None, Some(3)]);
        cleaner.remove_null_values(&None);
        assert_eq!(cleaner.get_data(), &vec![Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new(vec![1, 2, 2, 3, 3, 3, 4]);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3, 4]);
    }
}