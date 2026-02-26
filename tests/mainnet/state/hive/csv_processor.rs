use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
    errors: Vec<CsvError>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            match self.parse_line(&line, line_num + 1) {
                Ok(record) => self.records.push(record),
                Err(err) => self.errors.push(err),
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 columns, found {}", line_num, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: Invalid ID format '{}'", line_num, parts[0])
            ))?;

        let name = parts[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_num)
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: Invalid value format '{}'", line_num, parts[2])
            ))?;

        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Line {}: Invalid boolean format '{}'", line_num, parts[3])
            )),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn get_errors(&self) -> &[CsvError] {
        &self.errors
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&CsvRecord> {
        self.records.iter()
            .find(|r| r.name.to_lowercase() == name.to_lowercase())
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input_path: String, output_path: String, filter_column: usize, filter_value: String) -> Self {
        CsvProcessor {
            input_path,
            output_path,
            filter_column,
            filter_value,
        }
    }

    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if let Some(cell) = parts.get(self.filter_column) {
                if cell.trim() == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                }
            }
        }

        Ok(())
    }
}

pub fn transform_data(input: &str) -> String {
    input
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .collect::<Vec<String>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_transform_data() {
        let input = "hello, world, rust";
        let expected = "HELLO|WORLD|RUST";
        assert_eq!(transform_data(input), expected);
    }

    #[test]
    fn test_csv_processor() -> Result<(), Box<dyn Error>> {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let content = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active";
        fs::write(test_input, content)?;

        let processor = CsvProcessor::new(
            test_input.to_string(),
            test_output.to_string(),
            2,
            "active".to_string(),
        );
        
        processor.process()?;
        
        let output_content = fs::read_to_string(test_output)?;
        let expected = "id,name,status\n1,alice,active\n3,charlie,active\n";
        assert_eq!(output_content, expected);

        fs::remove_file(test_input)?;
        fs::remove_file(test_output)?;
        
        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Option<Vec<usize>>,
    has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: None,
            has_headers: true,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        CsvProcessor { config }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.config.has_headers {
                continue;
            }

            let record: Vec<String> = line
                .split(self.config.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            let filtered_record = if let Some(ref selected) = self.config.selected_columns {
                selected
                    .iter()
                    .filter_map(|&idx| record.get(idx).cloned())
                    .collect()
            } else {
                record
            };

            if !filtered_record.is_empty() {
                records.push(filtered_record);
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Carol,35,Tokyo").unwrap();

        let config = CsvConfig {
            selected_columns: Some(vec![0, 2]),
            ..Default::default()
        };

        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Alice", "New York"]);
        assert_eq!(result[1], vec!["Bob", "London"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
            vec!["Carol".to_string(), "35".to_string()],
        ];

        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        
        let filtered = processor.filter_records(&records, |record| {
            record.get(1)
                .and_then(|age| age.parse::<i32>().ok())
                .map_or(false, |age| age > 30)
        });

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Carol", "35"]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn filter_by_column_value(&self, column_name: &str, target_value: &str) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        
        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);
        
        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;
        
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        for result in csv_reader.records() {
            let record = result?;
            if record.get(column_index).map(|v| v == target_value).unwrap_or(false) {
                csv_writer.write_record(&record)?;
            }
        }
        
        csv_writer.flush()?;
        Ok(())
    }
    
    pub fn transform_column(&self, column_name: &str, transform_fn: fn(&str) -> String) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        
        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);
        
        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;
        
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        for result in csv_reader.records() {
            let mut record = result?.into_iter().map(String::from).collect::<Vec<String>>();
            if let Some(value) = record.get_mut(column_index) {
                *value = transform_fn(value);
            }
            csv_writer.write_record(&record)?;
        }
        
        csv_writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    fn uppercase_transform(s: &str) -> String {
        s.to_uppercase()
    }
    
    #[test]
    fn test_filter_by_column_value() {
        let input_content = "name,age,city\nAlice,30,London\nBob,25,Paris\nCharlie,35,London";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        processor.filter_by_column_value("city", "London").unwrap();
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = "name,age,city\nAlice,30,London\nCharlie,35,London\n";
        assert_eq!(output_content, expected);
    }
    
    #[test]
    fn test_transform_column() {
        let input_content = "name,age,city\nalice,30,london\nbob,25,paris";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        processor.transform_column("name", uppercase_transform).unwrap();
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = "name,age,city\nALICE,30,london\nBOB,25,paris\n";
        assert_eq!(output_content, expected);
    }
}