
use std::collections::HashMap;

pub struct DataCleaner {
    pub remove_nulls: bool,
    pub normalize_strings: bool,
    pub default_values: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_nulls: true,
            normalize_strings: true,
            default_values: HashMap::new(),
        }
    }

    pub fn clean_record(&self, record: &mut HashMap<String, String>) -> Result<(), String> {
        let mut keys_to_remove = Vec::new();

        for (key, value) in record.iter_mut() {
            if value.trim().is_empty() {
                if self.remove_nulls {
                    keys_to_remove.push(key.clone());
                } else if let Some(default_val) = self.default_values.get(key) {
                    *value = default_val.clone();
                }
            } else if self.normalize_strings {
                *value = value.trim().to_lowercase();
            }
        }

        for key in keys_to_remove {
            record.remove(&key);
        }

        Ok(())
    }

    pub fn clean_dataset(&self, dataset: &mut Vec<HashMap<String, String>>) -> Result<(), String> {
        for record in dataset.iter_mut() {
            self.clean_record(record)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_record_removes_null() {
        let cleaner = DataCleaner::new();
        let mut record = HashMap::from([
            ("name".to_string(), "john".to_string()),
            ("age".to_string(), "".to_string()),
            ("city".to_string(), "New York".to_string()),
        ]);

        cleaner.clean_record(&mut record).unwrap();
        assert_eq!(record.len(), 2);
        assert!(record.contains_key("name"));
        assert!(!record.contains_key("age"));
    }

    #[test]
    fn test_clean_record_normalizes_strings() {
        let cleaner = DataCleaner::new();
        let mut record = HashMap::from([
            ("name".to_string(), "  JOHN  ".to_string()),
            ("city".to_string(), "New York".to_string()),
        ]);

        cleaner.clean_record(&mut record).unwrap();
        assert_eq!(record.get("name").unwrap(), "john");
        assert_eq!(record.get("city").unwrap(), "new york");
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn is_valid_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn clean_data<R: io::Read, W: io::Write>(
    input: R,
    output: W,
) -> Result<usize, Box<dyn Error>> {
    let mut reader = Reader::from_reader(input);
    let mut writer = Writer::from_writer(output);
    let mut valid_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        if is_valid_record(&record) {
            writer.serialize(&record)?;
            valid_count += 1;
        }
    }

    writer.flush()?;
    Ok(valid_count)
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_data = b"id,name,value,active\n1,Alice,42.5,true\n2,, -1.0,false\n3,Bob,0.0,true";
    let mut output = Vec::new();

    let valid_records = clean_data(&input_data[..], &mut output)?;
    println!("Processed {} valid records", valid_records);

    let output_str = String::from_utf8(output)?;
    println!("Cleaned data:\n{}", output_str);

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

    pub fn deduplicate(&mut self) -> usize {
        let unique_set: HashSet<String> = self.records.drain(..).collect();
        let original_count = self.records.len();
        self.records = unique_set.into_iter().collect();
        original_count - self.records.len()
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| !record.trim().is_empty() && record.len() <= 255)
            .collect()
    }

    pub fn get_clean_records(&self) -> &Vec<String> {
        &self.records
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
        
        let removed = cleaner.deduplicate();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.get_clean_records().len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("a".repeat(256));
        
        let validation = cleaner.validate_records();
        assert_eq!(validation, vec![true, false, false]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder, Trim};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .trim(Trim::All)
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    for result in csv_reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_lowercase())
            .collect();
        csv_writer.write_record(&cleaned_record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "Name, Age, City").unwrap();
        writeln!(input_file, " John , 25 , New York ").unwrap();
        writeln!(input_file, "ALICE,30,LONDON").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        clean_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        ).unwrap();

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("john,25,new york"));
        assert!(content.contains("alice,30,london"));
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

    pub fn process_batch(&mut self, items: Vec<&str>) -> Vec<String> {
        items
            .iter()
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
        let result = cleaner.process_batch(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}