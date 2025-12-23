use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    include_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let mut output_writer = BufWriter::new(File::create(output_path)?);
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let mut rdr = csv::Reader::from_path(input_path)?;
        let headers = rdr.headers()?.clone();

        if include_headers && !headers_written {
            output_writer.write_all(headers.as_bytes())?;
            output_writer.write_all(b"\n")?;
            headers_written = true;
        }

        for result in rdr.records() {
            let record = result?;
            output_writer.write_all(record.as_slice().as_bytes())?;
            output_writer.write_all(b"\n")?;
        }

        if index < input_paths.len() - 1 {
            output_writer.write_all(b"\n")?;
        }
    }

    output_writer.flush()?;
    Ok(())
}