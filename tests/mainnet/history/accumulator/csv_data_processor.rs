use std::error::Error;
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
        
        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let mut records = Vec::new();
        for line in lines {
            let line = line?;
            let record: Vec<String> = line
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
        
        self.records.iter()
            .filter(|record| {
                if let Some(value) = record.get(column_index) {
                    predicate(value)
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    pub fn aggregate_numeric_column(&self, group_by_column: &str, aggregate_column: &str) -> HashMap<String, f64> {
        let group_idx = self.headers.iter().position(|h| h == group_by_column);
        let agg_idx = self.headers.iter().position(|h| h == aggregate_column);
        
        if group_idx.is_none() || agg_idx.is_none() {
            return HashMap::new();
        }
        
        let group_idx = group_idx.unwrap();
        let agg_idx = agg_idx.unwrap();
        
        let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();
        
        for record in &self.records {
            if let (Some(group_val), Some(agg_val)) = (record.get(group_idx), record.get(agg_idx)) {
                if let Ok(num) = agg_val.parse::<f64>() {
                    let entry = aggregates.entry(group_val.clone())
                        .or_insert((0.0, 0));
                    entry.0 += num;
                    entry.1 += 1;
                }
            }
        }
        
        aggregates.into_iter()
            .map(|(key, (sum, count))| (key, sum / count as f64))
            .collect()
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
    
    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,30,60000").unwrap();
        
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.get_record_count(), 3);
        
        let filtered = processor.filter_by_column("age", |age| age == "30");
        assert_eq!(filtered.len(), 2);
        
        let aggregates = processor.aggregate_numeric_column("age", "salary");
        assert_eq!(aggregates.get("30").unwrap(), &55000.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<Record>,
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
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
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

    pub fn find_max_value_record(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn find_min_value_record(&self) -> Option<&Record> {
        self.records.iter().min_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &[Record] {
        &self.records
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_processor() {
        let processor = CsvProcessor::new();
        assert_eq!(processor.count_records(), 0);
        assert_eq!(processor.calculate_total_value(), 0.0);
        assert_eq!(processor.calculate_average_value(), 0.0);
    }

    #[test]
    fn test_record_operations() {
        let mut processor = CsvProcessor::new();
        
        let record1 = Record {
            id: 1,
            name: "Item A".to_string(),
            category: "Electronics".to_string(),
            value: 100.0,
            active: true,
        };
        
        let record2 = Record {
            id: 2,
            name: "Item B".to_string(),
            category: "Books".to_string(),
            value: 50.0,
            active: false,
        };
        
        processor.add_record(record1.clone());
        processor.add_record(record2.clone());
        
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.calculate_total_value(), 150.0);
        assert_eq!(processor.calculate_average_value(), 75.0);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 1);
        assert_eq!(electronics[0].id, 1);
        
        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 1);
        assert_eq!(active_records[0].id, 1);
        
        let max_record = processor.find_max_value_record();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 1);
        
        let min_record = processor.find_min_value_record();
        assert!(min_record.is_some());
        assert_eq!(min_record.unwrap().id, 2);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("Electronics").unwrap().len(), 1);
        assert_eq!(groups.get("Books").unwrap().len(), 1);
    }
}