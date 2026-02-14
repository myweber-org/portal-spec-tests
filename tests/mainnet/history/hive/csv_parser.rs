
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(String),
    InvalidFormat(String),
    InvalidData(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
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
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_num, line) in lines {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Record, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(ParseError::InvalidFormat(
                format!("Line {}: Expected 4 fields, found {}", line_num, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| ParseError::InvalidData(
                format!("Line {}: Invalid ID '{}': {}", line_num, parts[0], e)
            ))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::InvalidData(
                format!("Line {}: Name cannot be empty", line_num)
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|e| ParseError::InvalidData(
                format!("Line {}: Invalid value '{}': {}", line_num, parts[2], e)
            ))?;

        let active = parts[3].parse::<bool>()
            .map_err(|e| ParseError::InvalidData(
                format!("Line {}: Invalid boolean '{}': {}", line_num, parts[3], e)
            ))?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
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
    fn test_parse_valid_csv() {
        let data = "id,name,value,active\n1,Test,3.14,true\n2,Another,42.0,false";
        let parser = CsvParser::new();
        let result = parser.parse(data.as_bytes()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Record {
            id: 1,
            name: "Test".to_string(),
            value: 3.14,
            active: true,
        });
    }

    #[test]
    fn test_parse_invalid_data() {
        let data = "id,name,value,active\n1,Test,not_a_number,true";
        let parser = CsvParser::new();
        let result = parser.parse(data.as_bytes());
        
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_delimiter() {
        let data = "id|name|value|active\n1|Test|3.14|true";
        let parser = CsvParser::new().with_delimiter('|');
        let result = parser.parse(data.as_bytes()).unwrap();
        
        assert_eq!(result[0].name, "Test");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if self.has_header && line_num == 0 {
                continue;
            }

            records.push(record);
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .skip(if self.has_header { 1 } else { 0 })
            .map(|line| {
                line.split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_with_header() {
        let parser = CsvParser::new(',', true);
        let csv_data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_parse_without_header() {
        let parser = CsvParser::new(';', false);
        let csv_data = "John;30;New York\nJane;25;London";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_parse_file() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value")?;
        writeln!(temp_file, "1,100")?;
        writeln!(temp_file, "2,200")?;
        
        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path())?;
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "100"]);
        Ok(())
    }

    #[test]
    fn test_empty_lines_skipped() {
        let parser = CsvParser::new(',', false);
        let csv_data = "a,b,c\n\n\nd,e,f\n\n";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["a", "b", "c"]);
        assert_eq!(result[1], vec!["d", "e", "f"]);
    }
}