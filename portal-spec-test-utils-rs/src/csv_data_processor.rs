
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: CsvRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.find_max_value().map(|r| r.value);

        (count, avg, max)
    }

    pub fn add_record(&mut self, record: CsvRecord) {
        self.records.push(record);
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor() {
        let mut processor = CsvProcessor::new();

        let record1 = CsvRecord {
            id: 1,
            name: "Item A".to_string(),
            value: 10.5,
            category: "Electronics".to_string(),
        };

        let record2 = CsvRecord {
            id: 2,
            name: "Item B".to_string(),
            value: 25.0,
            category: "Books".to_string(),
        };

        processor.add_record(record1);
        processor.add_record(record2);

        assert_eq!(processor.records.len(), 2);
        assert!(processor.calculate_average().is_some());
    }

    #[test]
    fn test_filtering() {
        let mut processor = CsvProcessor::new();

        let records = vec![
            CsvRecord {
                id: 1,
                name: "Test1".to_string(),
                value: 100.0,
                category: "A".to_string(),
            },
            CsvRecord {
                id: 2,
                name: "Test2".to_string(),
                value: 200.0,
                category: "B".to_string(),
            },
            CsvRecord {
                id: 3,
                name: "Test3".to_string(),
                value: 300.0,
                category: "A".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record);
        }

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        if line_number == 1 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() == 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();
            
            records.push(Record {
                id,
                name,
                value,
                category,
            });
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_average(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut category_totals: HashMap<String, f64> = HashMap::new();
    
    for record in records {
        *category_totals.entry(record.category.clone()).or_insert(0.0) += record.value;
    }
    
    category_totals.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        
        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.3);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let filtered = filter_by_category(&records, "Cat1");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "Cat1"));
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let avg = calculate_average(&records);
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 30.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 20.0, category: "Cat1".to_string() },
        ];
        
        let max_record = find_max_value(&records).unwrap();
        assert_eq!(max_record.value, 30.0);
        assert_eq!(max_record.name, "B");
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 2);
        
        let cat1_total: f64 = aggregated.iter()
            .find(|(cat, _)| cat == "Cat1")
            .map(|(_, total)| *total)
            .unwrap_or(0.0);
        
        assert_eq!(cat1_total, 40.0);
    }
}