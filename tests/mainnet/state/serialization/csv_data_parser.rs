use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvParser {
    delimiter: char,
    has_headers: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_headers: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn without_headers(mut self) -> Self {
        self.has_headers = false;
        self
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

            if self.has_headers && line_num == 0 {
                continue;
            }

            records.push(record);
        }

        Ok(records)
    }

    pub fn parse_string(&self, data: &str) -> Vec<Vec<String>> {
        let mut records = Vec::new();
        
        for (line_num, line) in data.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if self.has_headers && line_num == 0 {
                continue;
            }

            records.push(record);
        }

        records
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
        let parser = CsvParser::new();
        let data = "name,age,salary\nJohn,30,50000\nJane,25,45000";
        let records = parser.parse_string(data);
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "50000"]);
        assert_eq!(records[1], vec!["Jane", "25", "45000"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().with_delimiter(';');
        let data = "name;age;salary\nJohn;30;50000";
        let records = parser.parse_string(data);
        
        assert_eq!(records[0], vec!["John", "30", "50000"]);
    }

    #[test]
    fn test_average_calculation() {
        let parser = CsvParser::new();
        let data = "value\n10.5\n20.5\n30.0";
        let records = parser.parse_string(data);
        
        let avg = calculate_column_average(&records, 0).unwrap();
        assert!((avg - 20.3333).abs() < 0.0001);
    }
}