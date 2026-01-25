use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = Reader::from_reader(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = Writer::from_writer(output_file);
    
    for result in rdr.deserialize() {
        let record: Record = result?;
        
        match record.validate() {
            Ok(_) => {
                wtr.serialize(&record)?;
                println!("Processed record: {:?}", record);
            }
            Err(e) => {
                eprintln!("Validation failed for record {:?}: {}", record, e);
            }
        }
    }
    
    wtr.flush()?;
    Ok(())
}

fn main() {
    let input = "data/input.csv";
    let output = "data/output.csv";
    
    match process_csv(input, output) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    InvalidHeader(usize, usize),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;
        let mut expected_columns = None;

        for line in reader.lines() {
            let line_content = line?;
            line_number += 1;

            if line_content.trim().is_empty() {
                continue;
            }

            let columns: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if line_number == 1 && self.has_header {
                expected_columns = Some(columns.len());
                continue;
            }

            if let Some(expected) = expected_columns {
                if columns.len() != expected {
                    return Err(CsvError::InvalidHeader(expected, columns.len()));
                }
            }

            records.push(CsvRecord { columns });
        }

        if records.is_empty() {
            return Err(CsvError::ParseError("No valid records found".to_string()));
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[CsvRecord]) -> Result<(), CsvError> {
        for (idx, record) in records.iter().enumerate() {
            if record.columns.iter().any(|col| col.is_empty()) {
                return Err(CsvError::ParseError(format!(
                    "Record {} contains empty columns",
                    idx + 1
                )));
            }
        }
        Ok(())
    }
}

pub fn process_csv_file(
    file_path: &str,
    delimiter: char,
    has_header: bool,
) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let processor = CsvProcessor::new(delimiter, has_header);
    let records = processor.process_file(file_path)?;
    processor.validate_records(&records)?;
    Ok(records)
}