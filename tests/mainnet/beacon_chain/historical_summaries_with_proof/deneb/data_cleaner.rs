use std::collections::HashSet;

pub struct DataCleaner {
    pub data: Vec<Vec<Option<String>>>,
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
            let key: Vec<String> = row
                .iter()
                .map(|cell| cell.as_ref().unwrap_or(&"".to_string()).to_string())
                .collect();
            seen.insert(key)
        });
    }

    pub fn clean(&mut self) {
        self.remove_null_rows();
        self.deduplicate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let mut raw_data = vec![
            vec![Some("A".to_string()), Some("1".to_string())],
            vec![Some("B".to_string()), None],
            vec![Some("A".to_string()), Some("1".to_string())],
            vec![Some("C".to_string()), Some("3".to_string())],
        ];

        let mut cleaner = DataCleaner::new(raw_data);
        cleaner.clean();

        assert_eq!(cleaner.data.len(), 2);
        assert_eq!(cleaner.data[0][0], Some("A".to_string()));
        assert_eq!(cleaner.data[1][0], Some("C".to_string()));
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    pub records: Vec<String>,
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

    pub fn remove_duplicates(&mut self) -> usize {
        let mut unique_set = HashSet::new();
        let mut unique_records = Vec::new();
        let mut removed_count = 0;

        for record in self.records.drain(..) {
            if unique_set.insert(record.clone()) {
                unique_records.push(record);
            } else {
                removed_count += 1;
            }
        }

        self.records = unique_records;
        removed_count
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let mut valid_count = 0;
        let mut invalid_count = 0;

        for record in &self.records {
            if !record.trim().is_empty() && record.len() <= 1000 {
                valid_count += 1;
            } else {
                invalid_count += 1;
            }
        }

        (valid_count, invalid_count)
    }

    pub fn get_statistics(&self) -> (usize, usize, f64) {
        let total = self.records.len();
        let total_chars: usize = self.records.iter().map(|s| s.len()).sum();
        let avg_length = if total > 0 {
            total_chars as f64 / total as f64
        } else {
            0.0
        };

        (total, total_chars, avg_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("record1".to_string());
        cleaner.add_record("record2".to_string());
        cleaner.add_record("record1".to_string());
        cleaner.add_record("record3".to_string());
        cleaner.add_record("record2".to_string());

        let removed = cleaner.remove_duplicates();
        assert_eq!(removed, 2);
        assert_eq!(cleaner.records.len(), 3);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record(&"x".repeat(1001));

        let (valid, invalid) = cleaner.validate_records();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 2);
    }

    #[test]
    fn test_statistics() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("hello".to_string());
        cleaner.add_record("world".to_string());
        cleaner.add_record("test".to_string());

        let (total, chars, avg) = cleaner.get_statistics();
        assert_eq!(total, 3);
        assert_eq!(chars, 14);
        assert!((avg - 4.666).abs() < 0.001);
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
        
        if self.seen.contains(&trimmed) {
            return false;
        }
        
        self.seen.insert(trimmed.clone());
        self.records.push(trimmed);
        true
    }

    pub fn validate_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| record.len() > 3 && record.len() < 100)
            .collect()
    }

    pub fn deduplicate(&mut self) -> usize {
        let original_len = self.records.len();
        let mut unique = Vec::new();
        let mut new_seen = HashSet::new();
        
        for record in &self.records {
            if new_seen.insert(record.clone()) {
                unique.push(record.clone());
            }
        }
        
        self.records = unique;
        self.seen = new_seen;
        original_len - self.records.len()
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test");
        cleaner.add_record("test");
        cleaner.add_record("other");
        
        assert_eq!(cleaner.get_records().len(), 2);
        assert_eq!(cleaner.deduplicate(), 0);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("abc");
        cleaner.add_record("valid_record");
        cleaner.add_record("x");
        
        let valid = cleaner.validate_records();
        assert_eq!(valid.len(), 1);
    }
}use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    thresholds: HashMap<String, f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner {
            data,
            thresholds: HashMap::new(),
        }
    }

    pub fn calculate_iqr(&mut self) -> (f64, f64, f64, f64) {
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25) as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75) as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.thresholds.insert("lower_bound".to_string(), lower_bound);
        self.thresholds.insert("upper_bound".to_string(), upper_bound);
        self.thresholds.insert("q1".to_string(), q1);
        self.thresholds.insert("q3".to_string(), q3);

        (q1, q3, lower_bound, upper_bound)
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        let lower = self.thresholds.get("lower_bound").unwrap_or(&f64::MIN);
        let upper = self.thresholds.get("upper_bound").unwrap_or(&f64::MAX);

        self.data
            .iter()
            .filter(|&&x| x >= *lower && x <= *upper)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("min".to_string(), self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        stats.insert("max".to_string(), self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
        stats.insert("mean".to_string(), self.data.iter().sum::<f64>() / self.data.len() as f64);
        stats.insert("count".to_string(), self.data.len() as f64);
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        cleaner.calculate_iqr();
        let cleaned = cleaner.remove_outliers();
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data);
        let stats = cleaner.get_statistics();
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
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
        let trimmed = record.trim().to_lowercase();
        
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

    pub fn validate_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| record.len() >= 3 && record.len() <= 100)
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.seen.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.seen.clear();
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
        assert!(cleaner.add_record("TEST"));
        assert_eq!(cleaner.get_unique_count(), 1);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("ab");
        cleaner.add_record("valid");
        cleaner.add_record(&"x".repeat(101));
        
        let valid = cleaner.validate_records();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], "valid");
    }

    #[test]
    fn test_empty_record() {
        let mut cleaner = DataCleaner::new();
        assert!(!cleaner.add_record(""));
        assert!(!cleaner.add_record("   "));
        assert_eq!(cleaner.get_unique_count(), 0);
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
    value: f64,
    category: String,
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
        let record: Record = match result {
            Ok(rec) => rec,
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                continue;
            }
        };

        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: if record.value.is_finite() {
                record.value
            } else {
                0.0
            },
            category: record.category.to_uppercase(),
        };

        wtr.serialize(&cleaned_record)?;
    }

    wtr.flush()?;
    println!("Data cleaning completed successfully");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";

    match clean_csv_data(input_file, output_file) {
        Ok(_) => println!("Processing finished"),
        Err(e) => eprintln!("Error occurred: {}", e),
    }

    Ok(())
}