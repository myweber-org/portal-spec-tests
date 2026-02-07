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

        let headers = rdr.headers()?.clone();
        let mut row_count = 0;
        let mut column_counts = vec![0; headers.len()];
        let mut empty_cells = 0;

        for result in rdr.records() {
            let record = result?;
            row_count += 1;

            for (i, field) in record.iter().enumerate() {
                if i < column_counts.len() {
                    column_counts[i] += 1;
                    if field.trim().is_empty() {
                        empty_cells += 1;
                    }
                }
            }
        }

        Ok(AnalysisResult {
            file_path: self.file_path.clone(),
            row_count,
            column_count: headers.len(),
            headers: headers.iter().map(|s| s.to_string()).collect(),
            column_counts,
            empty_cells,
            total_cells: row_count * headers.len(),
        })
    }
}

pub struct AnalysisResult {
    file_path: String,
    row_count: usize,
    column_count: usize,
    headers: Vec<String>,
    column_counts: Vec<usize>,
    empty_cells: usize,
    total_cells: usize,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV Analysis Summary");
        println!("====================");
        println!("File: {}", self.file_path);
        println!("Rows: {}", self.row_count);
        println!("Columns: {}", self.column_count);
        println!("Total cells: {}", self.total_cells);
        println!("Empty cells: {}", self.empty_cells);
        println!("Data completeness: {:.2}%", 
            ((self.total_cells - self.empty_cells) as f64 / self.total_cells as f64) * 100.0);
        
        println!("\nColumn headers:");
        for (i, header) in self.headers.iter().enumerate() {
            println!("  {}. {}", i + 1, header);
        }

        if let Some(min) = self.column_counts.iter().min() {
            if let Some(max) = self.column_counts.iter().max() {
                println!("\nColumn record counts - Min: {}, Max: {}", min, max);
            }
        }
    }
}

pub fn validate_csv_structure(file_path: &str) -> Result<bool, Box<dyn Error>> {
    let analyzer = CsvAnalyzer::new(file_path);
    let result = analyzer.analyze()?;
    
    let consistent_columns = result.column_counts.iter().all(|&count| count == result.row_count);
    let has_headers = !result.headers.is_empty();
    
    Ok(consistent_columns && has_headers)
}