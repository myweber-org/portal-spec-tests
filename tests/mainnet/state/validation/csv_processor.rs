use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub struct CsvProcessor {
    delimiter: char,
    expected_columns: Option<usize>,
}

impl CsvProcessor {
    pub fn new(delimiter: char) -> Self {
        CsvProcessor {
            delimiter,
            expected_columns: None,
        }
    }

    pub fn with_expected_columns(mut self, columns: usize) -> Self {
        self.expected_columns = Some(columns);
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            let trimmed = line_content.trim();
            if trimmed.is_empty() {
                continue;
            }

            let columns: Vec<String> = trimmed
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if let Some(expected) = self.expected_columns {
                if columns.len() != expected {
                    return Err(CsvError::ValidationError(format!(
                        "Line {}: Expected {} columns, found {}",
                        line_number,
                        expected,
                        columns.len()
                    )));
                }
            }

            records.push(CsvRecord { columns });
        }

        if records.is_empty() {
            return Err(CsvError::ParseError("File contains no valid data".to_string()));
        }

        Ok(records)
    }

    pub fn extract_column(&self, records: &[CsvRecord], column_index: usize) -> Result<Vec<String>, CsvError> {
        let mut result = Vec::with_capacity(records.len());
        
        for (i, record) in records.iter().enumerate() {
            if column_index >= record.columns.len() {
                return Err(CsvError::ValidationError(format!(
                    "Record {}: Column index {} out of bounds (max {})",
                    i,
                    column_index,
                    record.columns.len() - 1
                )));
            }
            result.push(record.columns[column_index].clone());
        }
        
        Ok(result)
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

        let processor = CsvProcessor::new(',').with_expected_columns(3);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].columns, vec!["name", "age", "city"]);
        assert_eq!(records[1].columns, vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_column_extraction() {
        let records = vec![
            CsvRecord { columns: vec!["a".to_string(), "b".to_string()] },
            CsvRecord { columns: vec!["c".to_string(), "d".to_string()] },
        ];
        
        let processor = CsvProcessor::new(',');
        let column = processor.extract_column(&records, 1).unwrap();
        
        assert_eq!(column, vec!["b", "d"]);
    }
}