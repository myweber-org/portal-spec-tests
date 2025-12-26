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
}