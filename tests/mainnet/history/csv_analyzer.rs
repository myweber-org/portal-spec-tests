use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvAnalyzer {
    file_path: String,
    delimiter: char,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Self {
        CsvAnalyzer {
            file_path: file_path.to_string(),
            delimiter: ',',
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn analyze(&self) -> Result<AnalysisResult, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .from_reader(file);

        let mut row_count = 0;
        let mut column_count = 0;
        let mut empty_cells = 0;
        let mut headers = Vec::new();

        if let Some(result) = rdr.headers().ok() {
            headers = result.iter().map(|s| s.to_string()).collect();
            column_count = headers.len();
        }

        for result in rdr.records() {
            let record = result?;
            row_count += 1;

            for field in record.iter() {
                if field.trim().is_empty() {
                    empty_cells += 1;
                }
            }
        }

        Ok(AnalysisResult {
            file_path: self.file_path.clone(),
            row_count,
            column_count,
            empty_cells,
            headers,
            delimiter: self.delimiter,
        })
    }
}

pub struct AnalysisResult {
    file_path: String,
    row_count: usize,
    column_count: usize,
    empty_cells: usize,
    headers: Vec<String>,
    delimiter: char,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV Analysis Summary");
        println!("===================");
        println!("File: {}", self.file_path);
        println!("Rows: {}", self.row_count);
        println!("Columns: {}", self.column_count);
        println!("Headers: {:?}", self.headers);
        println!("Delimiter: '{}'", self.delimiter);
        println!("Empty cells: {}", self.empty_cells);
        println!("Total cells: {}", self.row_count * self.column_count);
        
        if self.row_count > 0 && self.column_count > 0 {
            let fill_rate = 100.0 * (1.0 - (self.empty_cells as f64) / (self.row_count * self.column_count) as f64);
            println!("Data fill rate: {:.2}%", fill_rate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,,Paris").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap());
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.row_count, 3);
        assert_eq!(result.column_count, 3);
        assert_eq!(result.empty_cells, 1);
        assert_eq!(result.headers, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_csv_with_custom_delimiter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name|age|city").unwrap();
        writeln!(temp_file, "Alice|30|New York").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap())
            .with_delimiter('|');
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.row_count, 1);
        assert_eq!(result.column_count, 3);
        assert_eq!(result.delimiter, '|');
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
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

        let mut data = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                data.push(record);
            }
        }

        Ok(CsvAnalyzer { headers, data })
    }

    pub fn row_count(&self) -> usize {
        self.data.len()
    }

    pub fn column_stats(&self, column_index: usize) -> Option<HashMap<String, usize>> {
        if column_index >= self.headers.len() {
            return None;
        }

        let mut stats = HashMap::new();
        for row in &self.data {
            let value = &row[column_index];
            *stats.entry(value.clone()).or_insert(0) += 1;
        }
        Some(stats)
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Vec<&Vec<String>>
    where
        F: Fn(&Vec<String>) -> bool,
    {
        self.data.iter().filter(|row| predicate(row)).collect()
    }

    pub fn get_column_names(&self) -> &[String] {
        &self.headers
    }

    pub fn numeric_column_summary(&self, column_index: usize) -> Option<(f64, f64, f64)> {
        if column_index >= self.headers.len() {
            return None;
        }

        let mut values = Vec::new();
        for row in &self.data {
            if let Ok(num) = row[column_index].parse::<f64>() {
                values.push(num);
            }
        }

        if values.is_empty() {
            return None;
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let avg = sum / values.len() as f64;

        Some((min, max, avg))
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
        writeln!(file, "Alice,25,London").unwrap();
        writeln!(file, "Bob,30,Paris").unwrap();
        writeln!(file, "Charlie,35,London").unwrap();
        writeln!(file, "Diana,28,New York").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(analyzer.row_count(), 4);
        assert_eq!(analyzer.get_column_names(), &["name", "age", "city"]);
    }

    #[test]
    fn test_column_stats() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        
        let city_stats = analyzer.column_stats(2).unwrap();
        assert_eq!(city_stats.get("London"), Some(&2));
        assert_eq!(city_stats.get("Paris"), Some(&1));
    }

    #[test]
    fn test_filter_rows() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        
        let london_rows = analyzer.filter_rows(|row| row[2] == "London");
        assert_eq!(london_rows.len(), 2);
    }

    #[test]
    fn test_numeric_summary() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        
        let summary = analyzer.numeric_column_summary(1).unwrap();
        assert_eq!(summary.0, 25.0); // min age
        assert_eq!(summary.1, 35.0); // max age
        assert_eq!(summary.2, 29.5); // average age
    }
}