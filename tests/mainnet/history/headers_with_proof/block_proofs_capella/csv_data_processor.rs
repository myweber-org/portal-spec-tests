
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_active_records(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            *counts.entry(record.category.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,99.99,true").unwrap();
        writeln!(temp_file, "2,books,24.50,false").unwrap();
        writeln!(temp_file, "3,electronics,149.99,true").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.filter_by_category("electronics").len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!(avg > 91.0 && avg < 92.0);
        
        assert_eq!(processor.get_active_records().len(), 2);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 3);
        
        let counts = processor.count_by_category();
        assert_eq!(counts.get("electronics"), Some(&2));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
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

        let mut data = Vec::new();
        for line in lines {
            let line = line?;
            let row: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if row.len() == headers.len() {
                data.push(row);
            }
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
            .filter(|row| row.get(column_index).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, Box<dyn Error>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Err("Column not found".into()),
        };

        let mut sum = 0.0;
        let mut count = 0;

        for row in &self.data {
            if let Some(value_str) = row.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Err("No numeric values found".into())
        }
    }

    pub fn get_row_count(&self) -> usize {
        self.data.len()
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
        
        assert_eq!(processor.get_headers(), &vec!["name", "age", "salary"]);
        assert_eq!(processor.get_row_count(), 3);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("age", "30");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Charlie");
    }

    #[test]
    fn test_aggregate_numeric() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let avg_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert!((avg_salary - 50000.0).abs() < 0.01);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    id: u32,
    customer: String,
    amount: f64,
    category: String,
    status: String,
}

struct TransactionProcessor {
    transactions: Vec<Transaction>,
}

impl TransactionProcessor {
    fn new() -> Self {
        TransactionProcessor {
            transactions: Vec::new(),
        }
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let transaction: Transaction = result?;
            self.transactions.push(transaction);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    fn filter_by_status(&self, status: &str) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| t.status == status)
            .collect()
    }

    fn calculate_total_amount(&self) -> f64 {
        self.transactions.iter().map(|t| t.amount).sum()
    }

    fn calculate_average_amount(&self) -> f64 {
        if self.transactions.is_empty() {
            return 0.0;
        }
        self.calculate_total_amount() / self.transactions.len() as f64
    }

    fn get_customer_summary(&self) -> Vec<(String, f64)> {
        let mut summary = std::collections::HashMap::new();
        
        for transaction in &self.transactions {
            let entry = summary.entry(transaction.customer.clone()).or_insert(0.0);
            *entry += transaction.amount;
        }
        
        summary.into_iter().collect()
    }

    fn save_filtered_transactions(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);
        
        for transaction in filtered {
            wtr.serialize(transaction)?;
        }
        
        wtr.flush()?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = TransactionProcessor::new();
    
    processor.load_from_file("transactions.csv")?;
    
    println!("Total transactions: {}", processor.transactions.len());
    println!("Total amount: ${:.2}", processor.calculate_total_amount());
    println!("Average transaction: ${:.2}", processor.calculate_average_amount());
    
    let completed = processor.filter_by_status("completed");
    println!("Completed transactions: {}", completed.len());
    
    let customer_summary = processor.get_customer_summary();
    println!("Customer summary:");
    for (customer, total) in customer_summary {
        println!("  {}: ${:.2}", customer, total);
    }
    
    processor.save_filtered_transactions("electronics.csv", "electronics")?;
    println!("Filtered transactions saved to electronics.csv");
    
    Ok(())
}use std::error::Error;
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

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| predicate(&record[column_index]))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;

        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        match operation {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
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

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,age,salary").unwrap();
        writeln!(file, "1,Alice,30,50000").unwrap();
        writeln!(file, "2,Bob,25,45000").unwrap();
        writeln!(file, "3,Charlie,35,60000").unwrap();
        writeln!(file, "4,Diana,28,55000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), &["id", "name", "age", "salary"]);
        assert_eq!(processor.record_count(), 4);
    }

    #[test]
    fn test_filtering() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let adults = processor.filter_by_column("age", |age| age.parse::<i32>().unwrap() >= 30);
        assert_eq!(adults.len(), 2);
    }

    #[test]
    fn test_aggregation() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let total_salary = processor.aggregate_numeric_column("salary", "sum");
        assert_eq!(total_salary, Some(210000.0));
        
        let avg_age = processor.aggregate_numeric_column("age", "avg");
        assert_eq!(avg_age, Some(29.5));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();
            
            let record = CsvRecord::new(id, name, value, category);
            if record.validate().is_ok() {
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn apply_transformation(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform_value(multiplier);
        }
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn add_record(&mut self, record: CsvRecord) -> Result<(), String> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "CategoryA".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord::new(2, "".to_string(), -50.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = CsvRecord::new(1, "Test".to_string(), 100.0, "CategoryA".to_string());
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
    }

    #[test]
    fn test_processor_operations() {
        let mut processor = CsvProcessor::new();
        
        let record1 = CsvRecord::new(1, "Item1".to_string(), 100.0, "CategoryA".to_string());
        let record2 = CsvRecord::new(2, "Item2".to_string(), 200.0, "CategoryB".to_string());
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        assert_eq!(processor.calculate_total_value(), 300.0);
        assert_eq!(processor.filter_by_category("CategoryA").len(), 1);
    }
}