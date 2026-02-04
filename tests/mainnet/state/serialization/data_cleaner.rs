
use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub validate_emails: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_duplicates: true,
            validate_emails: false,
        }
    }

    pub fn deduplicate_strings(&self, input: Vec<String>) -> Vec<String> {
        if !self.remove_duplicates {
            return input;
        }

        let mut seen = HashSet::new();
        input
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }

    pub fn clean_email_list(&self, emails: Vec<String>) -> Vec<String> {
        let mut cleaned = self.deduplicate_strings(emails);

        if self.validate_emails {
            cleaned.retain(|email| self.is_valid_email(email));
        }

        cleaned.sort();
        cleaned
    }

    fn is_valid_email(&self, email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2
            && !parts[0].is_empty()
            && !domain_parts.iter().any(|part| part.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new();
        let input = vec![
            "test@example.com".to_string(),
            "test@example.com".to_string(),
            "unique@domain.com".to_string(),
        ];

        let result = cleaner.deduplicate_strings(input);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"test@example.com".to_string()));
        assert!(result.contains(&"unique@domain.com".to_string()));
    }

    #[test]
    fn test_email_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.validate_emails = true;

        let emails = vec![
            "valid@example.com".to_string(),
            "invalid-email".to_string(),
            "another@valid.org".to_string(),
            "@missinglocal.com".to_string(),
        ];

        let result = cleaner.clean_email_list(emails);
        assert_eq!(result, vec!["another@valid.org", "valid@example.com"]);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_path)?;

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: record.value.max(0.0),
            category: record.category.to_uppercase(),
        };

        wtr.serialize(&cleaned_record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "input.csv";
    let output = "cleaned_output.csv";
    
    match clean_csv(input, output) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during cleaning: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            category: "A".to_string(),
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "B".to_string(),
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}