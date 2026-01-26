use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(File::create(output_path)?);
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let mut rdr = csv::Reader::from_path(input_path)?;
        let headers = rdr.headers()?.clone();

        if index == 0 && write_headers {
            writer.write_all(headers.as_bytes())?;
            writer.write_all(b"\n")?;
            headers_written = true;
        }

        for result in rdr.records() {
            let record = result?;
            if !headers_written && write_headers {
                writer.write_all(headers.as_bytes())?;
                writer.write_all(b"\n")?;
                headers_written = true;
            }
            writer.write_all(record.as_slice())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_header: bool,
) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut first_file = true;

    for path in input_paths {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let headers = rdr.headers()?.clone();

        if first_file {
            if write_header {
                writer.write_all(headers.as_bytes())?;
                writer.write_all(b"\n")?;
            }
            first_file = false;
        }

        for result in rdr.records() {
            let record = result?;
            writer.write_all(record.as_slice().as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}