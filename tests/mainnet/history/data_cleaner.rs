use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;
use std::io;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    for result in reader.records() {
        let record = result?;
        let filtered_record: Vec<&str> = record
            .iter()
            .filter(|field| !field.trim().is_empty())
            .collect();

        if filtered_record.len() == headers.len() {
            writer.write_record(&filtered_record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn clean_csv_from_stdin() -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_reader(io::stdin());
    let mut writer = Writer::from_writer(io::stdout());

    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    for result in reader.records() {
        let record = result?;
        let filtered_record: Vec<&str> = record
            .iter()
            .filter(|field| !field.trim().is_empty())
            .collect();

        if filtered_record.len() == headers.len() {
            writer.write_record(&filtered_record)?;
        }
    }

    writer.flush()?;
    Ok(())
}