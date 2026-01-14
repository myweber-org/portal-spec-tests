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
        
        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let mut records = Vec::new();
        for line in lines {
            let line = line?;
            let record: Vec<String> = line
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
    
    pub fn column_stats(&self, column_index: usize) -> Option<ColumnStats> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let mut numeric_values = Vec::new();
        let mut string_values = Vec::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                if let Ok(num) = value.parse::<f64>() {
                    numeric_values.push(num);
                } else {
                    string_values.push(value.clone());
                }
            }
        }
        
        Some(ColumnStats {
            column_name: self.headers[column_index].clone(),
            numeric_count: numeric_values.len(),
            string_count: string_values.len(),
            numeric_stats: if !numeric_values.is_empty() {
                Some(NumericStats::from_values(&numeric_values))
            } else {
                None
            },
            unique_strings: string_values.len(),
        })
    }
    
    pub fn filter_by_column(&self, column_index: usize, predicate: &dyn Fn(&str) -> bool) -> Vec<Vec<String>> {
        self.records.iter()
            .filter(|record| {
                record.get(column_index)
                    .map(|value| predicate(value))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
    
    pub fn get_column_frequency(&self, column_index: usize) -> Option<HashMap<String, usize>> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let mut frequency = HashMap::new();
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                *frequency.entry(value.clone()).or_insert(0) += 1;
            }
        }
        
        Some(frequency)
    }
}

pub struct ColumnStats {
    column_name: String,
    numeric_count: usize,
    string_count: usize,
    numeric_stats: Option<NumericStats>,
    unique_strings: usize,
}

pub struct NumericStats {
    min: f64,
    max: f64,
    mean: f64,
    median: f64,
}

impl NumericStats {
    fn from_values(values: &[f64]) -> Self {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min = *sorted.first().unwrap_or(&0.0);
        let max = *sorted.last().unwrap_or(&0.0);
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let median = if sorted.len() % 2 == 0 {
            let mid = sorted.len() / 2;
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };
        
        NumericStats { min, max, mean, median }
    }
}

impl std::fmt::Display for ColumnStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Column: {}", self.column_name)?;
        writeln!(f, "  Numeric values: {}", self.numeric_count)?;
        writeln!(f, "  String values: {}", self.string_count)?;
        writeln!(f, "  Unique strings: {}", self.unique_strings)?;
        
        if let Some(stats) = &self.numeric_stats {
            writeln!(f, "  Numeric Statistics:")?;
            writeln!(f, "    Min: {:.2}", stats.min)?;
            writeln!(f, "    Max: {:.2}", stats.max)?;
            writeln!(f, "    Mean: {:.2}", stats.mean)?;
            writeln!(f, "    Median: {:.2}", stats.median)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_analyzer() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,salary").unwrap();
        writeln!(temp_file, "1,Alice,30,50000").unwrap();
        writeln!(temp_file, "2,Bob,25,45000").unwrap();
        writeln!(temp_file, "3,Charlie,35,60000").unwrap();
        
        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(analyzer.row_count(), 3);
        assert_eq!(analyzer.column_count(), 4);
        
        let age_stats = analyzer.column_stats(2).unwrap();
        assert_eq!(age_stats.numeric_count, 3);
        assert_eq!(age_stats.string_count, 0);
        
        let filtered = analyzer.filter_by_column(1, &|name| name.starts_with('A'));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][1], "Alice");
    }
}