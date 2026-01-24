
use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_outliers_iqr(&self, factor: f64) -> Vec<f64> {
        if self.data.len() < 4 {
            return self.data.clone();
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75).floor() as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - factor * iqr;
        let upper_bound = q3 + factor * iqr;

        self.data
            .iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn summary_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let cleaner = DataCleaner::new(data);
        let cleaned = cleaner.remove_outliers_iqr(1.5);

        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_summary_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data);
        let stats = cleaner.summary_statistics();

        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    processed_count: usize,
    duplicates_removed: usize,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            processed_count: 0,
            duplicates_removed: 0,
        }
    }

    pub fn remove_duplicates(&mut self, data: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        
        for item in data {
            self.processed_count += 1;
            if seen.insert(item.clone()) {
                result.push(item);
            } else {
                self.duplicates_removed += 1;
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

    pub fn clean_data(&mut self, mut data: Vec<String>) -> Vec<String> {
        data = Self::normalize_strings(data);
        self.remove_duplicates(data)
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.processed_count, self.duplicates_removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        
        let result = cleaner.remove_duplicates(data);
        assert_eq!(result.len(), 3);
        assert_eq!(cleaner.stats(), (4, 1));
    }

    #[test]
    fn test_normalize_strings() {
        let data = vec![
            "  APPLE  ".to_string(),
            "Banana".to_string(),
            "CHERRY".to_string(),
        ];
        
        let result = DataCleaner::normalize_strings(data);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
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
    age: Option<u8>,
    score: Option<f64>,
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
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
        
        if record.name.is_empty() {
            record.name = "Unknown".to_string();
        }
        
        if record.age.is_none() || record.age.unwrap() > 120 {
            record.age = Some(30);
        }
        
        if record.score.is_none() {
            record.score = Some(0.0);
        } else {
            let score = record.score.unwrap();
            record.score = Some(score.clamp(0.0, 100.0));
        }
        
        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv_data() {
        let mut input_file = NamedTempFile::new().unwrap();
        let input_content = "id,name,age,score\n1,John Doe,25,85.5\n2, ,,120.0\n3,Alice,150,95.0\n4,Bob,30,\n";
        write!(input_file, "{}", input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
    }
}
use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    threshold: f64,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>, threshold: f64) -> Self {
        DataCleaner { data, threshold }
    }

    pub fn remove_outliers(&mut self) -> Vec<f64> {
        if self.data.len() < 4 {
            return self.data.clone();
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25) as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75) as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        self.data
            .iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let mean = sum / self.data.len() as f64;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.data.len() as f64;
        
        let std_dev = variance.sqrt();

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("count".to_string(), self.data.len() as f64);
        stats.insert("sum".to_string(), sum);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data, 1.5);
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data, 1.5);
        let stats = cleaner.get_statistics();
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
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
        for item in other.seen {
            self.seen.insert(item);
        }
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
        let data = vec![1, 2, 2, 3, 1, 4];
        let result = cleaner.deduplicate(data);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  Hello  ".to_string(),
            "WORLD".to_string(),
            "".to_string(),
            "  Rust  ".to_string(),
        ];
        let result = DataCleaner::normalize_strings(input);
        assert_eq!(result, vec!["hello", "world", "rust"]);
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