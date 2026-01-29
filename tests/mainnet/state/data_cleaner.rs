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
}