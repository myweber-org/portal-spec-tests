
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvAnalyzer {
    pub file_path: String,
    pub delimiter: char,
    pub has_header: bool,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Self {
        CsvAnalyzer {
            file_path: file_path.to_string(),
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn analyze(&self) -> Result<AnalysisResult, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut row_count = 0;
        let mut column_count = 0;
        let mut empty_cells = 0;
        let mut numeric_cells = 0;
        let mut text_cells = 0;

        let mut headers = Vec::new();

        if self.has_header {
            if let Some(Ok(header_line)) = lines.next() {
                headers = header_line
                    .split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect();
                column_count = headers.len();
            }
        }

        for line_result in lines {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }

            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_count == 0 {
                column_count = columns.len();
            }

            if columns.len() != column_count {
                return Err(format!(
                    "Row {} has {} columns, expected {}",
                    row_count + 1,
                    columns.len(),
                    column_count
                ).into());
            }

            for cell in columns {
                let trimmed = cell.trim();
                if trimmed.is_empty() {
                    empty_cells += 1;
                } else if trimmed.parse::<f64>().is_ok() {
                    numeric_cells += 1;
                } else {
                    text_cells += 1;
                }
            }

            row_count += 1;
        }

        if !self.has_header && row_count > 0 {
            headers = (1..=column_count)
                .map(|i| format!("Column_{}", i))
                .collect();
        }

        Ok(AnalysisResult {
            file_path: self.file_path.clone(),
            row_count,
            column_count,
            total_cells: row_count * column_count,
            empty_cells,
            numeric_cells,
            text_cells,
            headers,
        })
    }
}

pub struct AnalysisResult {
    pub file_path: String,
    pub row_count: usize,
    pub column_count: usize,
    pub total_cells: usize,
    pub empty_cells: usize,
    pub numeric_cells: usize,
    pub text_cells: usize,
    pub headers: Vec<String>,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV Analysis Summary");
        println!("====================");
        println!("File: {}", self.file_path);
        println!("Rows: {}", self.row_count);
        println!("Columns: {}", self.column_count);
        println!("Total cells: {}", self.total_cells);
        println!("Empty cells: {} ({:.1}%)", 
            self.empty_cells, 
            (self.empty_cells as f64 / self.total_cells as f64) * 100.0
        );
        println!("Numeric cells: {} ({:.1}%)", 
            self.numeric_cells,
            (self.numeric_cells as f64 / self.total_cells as f64) * 100.0
        );
        println!("Text cells: {} ({:.1}%)", 
            self.text_cells,
            (self.text_cells as f64 / self.total_cells as f64) * 100.0
        );
        
        if !self.headers.is_empty() {
            println!("\nHeaders:");
            for (i, header) in self.headers.iter().enumerate() {
                println!("  {}. {}", i + 1, header);
            }
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
        writeln!(temp_file, "Name,Age,City").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,,Paris").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap());
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.row_count, 3);
        assert_eq!(result.column_count, 3);
        assert_eq!(result.total_cells, 9);
        assert_eq!(result.empty_cells, 1);
        assert_eq!(result.numeric_cells, 2);
        assert_eq!(result.text_cells, 6);
        assert_eq!(result.headers, vec!["Name", "Age", "City"]);
    }

    #[test]
    fn test_csv_without_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap())
            .with_header(false);
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.row_count, 2);
        assert_eq!(result.column_count, 3);
        assert_eq!(result.headers, vec!["Column_1", "Column_2", "Column_3"]);
    }

    #[test]
    fn test_invalid_column_count() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Name,Age,City").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap());
        let result = analyzer.analyze();

        assert!(result.is_err());
    }
}