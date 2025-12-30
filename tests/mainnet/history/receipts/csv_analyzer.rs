
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub name: String,
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
}

pub struct CsvAnalyzer {
    data: Vec<Vec<f64>>,
    headers: Vec<String>,
}

impl CsvAnalyzer {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
        
        let mut data = Vec::new();
        for _ in 0..headers.len() {
            data.push(Vec::new());
        }
        
        for result in rdr.records() {
            let record = result?;
            for (i, field) in record.iter().enumerate() {
                if let Ok(value) = field.parse::<f64>() {
                    data[i].push(value);
                }
            }
        }
        
        Ok(CsvAnalyzer { data, headers })
    }
    
    pub fn analyze_columns(&self) -> Vec<ColumnStats> {
        self.headers.iter().enumerate().filter_map(|(i, header)| {
            let column_data = &self.data[i];
            if column_data.is_empty() {
                return None;
            }
            
            let count = column_data.len();
            let sum: f64 = column_data.iter().sum();
            let mean = sum / count as f64;
            
            let min = column_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = column_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            let variance: f64 = column_data.iter()
                .map(|value| {
                    let diff = mean - value;
                    diff * diff
                })
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();
            
            Some(ColumnStats {
                name: header.clone(),
                count,
                mean,
                min,
                max,
                std_dev,
            })
        }).collect()
    }
    
    pub fn correlation_matrix(&self) -> Vec<Vec<f64>> {
        let n = self.data.len();
        let mut matrix = vec![vec![0.0; n]; n];
        
        for i in 0..n {
            matrix[i][i] = 1.0;
            for j in (i + 1)..n {
                let corr = self.calculate_correlation(&self.data[i], &self.data[j]);
                matrix[i][j] = corr;
                matrix[j][i] = corr;
            }
        }
        
        matrix
    }
    
    fn calculate_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }
        
        let n = x.len() as f64;
        let mean_x: f64 = x.iter().sum::<f64>() / n;
        let mean_y: f64 = y.iter().sum::<f64>() / n;
        
        let mut numerator = 0.0;
        let mut denom_x = 0.0;
        let mut denom_y = 0.0;
        
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            let diff_x = xi - mean_x;
            let diff_y = yi - mean_y;
            numerator += diff_x * diff_y;
            denom_x += diff_x * diff_x;
            denom_y += diff_y * diff_y;
        }
        
        if denom_x == 0.0 || denom_y == 0.0 {
            return 0.0;
        }
        
        numerator / (denom_x.sqrt() * denom_y.sqrt())
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
        writeln!(temp_file, "col1,col2,col3").unwrap();
        writeln!(temp_file, "1.0,2.0,3.0").unwrap();
        writeln!(temp_file, "4.0,5.0,6.0").unwrap();
        writeln!(temp_file, "7.0,8.0,9.0").unwrap();
        
        let analyzer = CsvAnalyzer::from_file(temp_file.path()).unwrap();
        let stats = analyzer.analyze_columns();
        
        assert_eq!(stats.len(), 3);
        assert_eq!(stats[0].name, "col1");
        assert_eq!(stats[0].count, 3);
        assert!((stats[0].mean - 4.0).abs() < 0.001);
    }
}