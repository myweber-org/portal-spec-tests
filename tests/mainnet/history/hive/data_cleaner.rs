use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        self.dedupe_set.insert(normalized)
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

pub fn validate_email(email: &str) -> Result<bool, Box<dyn Error>> {
    if email.is_empty() {
        return Err("Email cannot be empty".into());
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Ok(false);
    }
    
    let domain_parts: Vec<&str> = parts[1].split('.').collect();
    Ok(domain_parts.len() >= 2 && !domain_parts.iter().any(|p| p.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World!  "), "hello world");
    }

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test"));
        assert!(!cleaner.deduplicate("TEST"));
        assert!(cleaner.deduplicate("another"));
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").unwrap());
        assert!(!validate_email("invalid-email").unwrap());
        assert!(validate_email("test@sub.domain.co.uk").unwrap());
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use log::{info, warn};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);
    
    let mut cleaned_count = 0;
    let mut skipped_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                warn!("Skipping malformed record: {}", e);
                skipped_count += 1;
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
        
        writer.serialize(&cleaned_record)?;
        cleaned_count += 1;
    }
    
    writer.flush()?;
    
    info!("Data cleaning completed: {} records processed, {} cleaned, {} skipped", 
          cleaned_count + skipped_count, cleaned_count, skipped_count);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    match clean_csv_data(input_file, output_file) {
        Ok(_) => info!("Successfully cleaned data from {} to {}", input_file, output_file),
        Err(e) => eprintln!("Error cleaning data: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_clean_csv_data() {
        let input_data = "id,name,value,category\n1, test ,3.14,science\n2,data,inf,engineering\n";
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(input_file.path().to_str().unwrap(), 
                                   output_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
    }
}use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    
    println!("Enter data (press Ctrl+D when finished):");
    for line in stdin.lock().lines() {
        match line {
            Ok(content) => buffer.push_str(&format!("{}\n", content)),
            Err(e) => eprintln!("Error reading input: {}", e),
        }
    }
    
    let cleaned = clean_data(&buffer);
    println!("\nCleaned data:");
    io::stdout().write_all(cleaned.as_bytes()).unwrap();
}use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    for result in csv_reader.records() {
        let record = result?;
        let filtered_record: Vec<&str> = record
            .iter()
            .filter(|field| !field.trim().is_empty())
            .collect();

        if filtered_record.len() == headers.len() {
            csv_writer.write_record(&filtered_record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}