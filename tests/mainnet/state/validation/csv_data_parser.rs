use csv::{ReaderBuilder, WriterBuilder};
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

#[derive(Debug)]
struct ValidationResult {
    valid_records: Vec<Record>,
    invalid_lines: Vec<String>,
    total_processed: usize,
}

fn parse_csv_file<P: AsRef<Path>>(file_path: P) -> Result<ValidationResult, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let mut valid_records = Vec::new();
    let mut invalid_lines = Vec::new();
    let mut total_processed = 0;

    for result in rdr.deserialize() {
        total_processed += 1;
        match result {
            Ok(record) => {
                let rec: Record = record;
                if validate_record(&rec) {
                    valid_records.push(rec);
                } else {
                    invalid_lines.push(format!("Line {}: Failed validation", total_processed));
                }
            }
            Err(e) => {
                invalid_lines.push(format!("Line {}: Parse error - {}", total_processed, e));
            }
        }
    }

    Ok(ValidationResult {
        valid_records,
        invalid_lines,
        total_processed,
    })
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0 && record.id > 0
}

fn write_validated_csv<P: AsRef<Path>>(
    records: &[Record],
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(file);

    for record in records {
        wtr.serialize(record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn generate_summary(result: &ValidationResult) -> String {
    format!(
        "Processed {} lines\nValid records: {}\nInvalid lines: {}",
        result.total_processed,
        result.valid_records.len(),
        result.invalid_lines.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let csv_data = "id,name,value,active\n1,Test,42.5,true\n2,Another,100.0,false";
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, csv_data.as_bytes()).unwrap();

        let result = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(result.total_processed, 2);
        assert_eq!(result.valid_records.len(), 2);
        assert_eq!(result.invalid_lines.len(), 0);
    }

    #[test]
    fn test_validation_logic() {
        let valid_record = Record {
            id: 1,
            name: "Valid".to_string(),
            value: 10.0,
            active: true,
        };
        assert!(validate_record(&valid_record));

        let invalid_record = Record {
            id: 0,
            name: "".to_string(),
            value: -5.0,
            active: false,
        };
        assert!(!validate_record(&invalid_record));
    }
}