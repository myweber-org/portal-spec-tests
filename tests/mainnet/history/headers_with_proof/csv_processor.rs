use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(Ok(line)) => line.split(',').map(|s| s.trim().to_string()).collect(),
            Some(Err(e)) => return Err(Box::new(e)),
            None => return Err("Empty CSV file".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column<F>(&self, column_index: usize, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&str) -> bool,
    {
        self.records
            .iter()
            .filter(|record| {
                record
                    .get(column_index)
                    .map(|value| predicate(value))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::from_file(temp_file.path()).unwrap();
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.get_headers(), &["name", "age", "city"]);

        let filtered = processor.filter_by_column(1, |age| age.parse::<u32>().unwrap_or(0) > 30);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Paris"]);
    }
}use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error)
    }
}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file(&self, file_path: &str) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(CsvError::ParseError(
                    "Empty record found".to_string(),
                    line_number,
                ));
            }

            records.push(fields);
        }

        if self.has_header && !records.is_empty() {
            let header = &records[0];
            if header.iter().any(|field| field.is_empty()) {
                return Err(CsvError::InvalidHeader(
                    "Header contains empty fields".to_string(),
                ));
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Ok(());
        }

        let expected_len = records[0].len();
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(CsvError::ParseError(
                    format!("Record has {} fields, expected {}", record.len(), expected_len),
                    idx + 1,
                ));
            }
        }

        Ok(())
    }
}

pub fn calculate_column_stats(records: &[Vec<String>]) -> Vec<(usize, usize)> {
    if records.is_empty() {
        return Vec::new();
    }

    let num_columns = records[0].len();
    let mut stats = vec![(0, 0); num_columns];

    for record in records {
        for (col_idx, field) in record.iter().enumerate() {
            let (ref mut total_len, ref mut count) = stats[col_idx];
            *total_len += field.len();
            *count += 1;
        }
    }

    stats
}