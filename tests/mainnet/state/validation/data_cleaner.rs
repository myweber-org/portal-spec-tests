use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    for result in csv_reader.deserialize() {
        let record: Record = result?;
        
        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: record.value.max(0.0),
            category: if record.category.is_empty() {
                "uncategorized".to_string()
            } else {
                record.category
            },
        };

        csv_writer.serialize(&cleaned_record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("input_data.csv");
    let output_path = Path::new("cleaned_data.csv");

    match clean_csv_data(input_path, output_path) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }

    Ok(())
}use csv::{ReaderBuilder, WriterBuilder};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    let mut seen = HashSet::new();
    for result in csv_reader.records() {
        let record = result?;
        let row_string = record.iter().collect::<Vec<&str>>().join(",");
        
        if seen.insert(row_string.clone()) {
            csv_writer.write_record(&record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_remove_duplicates() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "Alice,30,New York").unwrap();
        writeln!(input_file, "Bob,25,London").unwrap();
        writeln!(input_file, "Alice,30,New York").unwrap();
        writeln!(input_file, "Charlie,35,Paris").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        remove_duplicates(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        ).unwrap();

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines.contains(&"Alice,30,New York"));
        assert!(lines.contains(&"Bob,25,London"));
        assert!(lines.contains(&"Charlie,35,Paris"));
    }
}