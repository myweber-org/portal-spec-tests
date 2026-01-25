use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
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
        
        Ok(CsvProcessor { headers, records })
    }
    
    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };
        
        self.records.iter()
            .filter(|record| {
                if let Some(value) = record.get(column_index) {
                    predicate(value)
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    pub fn aggregate_numeric_column(&self, group_by_column: &str, aggregate_column: &str) -> HashMap<String, f64> {
        let group_idx = self.headers.iter().position(|h| h == group_by_column);
        let agg_idx = self.headers.iter().position(|h| h == aggregate_column);
        
        if group_idx.is_none() || agg_idx.is_none() {
            return HashMap::new();
        }
        
        let group_idx = group_idx.unwrap();
        let agg_idx = agg_idx.unwrap();
        
        let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();
        
        for record in &self.records {
            if let (Some(group_val), Some(agg_val)) = (record.get(group_idx), record.get(agg_idx)) {
                if let Ok(num) = agg_val.parse::<f64>() {
                    let entry = aggregates.entry(group_val.clone())
                        .or_insert((0.0, 0));
                    entry.0 += num;
                    entry.1 += 1;
                }
            }
        }
        
        aggregates.into_iter()
            .map(|(key, (sum, count))| (key, sum / count as f64))
            .collect()
    }
    
    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,30,60000").unwrap();
        
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.get_record_count(), 3);
        
        let filtered = processor.filter_by_column("age", |age| age == "30");
        assert_eq!(filtered.len(), 2);
        
        let aggregates = processor.aggregate_numeric_column("age", "salary");
        assert_eq!(aggregates.get("30").unwrap(), &55000.0);
    }
}