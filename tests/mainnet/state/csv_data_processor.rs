use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_headers {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, String> {
        let mut numeric_values = Vec::new();
        
        for (row_num, record) in records.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!("Row {}: Column index out of bounds", row_num + 1));
            }
            
            match record[column_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", 
                    row_num + 1, record[column_index])),
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records.iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
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
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.5").unwrap();
        writeln!(temp_file, "Charlie,35,55000.75").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let records = processor.read_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
        
        let ages = processor.validate_numeric_column(&records, 1).unwrap();
        assert_eq!(ages, vec![30.0, 25.0, 35.0]);
        
        let (mean, variance, std_dev) = processor.calculate_statistics(&ages);
        assert!((mean - 30.0).abs() < 0.001);
        
        let filtered = processor.filter_records(&records, |record| {
            record[0].starts_with('A') || record[0].starts_with('B')
        });
        assert_eq!(filtered.len(), 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 5 {
                continue;
            }

            let record = Record {
                id: parts[0].parse().unwrap_or(0),
                name: parts[1].to_string(),
                category: parts[2].to_string(),
                value: parts[3].parse().unwrap_or(0.0),
                active: parts[4].trim().eq_ignore_ascii_case("true"),
            };

            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn get_top_records(&self, limit: usize) -> Vec<Record> {
        let mut sorted = self.records.clone();
        sorted.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        sorted.into_iter().take(limit).collect()
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            *counts.entry(record.category.clone()).or_insert(0) += 1;
        }
        
        counts
    }

    pub fn merge_records(&mut self, other: Vec<Record>) {
        self.records.extend(other);
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert!(processor.is_empty());

        let test_data = "id,name,category,value,active\n\
                         1,ItemA,Electronics,100.50,true\n\
                         2,ItemB,Furniture,75.25,false\n\
                         3,ItemC,Electronics,150.00,true\n";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", test_data).unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.len(), 3);

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 2);

        let total = processor.calculate_total_value();
        assert!((total - 325.75).abs() < 0.01);

        let average = processor.calculate_average_value();
        assert!((average - 108.583).abs() < 0.01);

        let top = processor.get_top_records(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].name, "ItemC");

        let counts = processor.count_by_category();
        assert_eq!(counts.get("Electronics"), Some(&2));
        assert_eq!(counts.get("Furniture"), Some(&1));
    }
}