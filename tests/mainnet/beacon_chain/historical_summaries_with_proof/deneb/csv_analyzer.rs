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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_names: Vec<String>,
    pub numeric_columns: HashMap<String, Vec<f64>>,
    pub string_columns: HashMap<String, Vec<String>>,
}

impl CsvStats {
    pub fn new() -> Self {
        CsvStats {
            row_count: 0,
            column_count: 0,
            column_names: Vec::new(),
            numeric_columns: HashMap::new(),
            string_columns: HashMap::new(),
        }
    }

    pub fn analyze_file(path: &str, has_headers: bool) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stats = CsvStats::new();
        let mut lines = reader.lines();

        if has_headers {
            if let Some(Ok(header_line)) = lines.next() {
                stats.column_names = header_line.split(',').map(|s| s.trim().to_string()).collect();
                stats.column_count = stats.column_names.len();
            }
        }

        for line_result in lines {
            let line = line_result?;
            let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if stats.column_count == 0 {
                stats.column_count = values.len();
            }

            if values.len() != stats.column_count {
                continue;
            }

            stats.row_count += 1;

            for (i, value) in values.iter().enumerate() {
                let column_name = if has_headers && i < stats.column_names.len() {
                    stats.column_names[i].clone()
                } else {
                    format!("Column_{}", i + 1)
                };

                if let Ok(num) = value.parse::<f64>() {
                    stats.numeric_columns
                        .entry(column_name.clone())
                        .or_insert_with(Vec::new)
                        .push(num);
                } else {
                    stats.string_columns
                        .entry(column_name)
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
            }
        }

        Ok(stats)
    }

    pub fn calculate_column_stats(&self, column_name: &str) -> Option<ColumnStatistics> {
        if let Some(numbers) = self.numeric_columns.get(column_name) {
            if numbers.is_empty() {
                return None;
            }

            let sum: f64 = numbers.iter().sum();
            let count = numbers.len();
            let mean = sum / count as f64;
            
            let min = numbers.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = numbers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            let variance: f64 = numbers.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            let std_dev = variance.sqrt();

            Some(ColumnStatistics {
                column_name: column_name.to_string(),
                count,
                mean,
                min,
                max,
                std_dev,
                sum,
            })
        } else {
            None
        }
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        let mut result = Vec::new();
        
        for row_num in 0..self.row_count {
            let mut row_data = HashMap::new();
            
            for (col_name, values) in &self.string_columns {
                if let Some(value) = values.get(row_num) {
                    row_data.insert(col_name.clone(), value.clone());
                }
            }
            
            for (col_name, values) in &self.numeric_columns {
                if let Some(value) = values.get(row_num) {
                    row_data.insert(col_name.clone(), value.to_string());
                }
            }
            
            if predicate(&row_data) {
                let mut row_vec = Vec::new();
                for col_name in &self.column_names {
                    if let Some(value) = row_data.get(col_name) {
                        row_vec.push(value.clone());
                    } else {
                        row_vec.push(String::new());
                    }
                }
                result.push(row_vec);
            }
        }
        
        result
    }
}

#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    pub column_name: String,
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub sum: f64,
}

impl std::fmt::Display for ColumnStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistics for '{}':\n  Count: {}\n  Sum: {:.2}\n  Mean: {:.2}\n  Min: {:.2}\n  Max: {:.2}\n  Std Dev: {:.2}",
            self.column_name, self.count, self.sum, self.mean, self.min, self.max, self.std_dev
        )
    }
}

pub fn export_to_json(stats: &CsvStats, path: &str) -> Result<(), Box<dyn Error>> {
    use std::fs::write;
    
    let json_data = serde_json::json!({
        "row_count": stats.row_count,
        "column_count": stats.column_count,
        "column_names": stats.column_names,
        "numeric_columns": stats.numeric_columns.keys().collect::<Vec<_>>(),
        "string_columns": stats.string_columns.keys().collect::<Vec<_>>(),
    });
    
    write(path, json_data.to_string())?;
    Ok(())
}