use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvAnalyzer {
    pub file_path: String,
    pub delimiter: char,
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
        let mut headers = Vec::new();

        for result in rdr.records() {
            let record = result?;
            
            if row_count == 0 {
                column_count = record.len();
                headers = record.iter().map(|s| s.to_string()).collect();
            }
            
            row_count += 1;
        }

        Ok(AnalysisResult {
            file_path: self.file_path.clone(),
            row_count,
            column_count,
            headers,
            delimiter: self.delimiter,
        })
    }
}

pub struct AnalysisResult {
    pub file_path: String,
    pub row_count: usize,
    pub column_count: usize,
    pub headers: Vec<String>,
    pub delimiter: char,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV File Analysis Summary");
        println!("=========================");
        println!("File: {}", self.file_path);
        println!("Rows: {}", self.row_count);
        println!("Columns: {}", self.column_count);
        println!("Delimiter: '{}'", self.delimiter);
        println!("Headers: {:?}", self.headers);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap());
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.row_count, 2);
        assert_eq!(result.column_count, 3);
        assert_eq!(result.headers, vec!["name", "age", "city"]);
    }
}