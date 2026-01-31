use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct ColumnStats {
    pub name: String,
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P) -> Result<Vec<ColumnStats>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?.clone();
    let mut column_data: Vec<Vec<f64>> = vec![Vec::new(); headers.len()];
    
    for result in rdr.records() {
        let record = result?;
        for (i, field) in record.iter().enumerate() {
            if let Ok(value) = field.parse::<f64>() {
                column_data[i].push(value);
            }
        }
    }
    
    let mut stats = Vec::new();
    for (i, data) in column_data.iter().enumerate() {
        if !data.is_empty() {
            let col_stats = calculate_stats(&headers[i], data);
            stats.push(col_stats);
        }
    }
    
    Ok(stats)
}

fn calculate_stats(name: &str, data: &[f64]) -> ColumnStats {
    let count = data.len();
    let sum: f64 = data.iter().sum();
    let mean = sum / count as f64;
    
    let min = *data.iter().fold(&f64::INFINITY, |a, &b| a.min(&b));
    let max = *data.iter().fold(&f64::NEG_INFINITY, |a, &b| a.max(&b));
    
    let variance: f64 = data.iter()
        .map(|value| {
            let diff = mean - value;
            diff * diff
        })
        .sum::<f64>() / count as f64;
    
    let std_dev = variance.sqrt();
    
    ColumnStats {
        name: name.to_string(),
        count,
        mean,
        min,
        max,
        std_dev,
    }
}

pub fn print_summary(stats: &[ColumnStats]) {
    println!("CSV Analysis Summary:");
    println!("{:<20} {:<10} {:<10} {:<10} {:<10} {:<10}", 
             "Column", "Count", "Mean", "Min", "Max", "Std Dev");
    println!("{}", "-".repeat(80));
    
    for stat in stats {
        println!("{:<20} {:<10} {:<10.4} {:<10.4} {:<10.4} {:<10.4}",
                 stat.name, stat.count, stat.mean, stat.min, stat.max, stat.std_dev);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "col1,col2,col3").unwrap();
        writeln!(temp_file, "1.0,2.0,3.0").unwrap();
        writeln!(temp_file, "4.0,5.0,6.0").unwrap();
        writeln!(temp_file, "7.0,8.0,9.0").unwrap();
        
        let stats = analyze_csv(temp_file.path()).unwrap();
        assert_eq!(stats.len(), 3);
        assert_eq!(stats[0].name, "col1");
        assert_eq!(stats[0].count, 3);
        assert!((stats[0].mean - 4.0).abs() < 0.001);
    }
}use std::error::Error;
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

        let mut total_rows = 0;
        let mut total_columns = 0;
        let mut column_names = Vec::new();
        let mut empty_cells = 0;
        let mut numeric_columns = Vec::new();

        if let Ok(headers) = rdr.headers() {
            total_columns = headers.len();
            column_names = headers.iter().map(|s| s.to_string()).collect();
            
            for name in &column_names {
                if name.to_lowercase().contains("id") || 
                   name.to_lowercase().contains("count") ||
                   name.to_lowercase().contains("amount") {
                    numeric_columns.push(name.clone());
                }
            }
        }

        for result in rdr.records() {
            let record = result?;
            total_rows += 1;
            
            for field in record.iter() {
                if field.trim().is_empty() {
                    empty_cells += 1;
                }
            }
        }

        Ok(AnalysisResult {
            total_rows,
            total_columns,
            column_names,
            empty_cells,
            numeric_columns,
            file_size: std::fs::metadata(path)?.len(),
        })
    }

    pub fn validate(&self) -> Result<ValidationReport, Box<dyn Error>> {
        let analysis = self.analyze()?;
        
        let mut issues = Vec::new();
        
        if analysis.total_rows == 0 {
            issues.push("File contains no data rows".to_string());
        }
        
        if analysis.empty_cells > 0 {
            issues.push(format!("Found {} empty cells", analysis.empty_cells));
        }
        
        if analysis.total_columns == 0 {
            issues.push("No columns detected".to_string());
        }

        Ok(ValidationReport {
            is_valid: issues.is_empty(),
            issues,
            summary: analysis,
        })
    }
}

pub struct AnalysisResult {
    pub total_rows: usize,
    pub total_columns: usize,
    pub column_names: Vec<String>,
    pub empty_cells: usize,
    pub numeric_columns: Vec<String>,
    pub file_size: u64,
}

pub struct ValidationReport {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub summary: AnalysisResult,
}

impl std::fmt::Display for AnalysisResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CSV Analysis Report")?;
        writeln!(f, "==================")?;
        writeln!(f, "Total Rows: {}", self.total_rows)?;
        writeln!(f, "Total Columns: {}", self.total_columns)?;
        writeln!(f, "File Size: {} bytes", self.file_size)?;
        writeln!(f, "Empty Cells: {}", self.empty_cells)?;
        writeln!(f, "\nColumns:")?;
        
        for (i, name) in self.column_names.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, name)?;
        }
        
        if !self.numeric_columns.is_empty() {
            writeln!(f, "\nPotential Numeric Columns:")?;
            for name in &self.numeric_columns {
                writeln!(f, "  • {}", name)?;
            }
        }
        
        Ok(())
    }
}

pub fn analyze_csv_file(path: &str) -> Result<(), Box<dyn Error>> {
    let analyzer = CsvAnalyzer::new(path);
    let result = analyzer.analyze()?;
    println!("{}", result);
    
    let validation = analyzer.validate()?;
    if !validation.is_valid {
        println!("\nValidation Issues:");
        for issue in validation.issues {
            println!("  ⚠ {}", issue);
        }
    } else {
        println!("\n✓ File validation passed");
    }
    
    Ok(())
}