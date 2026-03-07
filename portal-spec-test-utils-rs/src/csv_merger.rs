use std::error::Error;
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
        let mut rdr = csv::Reader::from_path(path)?;
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
use clap::Parser;
use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Merge multiple CSV files with identical headers")]
struct Args {
    #[arg(short, long, required = true, help = "Input CSV files to merge")]
    input: Vec<PathBuf>,
    
    #[arg(short, long, help = "Output CSV file path")]
    output: Option<PathBuf>,
    
    #[arg(short, long, default_value = "true", help = "Include headers in output")]
    headers: bool,
}

fn merge_csv_files(args: &Args) -> Result<(), Box<dyn Error>> {
    let output_path = args.output.as_ref().map_or_else(|| PathBuf::from("merged.csv"), |p| p.clone());
    
    let mut writer = WriterBuilder::new()
        .has_headers(args.headers)
        .from_path(&output_path)?;
    
    let mut first_file = true;
    
    for input_path in &args.input {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(input_path)?;
        
        let headers = reader.headers()?.clone();
        
        if first_file && args.headers {
            writer.write_record(&headers)?;
            first_file = false;
        }
        
        for result in reader.records() {
            let record = result?;
            writer.write_record(&record)?;
        }
    }
    
    writer.flush()?;
    println!("Successfully merged {} files into {}", args.input.len(), output_path.display());
    
    Ok(())
}

fn main() {
    let args = Args::parse();
    
    if args.input.len() < 2 {
        eprintln!("Error: At least 2 input files required");
        std::process::exit(1);
    }
    
    if let Err(e) = merge_csv_files(&args) {
        eprintln!("Error merging CSV files: {}", e);
        std::process::exit(1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use csv::{ReaderBuilder, WriterBuilder};

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    if input_paths.is_empty() {
        return Err("No input files provided".into());
    }

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().from_writer(&mut writer);

    let mut headers_written = false;
    let mut expected_headers: Option<Vec<String>> = None;

    for input_path in input_paths {
        let file = File::open(input_path)?;
        let mut csv_reader = ReaderBuilder::new().from_reader(file);

        let headers = csv_reader.headers()?.clone();

        if let Some(ref expected) = expected_headers {
            if headers != *expected {
                return Err(format!(
                    "Header mismatch: expected {:?}, found {:?} in file {:?}",
                    expected,
                    headers,
                    input_path.as_ref()
                )
                .into());
            }
        } else {
            expected_headers = Some(headers.iter().map(|s| s.to_string()).collect());
        }

        if !headers_written {
            csv_writer.write_record(&headers)?;
            headers_written = true;
        }

        for result in csv_reader.records() {
            let record = result?;
            csv_writer.write_record(&record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_csv_files() {
        let csv1_content = "id,name,value\n1,test,100\n";
        let csv2_content = "id,name,value\n2,example,200\n";

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(&file1, csv1_content).unwrap();
        std::fs::write(&file2, csv2_content).unwrap();

        let inputs = [file1.path(), file2.path()];
        merge_csv_files(&inputs, output_file.path()).unwrap();

        let mut result = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut result)
            .unwrap();

        let expected = "id,name,value\n1,test,100\n2,example,200\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_header_mismatch() {
        let csv1_content = "id,name,value\n1,test,100\n";
        let csv2_content = "id,description,value\n2,example,200\n";

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(&file1, csv1_content).unwrap();
        std::fs::write(&file2, csv2_content).unwrap();

        let inputs = [file1.path(), file2.path()];
        let result = merge_csv_files(&inputs, output_file.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Header mismatch"));
    }
}