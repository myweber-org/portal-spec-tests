
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    quote_char: char,
    has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            quote_char: '"',
            has_headers: true,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        CsvProcessor { config }
    }

    pub fn filter_rows<P, F>(&self, file_path: P, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut result = Vec::new();

        if self.config.has_headers {
            if let Some(header_line) = lines.next() {
                let header = self.parse_line(&header_line?);
                result.push(header);
            }
        }

        for line in lines {
            let line = line?;
            let fields = self.parse_line(&line);
            if predicate(&fields) {
                result.push(fields);
            }
        }

        Ok(result)
    }

    fn parse_line(&self, line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            
            if ch == self.config.quote_char {
                if in_quotes && i + 1 < chars.len() && chars[i + 1] == self.config.quote_char {
                    current_field.push(self.config.quote_char);
                    i += 1;
                } else {
                    in_quotes = !in_quotes;
                }
            } else if ch == self.config.delimiter && !in_quotes {
                fields.push(current_field.clone());
                current_field.clear();
            } else {
                current_field.push(ch);
            }
            i += 1;
        }

        fields.push(current_field);
        fields
    }

    pub fn count_matching_rows<P, F>(&self, file_path: P, predicate: F) -> Result<usize, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&[String]) -> bool,
    {
        let filtered = self.filter_rows(file_path, predicate)?;
        let start_index = if self.config.has_headers { 1 } else { 0 };
        Ok(filtered.len() - start_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "\"John, Doe\",30,\"New York\"").unwrap();
        writeln!(file, "Jane,25,\"Los Angeles\"").unwrap();
        writeln!(file, "Bob,35,Chicago").unwrap();
        file
    }

    #[test]
    fn test_filter_rows() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(CsvConfig::default());
        
        let result = processor.filter_rows(file.path(), |fields| {
            fields.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age >= 30)
        }).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["name", "age", "city"]);
        assert_eq!(result[1], vec!["John, Doe", "30", "New York"]);
        assert_eq!(result[2], vec!["Bob", "35", "Chicago"]);
    }

    #[test]
    fn test_count_matching_rows() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(CsvConfig::default());
        
        let count = processor.count_matching_rows(file.path(), |fields| {
            fields.get(2).map_or(false, |city| city.contains("York"))
        }).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_custom_delimiter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name|age|city").unwrap();
        writeln!(file, "John|30|NYC").unwrap();
        writeln!(file, "Jane|25|LA").unwrap();
        
        let config = CsvConfig {
            delimiter: '|',
            ..CsvConfig::default()
        };
        let processor = CsvProcessor::new(config);
        
        let result = processor.filter_rows(file.path(), |_| true).unwrap();
        assert_eq!(result[1], vec!["John", "30", "NYC"]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if self.has_header && line_number == 1 {
                continue;
            }

            let record = self.parse_line(&line, line_number)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 fields, found {}", line_number, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid ID '{}': {}", line_number, parts[0], e)
            ))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_number)
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid value '{}': {}", line_number, parts[2], e)
            ))?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Line {}: Invalid boolean value '{}'", line_number, parts[3])
            )),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_stats(records: &[CsvRecord]) -> (f64, f64, usize) {
        let active_count = records.iter().filter(|r| r.active).count();
        
        if records.is_empty() {
            return (0.0, 0.0, active_count);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let avg = sum / records.len() as f64;
        let max = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);

        (avg, max, active_count)
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
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,false").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,true").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.3);
        assert!(!records[1].active);
        
        let (avg, max, active_count) = CsvProcessor::calculate_stats(&records);
        assert!((avg - 15.5).abs() < 0.001);
        assert!((max - 20.3).abs() < 0.001);
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_invalid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,not_a_number,true").unwrap();

        let processor = CsvProcessor::new(',', false);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_err());
        if let Err(CsvError::ParseError(msg)) = result {
            assert!(msg.contains("Invalid value"));
        } else {
            panic!("Expected ParseError");
        }
    }
}