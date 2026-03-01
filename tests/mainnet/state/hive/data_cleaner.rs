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

    pub fn normalize_whitespace(&mut self) {
        for record in &mut self.records {
            let normalized = record
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ");
            *record = normalized;
        }
    }

    pub fn to_lowercase(&mut self) {
        for record in &mut self.records {
            *record = record.to_lowercase();
        }
    }

    pub fn get_records(&self) -> &Vec<String> {
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
        cleaner.add_record("another".to_string());

        let unique = cleaner.deduplicate();
        assert_eq!(unique.len(), 2);
        assert_eq!(cleaner.get_records().len(), 2);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  multiple   spaces   ".to_string());
        cleaner.normalize_whitespace();

        assert_eq!(cleaner.get_records()[0], "multiple spaces");
    }
}use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data<R: BufRead, W: Write>(input: R, output: &mut W) -> io::Result<()> {
    let mut unique_lines = HashSet::new();
    
    for line in input.lines() {
        let line = line?;
        unique_lines.insert(line);
    }
    
    let mut sorted_lines: Vec<String> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    for line in sorted_lines {
        writeln!(output, "{}", line)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_clean_data() {
        let input_data = "banana\napple\nbanana\ncherry\napple\n";
        let expected_output = "apple\nbanana\ncherry\n";
        
        let input = Cursor::new(input_data);
        let mut output = Vec::new();
        
        clean_data(input, &mut output).unwrap();
        
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, expected_output);
    }
}