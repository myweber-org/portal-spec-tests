
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    quote_char: char,
    has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            quote_char: '"',
            has_headers: true,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        CsvProcessor { config }
    }

    pub fn filter_rows<P, F>(&self, file_path: P, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut result = Vec::new();

        if self.config.has_headers {
            if let Some(header_line) = lines.next() {
                let header = self.parse_line(&header_line?);
                result.push(header);
            }
        }

        for line in lines {
            let line = line?;
            let fields = self.parse_line(&line);
            if predicate(&fields) {
                result.push(fields);
            }
        }

        Ok(result)
    }

    fn parse_line(&self, line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            
            if ch == self.config.quote_char {
                if in_quotes && i + 1 < chars.len() && chars[i + 1] == self.config.quote_char {
                    current_field.push(self.config.quote_char);
                    i += 1;
                } else {
                    in_quotes = !in_quotes;
                }
            } else if ch == self.config.delimiter && !in_quotes {
                fields.push(current_field.clone());
                current_field.clear();
            } else {
                current_field.push(ch);
            }
            i += 1;
        }

        fields.push(current_field);
        fields
    }

    pub fn count_matching_rows<P, F>(&self, file_path: P, predicate: F) -> Result<usize, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&[String]) -> bool,
    {
        let filtered = self.filter_rows(file_path, predicate)?;
        let start_index = if self.config.has_headers { 1 } else { 0 };
        Ok(filtered.len() - start_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "\"John, Doe\",30,\"New York\"").unwrap();
        writeln!(file, "Jane,25,\"Los Angeles\"").unwrap();
        writeln!(file, "Bob,35,Chicago").unwrap();
        file
    }

    #[test]
    fn test_filter_rows() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(CsvConfig::default());
        
        let result = processor.filter_rows(file.path(), |fields| {
            fields.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age >= 30)
        }).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["name", "age", "city"]);
        assert_eq!(result[1], vec!["John, Doe", "30", "New York"]);
        assert_eq!(result[2], vec!["Bob", "35", "Chicago"]);
    }

    #[test]
    fn test_count_matching_rows() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(CsvConfig::default());
        
        let count = processor.count_matching_rows(file.path(), |fields| {
            fields.get(2).map_or(false, |city| city.contains("York"))
        }).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_custom_delimiter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name|age|city").unwrap();
        writeln!(file, "John|30|NYC").unwrap();
        writeln!(file, "Jane|25|LA").unwrap();
        
        let config = CsvConfig {
            delimiter: '|',
            ..CsvConfig::default()
        };
        let processor = CsvProcessor::new(config);
        
        let result = processor.filter_rows(file.path(), |_| true).unwrap();
        assert_eq!(result[1], vec!["John", "30", "NYC"]);
    }
}