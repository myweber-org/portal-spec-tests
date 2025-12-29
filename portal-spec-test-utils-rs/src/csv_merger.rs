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