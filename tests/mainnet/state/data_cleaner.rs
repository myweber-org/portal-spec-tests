use csv::ReaderBuilder;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".to_string());
    }
    Ok(())
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let mut valid_records = Vec::new();
    let mut error_count = 0;

    for result in rdr.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Parsing error: {}", e);
                error_count += 1;
                continue;
            }
        };

        match validate_record(&record) {
            Ok(_) => valid_records.push(record),
            Err(e) => {
                eprintln!("Validation error for ID {}: {}", record.id, e);
                error_count += 1;
            }
        }
    }

    println!("Processed {} records", valid_records.len() + error_count);
    println!("Valid records: {}", valid_records.len());
    println!("Invalid records: {}", error_count);

    if !valid_records.is_empty() {
        let output_file = File::create(output_path)?;
        let mut wtr = csv::Writer::from_writer(output_file);
        
        for record in valid_records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        println!("Cleaned data written to {}", output_path);
    }

    Ok(())
}
use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn normalize<F>(&mut self, transform: F) -> &mut Self
    where
        F: Fn(&T) -> T,
    {
        for item in &mut self.data {
            *item = transform(item);
        }
        self
    }

    pub fn filter<F>(&mut self, predicate: F) -> &mut Self
    where
        F: Fn(&T) -> bool,
    {
        self.data.retain(|item| predicate(item));
        self
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

pub fn clean_string_data(strings: Vec<String>) -> Vec<String> {
    let mut cleaner = DataCleaner::new(strings);
    cleaner
        .deduplicate()
        .normalize(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .into_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let data = vec![1, 2, 2, 3, 3, 3];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3]);
    }

    #[test]
    fn test_string_cleaning() {
        let strings = vec![
            "  HELLO  ".to_string(),
            "hello".to_string(),
            "".to_string(),
            "  WORLD  ".to_string(),
        ];
        let result = clean_string_data(strings);
        assert_eq!(result, vec!["hello", "world"]);
    }
}
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    sorted_lines.join("\n")
}

pub fn process_stream() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    
    let mut input_data = String::new();
    for line in stdin.lock().lines() {
        input_data.push_str(&line?);
        input_data.push('\n');
    }
    
    let cleaned = clean_data(&input_data);
    write!(output, "{}", cleaned)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = "banana\napple\nbanana\ncherry\napple";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".to_string());
    }
    Ok(())
}

fn clean_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut cleaned_records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        match validate_record(&record) {
            Ok(_) => cleaned_records.push(record),
            Err(e) => eprintln!("Skipping invalid record: {}", e),
        }
    }

    Ok(cleaned_records)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cleaned = clean_data("input.csv")?;
    println!("Cleaned {} valid records", cleaned.len());
    
    for record in &cleaned {
        println!("{:?}", record);
    }
    
    Ok(())
}use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    unique_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_items: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, data: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        
        for item in data {
            let normalized = Self::normalize_string(item);
            if self.unique_items.insert(normalized.clone()) {
                result.push(item.clone());
            }
        }
        
        result
    }

    pub fn normalize_string(input: &str) -> String {
        input.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    pub fn validate_email(email: &str) -> Result<(), Box<dyn Error>> {
        if email.contains('@') && email.contains('.') {
            Ok(())
        } else {
            Err("Invalid email format".into())
        }
    }

    pub fn clean_phone_number(phone: &str) -> String {
        phone.chars()
            .filter(|c| c.is_numeric())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "hello".to_string(),
            "HELLO".to_string(),
            "world".to_string(),
            "hello ".to_string(),
        ];
        
        let cleaned = cleaner.deduplicate(&data);
        assert_eq!(cleaned.len(), 2);
    }

    #[test]
    fn test_normalize_string() {
        assert_eq!(
            DataCleaner::normalize_string("  HELLO World!  "),
            "hello world"
        );
    }

    #[test]
    fn test_validate_email() {
        assert!(DataCleaner::validate_email("test@example.com").is_ok());
        assert!(DataCleaner::validate_email("invalid").is_err());
    }

    #[test]
    fn test_clean_phone_number() {
        assert_eq!(
            DataCleaner::clean_phone_number("+1 (123) 456-7890"),
            "11234567890"
        );
    }
}
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn clean_duplicate_rows(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("Empty or invalid input file".into()),
    };
    
    let mut seen = HashSet::new();
    let mut unique_rows = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        if !seen.contains(&line) {
            seen.insert(line.clone());
            unique_rows.push(line);
        }
    }
    
    let mut output_file = File::create(Path::new(output_path))?;
    writeln!(output_file, "{}", header)?;
    
    for row in unique_rows {
        writeln!(output_file, "{}", row)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_remove_duplicates() {
        let input_content = "id,name,value\n1,test,100\n2,example,200\n1,test,100\n3,sample,300";
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "{}", input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        clean_duplicate_rows(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = "id,name,value\n1,test,100\n2,example,200\n3,sample,300\n";
        
        assert_eq!(output_content, expected);
    }
}use std::collections::HashMap;

pub struct DataCleaner {
    threshold: f64,
}

impl DataCleaner {
    pub fn new(threshold: f64) -> Self {
        DataCleaner { threshold }
    }

    pub fn remove_outliers_iqr(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < 4 {
            return data.to_vec();
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75).floor() as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        data.iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn analyze_dataset(&self, data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if data.is_empty() {
            return stats;
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("count".to_string(), data.len() as f64);
        stats.insert("sum".to_string(), sum);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let cleaned = cleaner.remove_outliers_iqr(&data);
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = cleaner.analyze_dataset(&data);
        
        assert_eq!(stats["mean"], 3.0);
        assert_eq!(stats["count"], 5.0);
    }
}use std::collections::HashSet;

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
            let row_str: Vec<String> = row
                .iter()
                .map(|cell| cell.as_ref().unwrap_or(&"NULL".to_string()).to_string())
                .collect();
            let key = row_str.join("|");
            seen.insert(key)
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
}