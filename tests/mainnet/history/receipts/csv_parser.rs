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
            .enumerate()
            .filter_map(|(idx, line)| {
                if idx == 0 && self.has_header {
                    None
                } else {
                    let record: Vec<String> = line
                        .split(self.delimiter)
                        .map(|field| field.trim().to_string())
                        .collect();
                    if record.is_empty() {
                        None
                    } else {
                        Some(record)
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_with_comma_delimiter() {
        let parser = CsvParser::new(',', false);
        let data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["name", "age", "city"]);
        assert_eq!(result[1], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_parse_string_with_header() {
        let parser = CsvParser::new(';', true);
        let data = "Name;Age;City\nJohn;30;New York\nJane;25;London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_parse_string_with_tab_delimiter() {
        let parser = CsvParser::new('\t', false);
        let data = "id\tvalue\tstatus\n1\t100\tactive\n2\t200\tinactive";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["id", "value", "status"]);
    }
}