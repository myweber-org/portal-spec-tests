use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers: Vec<String> = if let Some(first_line) = lines.next() {
            first_line?.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            return Err("Empty CSV file".into());
        };
        
        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?.split(',').map(|s| s.trim().to_string()).collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }
        
        Ok(CsvProcessor { headers, records })
    }
    
    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        
        if let Some(idx) = column_index {
            self.records.iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Result<f64, String> {
        let column_index = self.headers.iter().position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        let numeric_values: Vec<f64> = self.records.iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .collect();
        
        if numeric_values.is_empty() {
            return Err("No valid numeric values found".into());
        }
        
        match operation {
            "sum" => Ok(numeric_values.iter().sum()),
            "avg" => Ok(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => Ok(numeric_values.iter().fold(f64::MIN, |a, &b| a.max(b))),
            "min" => Ok(numeric_values.iter().fold(f64::MAX, |a, &b| a.min(b))),
            _ => Err(format!("Unsupported operation: {}", operation))
        }
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
    
    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "Charlie,35,60000").unwrap();
        
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.record_count(), 3);
        
        let filtered = processor.filter_by_column("age", |age| age.parse::<i32>().unwrap() > 30);
        assert_eq!(filtered.len(), 1);
        
        let avg_salary = processor.aggregate_numeric_column("salary", "avg").unwrap();
        assert!((avg_salary - 51666.666).abs() < 0.001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Record {
            id,
            category,
            value,
            active,
        }
    }
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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            if index == 0 {
                continue;
            }

            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() == 4 {
                let id = parts[0].parse::<u32>()?;
                let category = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let active = parts[3].parse::<bool>()?;
                
                self.records.push(Record::new(id, category, value, active));
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            *counts.entry(record.category.clone()).or_insert(0) += 1;
        }
        
        counts
    }

    pub fn get_total_records(&self) -> usize {
        self.records.len()
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
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,299.99,true").unwrap();
        writeln!(temp_file, "2,books,19.99,true").unwrap();
        writeln!(temp_file, "3,electronics,599.99,false").unwrap();
        
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_total_records(), 3);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average > 0.0);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 3);
        
        let counts = processor.count_by_category();
        assert_eq!(counts.get("electronics"), Some(&2));
        assert_eq!(counts.get("books"), Some(&1));
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let mut total_records = 0;
    let mut filtered_records = 0;
    let mut sum_values = 0.0;

    for result in reader.deserialize() {
        let record: Record = result?;
        total_records += 1;

        if record.value >= min_value && record.active {
            writer.serialize(&record)?;
            filtered_records += 1;
            sum_values += record.value;
        }
    }

    writer.flush()?;

    if filtered_records > 0 {
        let average_value = sum_values / filtered_records as f64;
        println!("Processed {} records", total_records);
        println!("Filtered {} records with value >= {}", filtered_records, min_value);
        println!("Average value of filtered records: {:.2}", average_value);
    } else {
        println!("No records matched the filter criteria");
    }

    Ok(())
}

fn aggregate_by_category(input_path: &str) -> Result<Vec<(String, f64)>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut category_totals = std::collections::HashMap::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.active {
            let entry = category_totals.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value;
        }
    }

    let mut results: Vec<(String, f64)> = category_totals.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    Ok(results)
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            category: "A".to_string(),
            value: 10.5,
            active: true,
        };

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            category: "B".to_string(),
            value: -5.0,
            active: true,
        };

        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }

    #[test]
    fn test_aggregate_by_category() -> Result<(), Box<dyn Error>> {
        let test_data = "id,name,category,value,active\n1,Item1,CategoryA,10.5,true\n2,Item2,CategoryB,15.0,true\n3,Item3,CategoryA,5.5,true\n4,Item4,CategoryB,20.0,false";
        
        let mut temp_file = NamedTempFile::new()?;
        std::io::Write::write_all(&mut temp_file, test_data.as_bytes())?;
        
        let results = aggregate_by_category(temp_file.path().to_str().unwrap())?;
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "CategoryB");
        assert_eq!(results[0].1, 15.0);
        
        Ok(())
    }
}