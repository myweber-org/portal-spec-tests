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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
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

    pub fn transform_numeric_fields(&self, records: &mut Vec<Vec<String>>, column_index: usize) {
        for record in records.iter_mut() {
            if column_index < record.len() {
                if let Ok(num) = record[column_index].parse::<f64>() {
                    let transformed = (num * 100.0).round() / 100.0;
                    record[column_index] = format!("{:.2}", transformed);
                }
            }
        }
    }

    pub fn validate_record_lengths(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Ok(());
        }

        let expected_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", i + 1, record.len(), expected_len));
            }
        }
        Ok(())
    }
}

pub fn filter_records_by_column(
    records: Vec<Vec<String>>,
    column_index: usize,
    predicate: impl Fn(&str) -> bool,
) -> Vec<Vec<String>> {
    records
        .into_iter()
        .filter(|record| {
            column_index < record.len() && predicate(&record[column_index])
        })
        .collect()
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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

    pub fn validate_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if let Some(expected) = column_count {
                if columns.len() != expected {
                    return Err(format!("Line {} has {} columns, expected {}", 
                        index + 1, columns.len(), expected).into());
                }
            } else {
                column_count = Some(columns.len());
            }
            
            line_count += 1;
        }

        Ok(line_count)
    }

    pub fn transform_csv<P: AsRef<Path>>(
        &self, 
        input_path: P, 
        output_path: P,
        transform_fn: fn(&str) -> String
    ) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut lines = reader.lines();
        
        if self.has_headers {
            if let Some(header) = lines.next() {
                writeln!(output_file, "{}", header?)?;
            }
        }

        for line in lines {
            let original = line?;
            let transformed = transform_fn(&original);
            writeln!(output_file, "{}", transformed)?;
        }

        Ok(())
    }

    pub fn filter_csv<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        predicate: fn(&[&str]) -> bool
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;
        let mut kept_count = 0;

        let mut lines = reader.lines();
        
        if self.has_headers {
            if let Some(header) = lines.next() {
                writeln!(output_file, "{}", header?)?;
            }
        }

        for line in lines {
            let line_str = line?;
            let columns: Vec<&str> = line_str.split(self.delimiter).collect();
            
            if predicate(&columns) {
                writeln!(output_file, "{}", line_str)?;
                kept_count += 1;
            }
        }

        Ok(kept_count)
    }
}

pub fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

pub fn numeric_filter(columns: &[&str]) -> bool {
    if columns.is_empty() {
        return false;
    }
    
    columns[0].parse::<f64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_validation() {
        let content = "id,name,value\n1,test,100\n2,demo,200";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_csv_transformation() {
        let input_content = "id,name\n1,test\n2,demo";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(input_content.as_bytes()).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(',', true);
        processor.transform_csv(
            input_file.path(), 
            output_file.path(), 
            uppercase_transform
        ).unwrap();
        
        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert_eq!(output_content, "id,name\n1,TEST\n2,DEMO\n");
    }

    #[test]
    fn test_csv_filtering() {
        let input_content = "value,description\n100,valid\ninvalid,test\n200,another";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(input_content.as_bytes()).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let kept = processor.filter_csv(
            input_file.path(), 
            output_file.path(), 
            numeric_filter
        ).unwrap();
        
        assert_eq!(kept, 2);
        
        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert_eq!(output_content, "value,description\n100,valid\n200,another\n");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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

    pub fn read_and_validate(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(format!("Empty line found at line {}", line_number).into());
            }

            if self.has_headers && line_number == 1 {
                continue;
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid data records found".into());
        }

        Ok(records)
    }

    pub fn transform_numeric_fields(
        &self,
        records: Vec<Vec<String>>,
        column_index: usize,
        multiplier: f64,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut transformed = Vec::new();

        for record in records {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }

            let mut new_record = record.clone();
            if let Ok(value) = new_record[column_index].parse::<f64>() {
                let transformed_value = value * multiplier;
                new_record[column_index] = format!("{:.2}", transformed_value);
            }

            transformed.push(new_record);
        }

        Ok(transformed)
    }

    pub fn write_to_file(&self, records: Vec<Vec<String>>, output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(output_path)?;

        for record in records {
            let line = record.join(&self.delimiter.to_string());
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    pub fn calculate_column_average(&self, records: &[Vec<String>], column_index: usize) -> Result<f64, Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to process".into());
        }

        let mut sum = 0.0;
        let mut count = 0;

        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Err("No valid numeric values found in specified column".into());
        }

        Ok(sum / count as f64)
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
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.read_and_validate(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000"]);
        
        let transformed = processor.transform_numeric_fields(records, 2, 1.1).unwrap();
        assert_eq!(transformed[0][2], "55000.00");
        
        let average = processor.calculate_column_average(&transformed, 1).unwrap();
        assert!((average - 30.0).abs() < 0.01);
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
            
            if parts.len() == 4 {
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
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
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
        processor.load_from_file(file_path).unwrap();
        
        assert_eq!(processor.total_records(), 3);
        assert_eq!(processor.filter_by_category("Electronics").len(), 2);
        assert_eq!(processor.get_unique_categories().len(), 2);
        
        let avg = processor.calculate_average_value();
        assert!(avg > 11.0 && avg < 11.5);
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
            let line = line?;
            let record: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records.iter()
            .filter(|record| record.get(column_index).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let sum: f64 = self.records.iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .sum();

        if self.records.is_empty() {
            None
        } else {
            Some(sum)
        }
    }

    pub fn group_by_column(&self, group_column: &str, aggregate_column: &str) -> HashMap<String, f64> {
        let group_idx = match self.headers.iter().position(|h| h == group_column) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };

        let agg_idx = match self.headers.iter().position(|h| h == aggregate_column) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };

        let mut groups: HashMap<String, Vec<f64>> = HashMap::new();

        for record in &self.records {
            if let (Some(group_val), Some(agg_val)) = (record.get(group_idx), record.get(agg_idx)) {
                if let Ok(num) = agg_val.parse::<f64>() {
                    groups.entry(group_val.clone())
                        .or_insert_with(Vec::new)
                        .push(num);
                }
            }
        }

        groups.into_iter()
            .map(|(key, values)| (key, values.iter().sum()))
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_names(&self) -> &[String] {
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
        let file = create_test_csv();
        let processor = CsvProcessor::new(file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.record_count(), 4);
        assert_eq!(processor.column_names(), &["name", "department", "salary"]);
    }

    #[test]
    fn test_filter_by_column() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(file.path().to_str().unwrap()).unwrap();
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
    }

    #[test]
    fn test_aggregate_numeric() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(file.path().to_str().unwrap()).unwrap();
        let total_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert_eq!(total_salary, 290000.0);
    }

    #[test]
    fn test_group_by() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(file.path().to_str().unwrap()).unwrap();
        let dept_salaries = processor.group_by_column("department", "salary");
        assert_eq!(dept_salaries.get("Engineering").unwrap(), &155000.0);
        assert_eq!(dept_salaries.get("Marketing").unwrap(), &135000.0);
    }
}