
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvProcessor {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl CsvProcessor {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
        let mut data = Vec::new();
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            data.push(row);
        }
        
        Ok(CsvProcessor { data, headers })
    }
    
    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data
            .iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }
    
    pub fn aggregate_column(&self, column_index: usize, operation: &str) -> Option<f64> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let values: Vec<f64> = self.data
            .iter()
            .filter_map(|row| row.get(column_index).and_then(|s| s.parse::<f64>().ok()))
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        match operation {
            "sum" => Some(values.iter().sum()),
            "avg" => Some(values.iter().sum::<f64>() / values.len() as f64),
            "max" => values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_processing() {
        let csv_content = "name,age,salary\nAlice,30,50000\nBob,25,45000\nCharlie,35,60000";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let processor = CsvProcessor::from_file(temp_file.path()).unwrap();
        assert_eq!(processor.row_count(), 3);
        assert_eq!(processor.column_count(), 3);
        
        let filtered = processor.filter_rows(|row| {
            row.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age > 30)
        });
        assert_eq!(filtered.len(), 1);
        
        let avg_salary = processor.aggregate_column(2, "avg");
        assert!(avg_salary.is_some());
        assert!((avg_salary.unwrap() - 51666.666).abs() < 0.001);
    }
}