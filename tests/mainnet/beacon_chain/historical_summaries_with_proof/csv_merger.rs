use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_header: bool,
) -> Result<(), Box<dyn Error>> {
    let mut output_writer = BufWriter::new(File::create(output_path)?);
    let mut first_file = true;

    for input_path in input_paths {
        let mut rdr = csv::Reader::from_path(input_path)?;
        let headers = rdr.headers()?.clone();

        if first_file {
            if write_header {
                writeln!(output_writer, "{}", headers.as_str())?;
            }
            first_file = false;
        }

        for result in rdr.records() {
            let record = result?;
            writeln!(output_writer, "{}", record.as_str())?;
        }
    }

    output_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_csv_files() -> Result<(), Box<dyn Error>> {
        let file1_content = "id,name\n1,Alice\n2,Bob\n";
        let file2_content = "id,name\n3,Charlie\n4,Diana\n";

        let mut temp_file1 = NamedTempFile::new()?;
        let mut temp_file2 = NamedTempFile::new()?;
        temp_file1.write_all(file1_content.as_bytes())?;
        temp_file2.write_all(file2_content.as_bytes())?;

        let output_file = NamedTempFile::new()?;
        let input_paths = vec![temp_file1.path(), temp_file2.path()];

        merge_csv_files(&input_paths, output_file.path(), true)?;

        let mut output_content = String::new();
        File::open(output_file.path())?.read_to_string(&mut output_content)?;

        let expected = "id,name\n1,Alice\n2,Bob\n3,Charlie\n4,Diana\n";
        assert_eq!(output_content, expected);

        Ok(())
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use csv::{ReaderBuilder, WriterBuilder};

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    if input_paths.is_empty() {
        return Err("No input files provided".into());
    }

    let mut headers_written = false;
    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new().from_writer(BufWriter::new(output_file));

    for input_path in input_paths {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        if !headers_written {
            let headers = csv_reader.headers()?.clone();
            writer.write_record(&headers)?;
            headers_written = true;
        }

        for result in csv_reader.records() {
            let record = result?;
            writer.write_record(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_files = vec![
        "data/file1.csv",
        "data/file2.csv",
        "data/file3.csv",
    ];
    
    merge_csv_files(&input_files, "merged_output.csv")?;
    
    println!("CSV files merged successfully");
    Ok(())
}