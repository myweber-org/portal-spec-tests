use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn read_and_filter(&self, column_index: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let record: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            
            if record.get(column_index).map_or(false, |value| value == filter_value) {
                filtered_records.push(record);
            }
        }

        Ok(filtered_records)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        Ok(count)
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
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,alice,30").unwrap();
        writeln!(temp_file, "2,bob,25").unwrap();
        writeln!(temp_file, "3,alice,35").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let filtered = processor.read_and_filter(1, "alice").unwrap();
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "1");
        assert_eq!(filtered[1][0], "3");
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

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let record = Record {
            id: parts[0].parse()?,
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse().unwrap_or(false),
        };
        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.category == category && r.active)
        .collect()
}

pub fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record {
                id: 1,
                category: "A".to_string(),
                value: 10.5,
                active: true,
            },
            Record {
                id: 2,
                category: "B".to_string(),
                value: 20.0,
                active: true,
            },
            Record {
                id: 3,
                category: "A".to_string(),
                value: 15.0,
                active: false,
            },
        ];

        let filtered = filter_by_category(&records, "A");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record {
                id: 1,
                category: "Test".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                category: "Test".to_string(),
                value: 20.0,
                active: true,
            },
            Record {
                id: 3,
                category: "Test".to_string(),
                value: 30.0,
                active: true,
            },
        ];

        let refs: Vec<&Record> = records.iter().collect();
        let avg = calculate_average(&refs).unwrap();
        assert_eq!(avg, 20.0);
    }
}use csv::{Reader, Writer};
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

#[derive(Debug)]
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
}

fn read_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn aggregate_by_category(records: &[Record]) -> Vec<AggregatedData> {
    use std::collections::HashMap;

    let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_map.entry(record.category.clone()).or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_map
        .into_iter()
        .map(|(category, (total, count))| AggregatedData {
            category,
            total_value: total,
            average_value: total / count as f64,
            record_count: count,
        })
        .collect()
}

fn write_aggregated_data<P: AsRef<Path>>(
    aggregated: &[AggregatedData],
    path: P,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = Writer::from_writer(file);

    for data in aggregated {
        writer.serialize(data)?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = read_csv_file(input_path)?;
    let active_records = filter_active_records(&records);
    let aggregated_data = aggregate_by_category(&active_records);
    write_aggregated_data(&aggregated_data, output_path)?;

    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Generated {} aggregated categories", aggregated_data.len());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";

    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Category1".to_string(),
                value: 10.5,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Category2".to_string(),
                value: 20.0,
                active: false,
            },
        ];

        let active = filter_active_records(&records);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Category1".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Category1".to_string(),
                value: 20.0,
                active: true,
            },
        ];

        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].total_value, 30.0);
        assert_eq!(aggregated[0].average_value, 15.0);
        assert_eq!(aggregated[0].record_count, 2);
    }

    #[test]
    fn test_csv_roundtrip() -> Result<(), Box<dyn Error>> {
        let records = vec![Record {
            id: 1,
            name: "Test Item".to_string(),
            category: "Test Category".to_string(),
            value: 42.0,
            active: true,
        }];

        let temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;

        let mut writer = Writer::from_writer(&temp_input);
        for record in &records {
            writer.serialize(record)?;
        }
        writer.flush()?;

        process_csv_data(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
        )?;

        Ok(())
    }
}use std::error::Error;
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
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

    pub fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|r| r.category.clone())
            .collect();
        
        categories.sort();
        categories.dedup();
        categories
    }

    pub fn total_records(&self) -> usize {
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,15.0,Books").unwrap();
        writeln!(temp_file, "3,ItemC,8.75,Electronics").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(file_path);
        assert!(result.is_ok());
        assert_eq!(processor.total_records(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average_value();
        assert!((avg - 11.416666).abs() < 0.001);
        
        let max_record = processor.find_max_value_record();
        assert_eq!(max_record.unwrap().name, "ItemB");
        
        let categories = processor.get_unique_categories();
        assert_eq!(categories, vec!["Books", "Electronics"]);
    }
}