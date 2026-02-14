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
        content
            .lines()
            .skip(if self.has_header { 1 } else { 0 })
            .map(|line| {
                line.split(self.delimiter)
                    .map(|field| field.trim().to_string())
                    .collect()
            })
            .filter(|record: &Vec<String>| !record.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let parser = CsvParser::new();
        let csv_data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().delimiter(';');
        let csv_data = "name;age;city\nJohn;30;New York";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_no_header() {
        let parser = CsvParser::new().has_header(false);
        let csv_data = "John,30,New York\nJane,25,London";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Debug)]
pub struct CsvParser {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            headers: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn parse_file(&mut self, path: &str, has_headers: bool) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if has_headers {
            if let Some(header_line) = lines.next() {
                let header_line = header_line?;
                self.headers = header_line.split(',').map(|s| s.trim().to_string()).collect();
            }
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            self.records.push(record);
        }

        Ok(())
    }

    pub fn get_column<T: FromStr>(&self, column_index: usize) -> Result<Vec<T>, Box<dyn Error>>
    where
        T::Err: Error + 'static,
    {
        let mut column_data = Vec::new();
        for record in &self.records {
            if column_index < record.len() {
                let value = record[column_index].parse::<T>()?;
                column_data.push(value);
            }
        }
        Ok(column_data)
    }

    pub fn get_header_column<T: FromStr>(&self, header_name: &str) -> Result<Vec<T>, Box<dyn Error>>
    where
        T::Err: Error + 'static,
    {
        let column_index = self.headers
            .iter()
            .position(|h| h == header_name)
            .ok_or_else(|| format!("Header '{}' not found", header_name))?;
        
        self.get_column(column_index)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
        if !self.records.is_empty() {
            self.records[0].len()
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let mut parser = CsvParser::new();
        let result = parser.parse_file(temp_file.path().to_str().unwrap(), true);
        assert!(result.is_ok());
        assert_eq!(parser.record_count(), 2);
        assert_eq!(parser.column_count(), 3);
        
        let ages: Vec<i32> = parser.get_header_column("age").unwrap();
        assert_eq!(ages, vec![30, 25]);
    }
}