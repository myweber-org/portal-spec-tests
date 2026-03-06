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

            records.push(fields);
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        let mut records = Vec::new();
        
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            records.push(fields);
        }

        records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
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
    fn test_file_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value\n1,test\n2,example")?;
        
        let parser = CsvParser::new();
        let result = parser.parse_file(temp_file.path())?;
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "test"]);
        Ok(())
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

        if records.is_empty() {
            return Err("No valid records found in CSV file".into());
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
    fn test_parse_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,test").unwrap();
        writeln!(temp_file, "2,data").unwrap();

        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "test"]);
    }

    #[test]
    fn test_empty_file() {
        let parser = CsvParser::new(',', false);
        let result = parser.parse_string("");
        assert!(result.is_empty());
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
            
            if line_num == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            records.push(record);
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        content
            .lines()
            .enumerate()
            .filter(|(line_num, line)| {
                if *line_num == 0 && self.has_header {
                    false
                } else {
                    !line.trim().is_empty()
                }
            })
            .map(|(_, line)| {
                line.split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .collect()
    }
}

pub fn calculate_column_average(records: &[Vec<String>], column_index: usize) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0;

    for record in records {
        if column_index < record.len() {
            if let Ok(value) = record[column_index].parse::<f64>() {
                sum += value;
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

    #[test]
    fn test_csv_parsing() {
        let parser = CsvParser::new(',', true);
        let csv_data = "name,age,score\nAlice,30,95.5\nBob,25,87.0\nCharlie,35,91.5";
        
        let records = parser.parse_string(csv_data);
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "95.5"]);
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            vec!["10.0".to_string(), "20.0".to_string()],
            vec!["30.0".to_string(), "40.0".to_string()],
            vec!["50.0".to_string(), "60.0".to_string()],
        ];
        
        let avg = calculate_column_average(&records, 0).unwrap();
        assert!((avg - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new(';', false);
        let csv_data = "name;age;score\nAlice;30;95.5";
        
        let records = parser.parse_string(csv_data);
        assert_eq!(records[0], vec!["name", "age", "score"]);
        assert_eq!(records[1], vec!["Alice", "30", "95.5"]);
    }
}