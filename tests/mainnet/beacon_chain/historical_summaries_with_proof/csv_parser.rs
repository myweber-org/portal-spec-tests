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