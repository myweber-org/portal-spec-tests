
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub columns: Vec<String>,
    pub values: Vec<f64>,
}

pub struct CsvAnalyzer {
    records: Vec<CsvRecord>,
    headers: Vec<String>,
}

impl CsvAnalyzer {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
        let mut records = Vec::new();
        
        for result in rdr.records() {
            let record = result?;
            let values: Vec<f64> = record.iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            
            if values.len() == headers.len() {
                records.push(CsvRecord {
                    columns: headers.clone(),
                    values,
                });
            }
        }
        
        Ok(CsvAnalyzer { records, headers })
    }
    
    pub fn column_stats(&self, column_index: usize) -> Option<ColumnStatistics> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|r| r.values.get(column_index).copied())
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Some(ColumnStatistics {
            mean,
            std_dev,
            min,
            max,
            count: values.len(),
        })
    }
    
    pub fn filter_records<F>(&self, predicate: F) -> Vec<CsvRecord>
    where
        F: Fn(&CsvRecord) -> bool,
    {
        self.records.iter()
            .filter(|r| predicate(r))
            .cloned()
            .collect()
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
    
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl ColumnStatistics {
    pub fn display(&self) -> String {
        format!(
            "Count: {}, Mean: {:.2}, StdDev: {:.2}, Range: [{:.2}, {:.2}]",
            self.count, self.mean, self.std_dev, self.min, self.max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "col1,col2,col3").unwrap();
        writeln!(temp_file, "1.0,2.0,3.0").unwrap();
        writeln!(temp_file, "4.0,5.0,6.0").unwrap();
        
        let analyzer = CsvAnalyzer::load_from_file(temp_file.path()).unwrap();
        assert_eq!(analyzer.record_count(), 2);
        assert_eq!(analyzer.get_headers(), vec!["col1", "col2", "col3"]);
    }
    
    #[test]
    fn test_column_stats() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "value").unwrap();
        writeln!(temp_file, "10.0").unwrap();
        writeln!(temp_file, "20.0").unwrap();
        writeln!(temp_file, "30.0").unwrap();
        
        let analyzer = CsvAnalyzer::load_from_file(temp_file.path()).unwrap();
        let stats = analyzer.column_stats(0).unwrap();
        
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
        assert_eq!(stats.count, 3);
    }
}