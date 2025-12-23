use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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
        let headers: Vec<String> = headers_line.split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
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
        
        match column_index {
            Some(idx) => self.records.iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records.iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .collect();
        
        if numeric_values.is_empty() {
            return None;
        }
        
        match operation.to_lowercase().as_str() {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }
    
    pub fn write_to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        
        writeln!(file, "{}", self.headers.join(","))?;
        
        for record in &self.records {
            writeln!(file, "{}", record.join(","))?;
        }
        
        Ok(())
    }
    
    pub fn group_by_column(&self, column_name: &str) -> Option<HashMap<String, Vec<Vec<String>>>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)?;
        
        let mut groups: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        
        for record in &self.records {
            let key = record[column_index].clone();
            groups.entry(key)
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        Some(groups)
    }
}

pub fn process_csv_data(input_path: &str, output_path: &str, filter_column: &str, filter_value: &str) -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::from_file(input_path)?;
    
    let filtered_data = processor.filter_by_column(filter_column, |value| value == filter_value);
    
    let mut output_processor = CsvProcessor {
        headers: processor.headers.clone(),
        records: filtered_data,
    };
    
    output_processor.write_to_file(output_path)?;
    
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() == 4 {
            let record = Record {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                category: parts[3].to_string(),
            };
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[&Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
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
        writeln!(temp_file, "2,ItemB,15.2,Category2").unwrap();

        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 15.2);
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

        let filtered = filter_by_category(&records, "Cat1");
        let avg = calculate_average(&filtered);
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 25.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 15.0, category: "Cat1".to_string() },
        ];

        let max_record = find_max_value(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 25.0);
    }
}