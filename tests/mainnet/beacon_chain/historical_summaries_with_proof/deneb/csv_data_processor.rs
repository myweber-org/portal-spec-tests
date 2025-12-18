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
}