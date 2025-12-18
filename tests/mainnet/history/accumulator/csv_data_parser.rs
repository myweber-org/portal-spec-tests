
use std::error::Error;
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
pub enum ParseError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::IoError(error.to_string())
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

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_num, line) in lines {
            let line = line?;
            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(ParseError::ParseError(
                format!("Line {}: Expected 4 columns, found {}", line_num, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| ParseError::ParseError(
                format!("Line {}: Invalid ID '{}': {}", line_num, parts[0], e)
            ))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::ValidationError(
                format!("Line {}: Name cannot be empty", line_num)
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|e| ParseError::ParseError(
                format!("Line {}: Invalid value '{}': {}", line_num, parts[2], e)
            ))?;

        if value < 0.0 {
            return Err(ParseError::ValidationError(
                format!("Line {}: Value cannot be negative: {}", line_num, value)
            ));
        }

        let active = parts[3].parse::<bool>()
            .map_err(|e| ParseError::ParseError(
                format!("Line {}: Invalid boolean '{}': {}", line_num, parts[3], e)
            ))?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_stats(records: &[CsvRecord]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let mean = sum / count;

        let variance: f64 = records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Item1,10.5,true").unwrap();
        writeln!(temp_file, "2,Item2,20.0,false").unwrap();

        let parser = CsvParser::new();
        let result = parser.parse_file(temp_file.path());

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Item1");
        assert_eq!(records[1].value, 20.0);
    }

    #[test]
    fn test_parse_invalid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "invalid,Item1,10.5,true").unwrap();

        let parser = CsvParser::new();
        let result = parser.parse_file(temp_file.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_stats() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (mean, variance, std_dev) = CsvParser::calculate_stats(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}