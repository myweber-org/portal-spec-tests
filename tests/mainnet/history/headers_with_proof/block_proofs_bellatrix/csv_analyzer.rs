use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvAnalyzer { headers, records })
    }

    pub fn row_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
        self.headers.len()
    }

    pub fn column_stats(&self, column_index: usize) -> Option<HashMap<String, usize>> {
        if column_index >= self.headers.len() {
            return None;
        }

        let mut frequency = HashMap::new();
        for record in &self.records {
            let value = &record[column_index];
            *frequency.entry(value.clone()).or_insert(0) += 1;
        }

        Some(frequency)
    }

    pub fn filter_by_column(&self, column_index: usize, value: &str) -> Vec<Vec<String>> {
        self.records
            .iter()
            .filter(|record| record[column_index] == value)
            .cloned()
            .collect()
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }

    pub fn unique_values(&self, column_index: usize) -> Option<Vec<String>> {
        if column_index >= self.headers.len() {
            return None;
        }

        let mut values: Vec<String> = self.records
            .iter()
            .map(|record| record[column_index].clone())
            .collect();
        
        values.sort();
        values.dedup();
        Some(values)
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
        writeln!(file, "Alice,30,New York").unwrap();
        writeln!(file, "Bob,25,London").unwrap();
        writeln!(file, "Charlie,30,Paris").unwrap();
        writeln!(file, "Diana,25,New York").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(analyzer.row_count(), 4);
        assert_eq!(analyzer.column_count(), 3);
        assert_eq!(analyzer.get_headers(), &vec!["name".to_string(), "age".to_string(), "city".to_string()]);
    }

    #[test]
    fn test_column_stats() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        
        let stats = analyzer.column_stats(1).unwrap();
        assert_eq!(stats.get("30"), Some(&2));
        assert_eq!(stats.get("25"), Some(&2));
    }

    #[test]
    fn test_filtering() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = analyzer.filter_by_column(2, "New York");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Diana");
    }

    #[test]
    fn test_unique_values() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        
        let unique_cities = analyzer.unique_values(2).unwrap();
        assert_eq!(unique_cities, vec!["London", "New York", "Paris"]);
    }
}