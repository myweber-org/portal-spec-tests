use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
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

    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        let mut records = Vec::new();
        
        for line in content.lines() {
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        records
    }
}

pub fn parse_csv_file(path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let parser = CsvParser::new();
    parser.parse_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_string() {
        let parser = CsvParser::new();
        let data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().delimiter(';');
        let data = "name;age;city\nJohn;30;New York";
        let result = parser.parse_string(data);
        
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_parse_file() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "name,age,city")?;
        writeln!(temp_file, "John,30,New York")?;
        writeln!(temp_file, "Jane,25,London")?;
        
        let parser = CsvParser::new();
        let result = parser.parse_file(temp_file.path())?;
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        Ok(())
    }

    #[test]
    fn test_no_header() {
        let parser = CsvParser::new().has_header(false);
        let data = "John,30,New York\nJane,25,London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(String),
    InvalidFormat(String),
    InvalidData(String),
    MissingColumn(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ParseError::MissingColumn(msg) => write!(f, "Missing column: {}", msg),
        }
    }
}

impl Error for ParseError {}

pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl Default for CsvParser {
    fn default() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: true,
        }
    }
}

impl CsvParser {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Record>, ParseError> {
        let file = File::open(path.as_ref())
            .map_err(|e| ParseError::IoError(format!("Failed to open file: {}", e)))?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line
                .map_err(|e| ParseError::IoError(format!("Failed to read line {}: {}", line_number, e)))?;

            if line_number == 1 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line, line_number)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<Record, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();

        if parts.len() < 4 {
            return Err(ParseError::InvalidFormat(format!(
                "Line {}: Expected 4 columns, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0]
            .trim()
            .parse::<u32>()
            .map_err(|e| ParseError::InvalidData(format!("Line {}: Invalid ID: {}", line_number, e)))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::InvalidData(format!("Line {}: Name cannot be empty", line_number)));
        }

        let value = parts[2]
            .trim()
            .parse::<f64>()
            .map_err(|e| ParseError::InvalidData(format!("Line {}: Invalid value: {}", line_number, e)))?;

        let active = parts[3]
            .trim()
            .parse::<bool>()
            .map_err(|e| ParseError::InvalidData(format!("Line {}: Invalid active flag: {}", line_number, e)))?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let mean = sum / count;

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
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
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,false").unwrap();

        let parser = CsvParser::default();
        let records = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[0].value, 10.5);
        assert_eq!(records[0].active, true);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record {
                id: 1,
                name: "Test1".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                value: 20.0,
                active: false,
            },
            Record {
                id: 3,
                name: "Test3".to_string(),
                value: 30.0,
                active: true,
            },
        ];

        let (mean, variance, std_dev) = CsvParser::calculate_statistics(&records);

        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}