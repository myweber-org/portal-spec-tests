
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

    pub fn process(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.unique_items.contains(&normalized) {
            return None;
        }

        self.unique_items.insert(normalized.clone());
        Some(normalized)
    }

    pub fn get_unique_count(&self) -> usize {
        self.unique_items.len()
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process("Hello"), Some("hello".to_string()));
        assert_eq!(cleaner.process("  HELLO  "), None);
        assert_eq!(cleaner.process("world"), Some("world".to_string()));
        assert_eq!(cleaner.process(""), None);
        
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_clear_functionality() {
        let mut cleaner = DataCleaner::new();
        
        cleaner.process("test");
        assert_eq!(cleaner.get_unique_count(), 1);
        
        cleaner.clear();
        assert_eq!(cleaner.get_unique_count(), 0);
        
        assert_eq!(cleaner.process("test"), Some("test".to_string()));
    }
}
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("Empty or invalid input file".into()),
    };
    
    let mut seen = HashSet::new();
    let mut unique_lines = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        if !seen.contains(&line) {
            seen.insert(line.clone());
            unique_lines.push(line);
        }
    }
    
    let mut output_file = File::create(Path::new(output_path))?;
    writeln!(output_file, "{}", header)?;
    
    for line in unique_lines {
        writeln!(output_file, "{}", line)?;
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
        let input_content = "id,name,value\n1,test,100\n2,test,200\n1,test,100\n3,other,300";
        let expected_output = "id,name,value\n1,test,100\n2,test,200\n3,other,300";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), input_content).unwrap();
        
        remove_duplicates(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        assert_eq!(output_content.trim(), expected_output);
    }
}
use std::collections::HashSet;

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

    pub fn deduplicate(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                unique_records.push(record);
            }
        }

        self.records = unique_records.clone();
        unique_records
    }

    pub fn validate_records(&self) -> Result<(), String> {
        for (index, record) in self.records.iter().enumerate() {
            if record.trim().is_empty() {
                return Err(format!("Empty record found at index {}", index));
            }
            
            if record.len() > 1000 {
                return Err(format!("Record too long at index {}", index));
            }
        }
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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
        
        let deduped = cleaner.deduplicate();
        assert_eq!(deduped.len(), 2);
        assert_eq!(cleaner.get_record_count(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid record".to_string());
        assert!(cleaner.validate_records().is_ok());
        
        cleaner.add_record("".to_string());
        assert!(cleaner.validate_records().is_err());
    }
}