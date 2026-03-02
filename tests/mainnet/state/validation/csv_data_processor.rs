use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            if index == 0 {
                continue;
            }
            
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 4 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    value: parts[2].parse()?,
                    category: parts[3].to_string(),
                };
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

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn find_max_value_record(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,42.0,Books").unwrap();
        writeln!(temp_file, "3,ItemC,18.75,Electronics").unwrap();
        
        let filepath = temp_file.path().to_str().unwrap();
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(filepath).unwrap();
        
        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.filter_by_category("Electronics").len(), 2);
        
        let avg = processor.calculate_average_value();
        assert!((avg - 28.75).abs() < 0.01);
        
        let max_record = processor.find_max_value_record().unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 42.0);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|r| r.active)
        .collect()
}

fn calculate_category_averages(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();
    
    for record in records {
        let entry = category_totals
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }
    
    category_totals
        .into_iter()
        .map(|(category, (total, count))| (category, total / count as f64))
        .collect()
}

fn save_processed_data<P: AsRef<Path>>(
    records: &[&Record],
    averages: &[(String, f64)],
    output_path: P
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_path)?;
    
    writer.write_record(&["ID", "Name", "Category", "Value", "Active"])?;
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.write_record(&[])?;
    writer.write_record(&["Category", "Average Value"])?;
    for (category, avg) in averages {
        writer.write_record(&[category, &avg.to_string()])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    let active_records = filter_active_records(&records);
    let category_averages = calculate_category_averages(&records);
    
    save_processed_data(&active_records, &category_averages, output_path)?;
    
    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Calculated averages for {} categories", category_averages.len());
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed_output.csv";
    
    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
    
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

    pub fn filter_by_column(&self, column_name: &str, predicate: fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name);

        match column_index {
            Some(idx) => self.records.iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: Aggregation) -> Result<f64, String> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let numeric_values: Vec<f64> = self.records.iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return Err("No valid numeric data found".into());
        }

        match operation {
            Aggregation::Sum => Ok(numeric_values.iter().sum()),
            Aggregation::Average => Ok(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            Aggregation::Max => Ok(numeric_values.iter().fold(f64::MIN, |a, &b| a.max(b))),
            Aggregation::Min => Ok(numeric_values.iter().fold(f64::MAX, |a, &b| a.min(b))),
        }
    }

    pub fn get_column_stats(&self, column_name: &str) -> Result<ColumnStats, String> {
        let values: Vec<&str> = self.records.iter()
            .filter_map(|record| {
                let idx = self.headers.iter().position(|h| h == column_name)?;
                Some(record[idx].as_str())
            })
            .collect();

        if values.is_empty() {
            return Err("Column not found or empty".into());
        }

        let unique_count = values.iter().collect::<std::collections::HashSet<_>>().len();
        let null_count = values.iter().filter(|&&v| v.is_empty()).count();

        Ok(ColumnStats {
            total_count: values.len(),
            unique_count,
            null_count,
            sample_values: values.iter().take(3).map(|&s| s.to_string()).collect(),
        })
    }
}

pub enum Aggregation {
    Sum,
    Average,
    Max,
    Min,
}

pub struct ColumnStats {
    pub total_count: usize,
    pub unique_count: usize,
    pub null_count: usize,
    pub sample_values: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary,department").unwrap();
        writeln!(file, "Alice,30,50000,Engineering").unwrap();
        writeln!(file, "Bob,25,45000,Marketing").unwrap();
        writeln!(file, "Charlie,35,60000,Engineering").unwrap();
        writeln!(file, "Diana,28,48000,Sales").unwrap();
        writeln!(file, "Eve,40,55000,Engineering").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.headers, vec!["name", "age", "salary", "department"]);
        assert_eq!(processor.records.len(), 5);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", |dept| dept == "Engineering");
        assert_eq!(engineering_records.len(), 3);
        
        let high_salary = processor.filter_by_column("salary", |salary| {
            salary.parse::<i32>().unwrap_or(0) > 50000
        });
        assert_eq!(high_salary.len(), 2);
    }

    #[test]
    fn test_aggregation() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let total_salary = processor.aggregate_numeric_column("salary", Aggregation::Sum).unwrap();
        assert_eq!(total_salary, 258000.0);
        
        let avg_age = processor.aggregate_numeric_column("age", Aggregation::Average).unwrap();
        assert_eq!(avg_age, 31.6);
    }

    #[test]
    fn test_column_stats() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let stats = processor.get_column_stats("department").unwrap();
        assert_eq!(stats.total_count, 5);
        assert_eq!(stats.unique_count, 3);
        assert_eq!(stats.null_count, 0);
    }
}