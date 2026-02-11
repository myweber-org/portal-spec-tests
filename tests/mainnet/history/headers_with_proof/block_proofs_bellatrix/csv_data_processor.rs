use csv::{Reader, Writer};
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

fn aggregate_by_category(input_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut category_totals = std::collections::HashMap::new();
    let mut category_counts = std::collections::HashMap::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.active {
            let total = category_totals.entry(record.category.clone()).or_insert(0.0);
            *total += record.value;
            
            let count = category_counts.entry(record.category).or_insert(0);
            *count += 1;
        }
    }

    println!("Aggregation by category:");
    for (category, total) in category_totals {
        if let Some(count) = category_counts.get(&category) {
            let average = total / *count as f64;
            println!("Category: {}, Total: {:.2}, Count: {}, Average: {:.2}", 
                    category, total, count, average);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/filtered_output.csv";
    
    match process_csv(input_file, output_file, 50.0) {
        Ok(_) => {
            println!("CSV processing completed successfully");
            aggregate_by_category(output_file)?;
        }
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
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
            let record: Vec<String> = line?
                .split(',')
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

        self.records
            .iter()
            .filter(|record| record.get(column_index).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, Box<dyn Error>> {
        let column_index = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or("Column not found")?;

        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
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
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary").unwrap();
        writeln!(file, "Alice,30,50000").unwrap();
        writeln!(file, "Bob,25,45000").unwrap();
        writeln!(file, "Charlie,30,55000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), &["name", "age", "salary"]);
        assert_eq!(processor.get_record_count(), 3);
    }

    #[test]
    fn test_filtering() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("age", "30");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Charlie");
    }

    #[test]
    fn test_aggregation() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let avg_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert!((avg_salary - 50000.0).abs() < 0.01);
    }
}