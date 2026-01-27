use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn filter_and_aggregate<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    target_category: &str,
    min_value: f64,
) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let mut total_value = 0.0;
    let mut record_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.category == target_category && record.value >= min_value && record.active {
            writer.serialize(&record)?;
            total_value += record.value;
            record_count += 1;
        }
    }

    writer.flush()?;

    if record_count > 0 {
        let average_value = total_value / record_count as f64;
        println!("Processed {} records", record_count);
        println!("Total value: {:.2}", total_value);
        println!("Average value: {:.2}", average_value);
    } else {
        println!("No records matched the criteria");
    }

    Ok(())
}

fn generate_sample_data<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;

    let sample_records = vec![
        Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 249.99, active: true },
        Record { id: 2, name: "Item B".to_string(), category: "Books".to_string(), value: 19.99, active: true },
        Record { id: 3, name: "Item C".to_string(), category: "Electronics".to_string(), value: 599.99, active: false },
        Record { id: 4, name: "Item D".to_string(), category: "Electronics".to_string(), value: 129.99, active: true },
        Record { id: 5, name: "Item E".to_string(), category: "Clothing".to_string(), value: 49.99, active: true },
    ];

    for record in sample_records {
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "sample_data.csv";
    let output_file = "filtered_data.csv";
    
    generate_sample_data(input_file)?;
    
    filter_and_aggregate(
        input_file,
        output_file,
        "Electronics",
        100.0
    )?;

    Ok(())
}use std::error::Error;
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
            let fields: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name);

        if let Some(idx) = column_index {
            self.records.iter()
                .filter(|record| {
                    if let Some(value) = record.get(idx) {
                        predicate(value)
                    } else {
                        false
                    }
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn aggregate_numeric_column(&self, group_by_column: &str, aggregate_column: &str) -> HashMap<String, f64> {
        let group_idx = self.headers.iter()
            .position(|h| h == group_by_column);
        let agg_idx = self.headers.iter()
            .position(|h| h == aggregate_column);

        if let (Some(group_idx), Some(agg_idx)) = (group_idx, agg_idx) {
            let mut result = HashMap::new();
            
            for record in &self.records {
                if let (Some(group_val), Some(agg_val)) = (record.get(group_idx), record.get(agg_idx)) {
                    if let Ok(num) = agg_val.parse::<f64>() {
                        *result.entry(group_val.clone()).or_insert(0.0) += num;
                    }
                }
            }
            result
        } else {
            HashMap::new()
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
        writeln!(file, "name,department,salary").unwrap();
        writeln!(file, "Alice,Engineering,75000").unwrap();
        writeln!(file, "Bob,Marketing,65000").unwrap();
        writeln!(file, "Charlie,Engineering,80000").unwrap();
        writeln!(file, "Diana,Marketing,70000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), &["name", "department", "salary"]);
        assert_eq!(processor.record_count(), 4);
    }

    #[test]
    fn test_filtering() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", |dept| dept == "Engineering");
        assert_eq!(engineering_records.len(), 2);
    }

    #[test]
    fn test_aggregation() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let totals = processor.aggregate_numeric_column("department", "salary");
        
        assert_eq!(totals.get("Engineering"), Some(&155000.0));
        assert_eq!(totals.get("Marketing"), Some(&135000.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_filtered_data(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

        for record in filtered {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn get_active_records(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.record_count(), 0);
        assert_eq!(processor.calculate_average(), None);
    }

    #[test]
    fn test_record_operations() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord {
            id: 1,
            category: "A".to_string(),
            value: 10.5,
            active: true,
        });

        processor.add_record(DataRecord {
            id: 2,
            category: "B".to_string(),
            value: 20.0,
            active: false,
        });

        assert_eq!(processor.record_count(), 2);
        assert_eq!(processor.calculate_average(), Some(15.25));
        assert_eq!(processor.filter_by_category("A").len(), 1);
        assert_eq!(processor.get_active_records().len(), 1);
    }
}