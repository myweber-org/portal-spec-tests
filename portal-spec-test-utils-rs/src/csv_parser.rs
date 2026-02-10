use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    InvalidFormat(String),
    InvalidNumber(String),
    InvalidBoolean(String),
    MissingField,
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Record>, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.parse(reader)
    }

    pub fn parse<R: BufRead>(&self, reader: R) -> Result<Vec<Record>, ParseError> {
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for (line_num, line) in lines.enumerate() {
            let line = line?;
            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Record, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() < 4 {
            return Err(ParseError::MissingField);
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| ParseError::InvalidNumber(format!("Invalid ID at line {}: {}", line_num, parts[0])))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::InvalidFormat(format!("Empty name at line {}", line_num)));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber(format!("Invalid value at line {}: {}", line_num, parts[2])))?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(ParseError::InvalidBoolean(format!("Invalid boolean at line {}: {}", line_num, parts[3]))),
        };

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    pub fn validate_records(&self, records: &[Record]) -> Result<(), ParseError> {
        let mut seen_ids = std::collections::HashSet::new();
        
        for record in records {
            if !seen_ids.insert(record.id) {
                return Err(ParseError::InvalidFormat(format!("Duplicate ID found: {}", record.id)));
            }
            
            if record.value < 0.0 {
                return Err(ParseError::InvalidFormat(format!("Negative value for ID {}: {}", record.id, record.value)));
            }
        }
        
        Ok(())
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let parser = CsvParser::new();
        let line = "1,John Doe,42.5,true";
        let record = parser.parse_line(line, 1).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "John Doe");
        assert_eq!(record.value, 42.5);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_parse_invalid_number() {
        let parser = CsvParser::new();
        let line = "abc,John Doe,42.5,true";
        let result = parser.parse_line(line, 1);
        
        assert!(matches!(result, Err(ParseError::InvalidNumber(_))));
    }

    #[test]
    fn test_parse_missing_field() {
        let parser = CsvParser::new();
        let line = "1,John Doe,42.5";
        let result = parser.parse_line(line, 1);
        
        assert!(matches!(result, Err(ParseError::MissingField)));
    }

    #[test]
    fn test_validate_records() {
        let parser = CsvParser::new();
        let records = vec![
            Record { id: 1, name: "Alice".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Bob".to_string(), value: 20.0, active: false },
        ];
        
        assert!(parser.validate_records(&records).is_ok());
    }

    #[test]
    fn test_validate_duplicate_ids() {
        let parser = CsvParser::new();
        let records = vec![
            Record { id: 1, name: "Alice".to_string(), value: 10.0, active: true },
            Record { id: 1, name: "Bob".to_string(), value: 20.0, active: false },
        ];
        
        assert!(matches!(parser.validate_records(&records), Err(ParseError::InvalidFormat(_))));
    }
}