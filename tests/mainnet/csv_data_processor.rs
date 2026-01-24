use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let line = line?;
            let record: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records.iter()
            .filter(|record| record.get(column_index).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, Box<dyn Error>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Err("Column not found".into()),
        };

        let mut total = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    total += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(total / count as f64)
        } else {
            Err("No numeric values found".into())
        }
    }

    pub fn group_by_column(&self, group_column: &str, agg_column: &str) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let group_idx = match self.headers.iter().position(|h| h == group_column) {
            Some(idx) => idx,
            None => return Err("Group column not found".into()),
        };

        let agg_idx = match self.headers.iter().position(|h| h == agg_column) {
            Some(idx) => idx,
            None => return Err("Aggregation column not found".into()),
        };

        let mut groups: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            if let (Some(group_val), Some(agg_val_str)) = (record.get(group_idx), record.get(agg_idx)) {
                if let Ok(agg_val) = agg_val_str.parse::<f64>() {
                    let entry = groups.entry(group_val.clone())
                        .or_insert((0.0, 0));
                    entry.0 += agg_val;
                    entry.1 += 1;
                }
            }
        }

        let result: HashMap<String, f64> = groups.into_iter()
            .map(|(key, (sum, count))| (key, sum / count as f64))
            .collect();

        Ok(result)
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

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary,department").unwrap();
        writeln!(temp_file, "Alice,30,50000,Engineering").unwrap();
        writeln!(temp_file, "Bob,25,45000,Marketing").unwrap();
        writeln!(temp_file, "Charlie,35,60000,Engineering").unwrap();
        writeln!(temp_file, "Diana,28,48000,HR").unwrap();
        temp_file
    }

    #[test]
    fn test_csv_loading() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), &["name", "age", "salary", "department"]);
        assert_eq!(processor.get_record_count(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let marketing_records = processor.filter_by_column("department", "Marketing");
        assert_eq!(marketing_records.len(), 1);
    }

    #[test]
    fn test_aggregate_numeric() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let avg_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert!((avg_salary - 50750.0).abs() < 0.01);
    }

    #[test]
    fn test_group_by_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let dept_avg_salary = processor.group_by_column("department", "salary").unwrap();
        assert!((dept_avg_salary["Engineering"] - 55000.0).abs() < 0.01);
        assert!((dept_avg_salary["Marketing"] - 45000.0).abs() < 0.01);
        assert!((dept_avg_salary["HR"] - 48000.0).abs() < 0.01);
    }
}