use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);
    
    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;
    
    for result in rdr.records() {
        let record = result?;
        if record.iter().all(|field| !field.trim().is_empty()) {
            wtr.write_record(&record)?;
        }
    }
    
    wtr.flush()?;
    Ok(())
}

pub fn clean_csv_from_stdin() -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(io::stdin());
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(io::stdout());
    
    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;
    
    for result in rdr.records() {
        let record = result?;
        if record.iter().all(|field| !field.trim().is_empty()) {
            wtr.write_record(&record)?;
        }
    }
    
    wtr.flush()?;
    Ok(())
}