
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvProcessor {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl CsvProcessor {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
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
    
    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };
        
        self.data
            .iter()
            .filter(|row| row.get(column_index).map_or(false, |cell| cell == value))
            .cloned()
            .collect()
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, Box<dyn Error>> {
        let column_index = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        let mut sum = 0.0;
        let mut count = 0;
        
        for row in &self.data {
            if let Some(cell) = row.get(column_index) {
                if let Ok(value) = cell.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Err("No numeric values found in column".into())
        }
    }
    
    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };
        
        let mut unique_values = std::collections::HashSet::new();
        for row in &self.data {
            if let Some(value) = row.get(column_index) {
                unique_values.insert(value.clone());
            }
        }
        
        unique_values.into_iter().collect()
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
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,New York").unwrap();
        
        let processor = CsvProcessor::new(temp_file.path()).unwrap();
        assert_eq!(processor.row_count(), 3);
        assert_eq!(processor.column_count(), 3);
        
        let ny_residents = processor.filter_by_column("city", "New York");
        assert_eq!(ny_residents.len(), 2);
        
        let unique_cities = processor.get_unique_values("city");
        assert_eq!(unique_cities.len(), 2);
    }
}