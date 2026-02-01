
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_headers {
            let _headers = lines.next().transpose()?;
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() && !record.iter().all(|field| field.is_empty()) {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_columns: usize) -> bool {
        record.len() == expected_columns && 
        !record.iter().any(|field| field.is_empty())
    }

    pub fn transform_numeric_fields(records: &[Vec<String>]) -> Vec<Vec<String>> {
        records
            .iter()
            .map(|record| {
                record
                    .iter()
                    .map(|field| {
                        if let Ok(num) = field.parse::<f64>() {
                            format!("{:.2}", num)
                        } else {
                            field.clone()
                        }
                    })
                    .collect()
            })
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Alice,30.5,London").unwrap();
        writeln!(temp_file, "Bob,,Paris").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 3);
        assert!(processor.validate_record(&result[0], 3));
        assert!(!processor.validate_record(&result[2], 3));
        
        let transformed = CsvProcessor::transform_numeric_fields(&result);
        assert_eq!(transformed[1][1], "30.50");
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            if index == 0 {
                continue;
            }
            
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 4 {
                let id = parts[0].parse::<u32>()?;
                let category = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let active = parts[3].parse::<bool>().unwrap_or(false);
                
                self.records.push(Record {
                    id,
                    category,
                    value,
                    active,
                });
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

    pub fn calculate_average_by_category(&self) -> HashMap<String, f64> {
        let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();
        
        for record in &self.records {
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

    pub fn get_active_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,250.50,true").unwrap();
        writeln!(temp_file, "2,books,45.99,true").unwrap();
        writeln!(temp_file, "3,electronics,120.75,false").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.total_records(), 3);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let averages = processor.calculate_average_by_category();
        assert!(averages.contains_key("electronics"));
        
        let active_records = processor.get_active_records();
        assert_eq!(active_records.len(), 2);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value, 250.50);
    }
}