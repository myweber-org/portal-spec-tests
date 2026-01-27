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
}