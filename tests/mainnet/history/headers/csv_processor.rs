use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
    MissingColumn(String),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        if self.has_header && !records.is_empty() {
            let header = records.remove(0);
            self.validate_header(&header)?;
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Vec<String>, CsvError> {
        let fields: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.iter().any(|f| f.is_empty()) {
            return Err(CsvError::ParseError(
                format!("Empty field found in line {}", line_num),
                line_num,
            ));
        }

        Ok(fields)
    }

    fn validate_header(&self, header: &[String]) -> Result<(), CsvError> {
        let mut seen = std::collections::HashSet::new();
        
        for column in header {
            if column.is_empty() {
                return Err(CsvError::InvalidHeader(
                    "Empty column name in header".to_string(),
                ));
            }
            
            if !seen.insert(column) {
                return Err(CsvError::InvalidHeader(
                    format!("Duplicate column name: {}", column),
                ));
            }
        }

        let required = ["id", "name", "value"];
        for req in &required {
            if !header.iter().any(|h| h == req) {
                return Err(CsvError::MissingColumn(req.to_string()));
            }
        }

        Ok(())
    }
}

pub fn calculate_average(values: &[Vec<String>], column_index: usize) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0;

    for record in values {
        if let Some(value) = record.get(column_index) {
            if let Ok(num) = value.parse::<f64>() {
                sum += num;
                count += 1;
            }
        }
    }

    if count > 0 {
        Some(sum / count as f64)
    } else {
        None
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
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,item1,10.5").unwrap();
        writeln!(temp_file, "2,item2,20.3").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["1", "item1", "10.5"]);
    }

    #[test]
    fn test_average_calculation() {
        let data = vec![
            vec!["1".to_string(), "item1".to_string(), "10.0".to_string()],
            vec!["2".to_string(), "item2".to_string(), "20.0".to_string()],
            vec!["3".to_string(), "item3".to_string(), "30.0".to_string()],
        ];
        
        let avg = calculate_average(&data, 2);
        assert_eq!(avg, Some(20.0));
    }
}