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

            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_num + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid records found in CSV file".into());
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut records = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            if line_num == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_num + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
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
    fn test_basic_csv_parsing() {
        let parser = CsvParser::new();
        let csv_data = "name,age,city\nJohn,30,New York\nJane,25,London";
        
        let result = parser.parse_string(csv_data).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().delimiter(';');
        let csv_data = "name;age;city\nJohn;30;New York";
        
        let result = parser.parse_string(csv_data).unwrap();
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_no_header() {
        let parser = CsvParser::new().has_header(false);
        let csv_data = "John,30,New York\nJane,25,London";
        
        let result = parser.parse_string(csv_data).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_empty_field_error() {
        let parser = CsvParser::new();
        let csv_data = "name,age,city\nJohn,,New York";
        
        let result = parser.parse_string(csv_data);
        assert!(result.is_err());
    }
}