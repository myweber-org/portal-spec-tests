use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn new(id: u32, name: &str, category: &str, value: f64, active: bool) -> Self {
        Record {
            id,
            name: name.to_string(),
            category: category.to_string(),
            value,
            active,
        }
    }
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn export_filtered(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut wtr = Writer::from_path(output_path)?;
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    let sample_data = vec![
        Record::new(1, "ItemA", "Electronics", 249.99, true),
        Record::new(2, "ItemB", "Books", 19.99, true),
        Record::new(3, "ItemC", "Electronics", 599.99, false),
        Record::new(4, "ItemD", "Clothing", 49.99, true),
        Record::new(5, "ItemE", "Electronics", 129.99, true),
    ];
    
    processor.records = sample_data;
    
    println!("Average value: {:.2}", processor.calculate_average());
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Active Electronics items: {}", electronics.len());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Highest value item: {} - ${:.2}", max_record.name, max_record.value);
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Processing error: {}", e);
    }
}use std::error::Error;
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
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            self.records
                .iter()
                .filter(|record| record.get(col_index) == Some(&value.to_string()))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            let sum: f64 = self.records
                .iter()
                .filter_map(|record| record.get(col_index).and_then(|s| s.parse::<f64>().ok()))
                .sum();
            Some(sum)
        } else {
            None
        }
    }

    pub fn group_by_column(&self, group_column: &str, aggregate_column: &str) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        
        if let (Some(group_idx), Some(agg_idx)) = (
            self.headers.iter().position(|h| h == group_column),
            self.headers.iter().position(|h| h == aggregate_column)
        ) {
            for record in &self.records {
                if let (Some(group_val), Some(agg_val)) = (
                    record.get(group_idx),
                    record.get(agg_idx).and_then(|s| s.parse::<f64>().ok())
                ) {
                    *result.entry(group_val.clone()).or_insert(0.0) += agg_val;
                }
            }
        }
        
        result
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &Vec<String> {
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
        writeln!(file, "name,department,salary").unwrap();
        writeln!(file, "Alice,Engineering,75000").unwrap();
        writeln!(file, "Bob,Marketing,65000").unwrap();
        writeln!(file, "Charlie,Engineering,80000").unwrap();
        writeln!(file, "Diana,Marketing,70000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), &vec!["name", "department", "salary"]);
        assert_eq!(processor.get_record_count(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
    }

    #[test]
    fn test_aggregate_numeric() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let total_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert_eq!(total_salary, 290000.0);
    }

    #[test]
    fn test_group_by() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let dept_salaries = processor.group_by_column("department", "salary");
        assert_eq!(dept_salaries.get("Engineering"), Some(&155000.0));
        assert_eq!(dept_salaries.get("Marketing"), Some(&135000.0));
    }
}