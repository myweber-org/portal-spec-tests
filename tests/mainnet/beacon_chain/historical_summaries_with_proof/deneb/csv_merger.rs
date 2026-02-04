use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let path = Path::new(input_path);
        let mut rdr = csv::Reader::from_path(path)?;
        let headers = rdr.headers()?.clone();

        if index == 0 {
            writer.write_all(headers.as_bytes())?;
            writer.write_all(b"\n")?;
            headers_written = true;
        } else if headers != rdr.headers()? {
            eprintln!("Warning: Headers in {} differ from first file. Skipping.", input_path);
            continue;
        }

        for result in rdr.records() {
            let record = result?;
            writer.write_all(record.as_slice().as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_merge_csv_files() {
        let test_dir = "test_merge";
        fs::create_dir_all(test_dir).unwrap();

        let file1 = format!("{}/data1.csv", test_dir);
        let file2 = format!("{}/data2.csv", test_dir);
        let output = format!("{}/merged.csv", test_dir);

        fs::write(&file1, "id,name\n1,Alice\n2,Bob").unwrap();
        fs::write(&file2, "id,name\n3,Charlie\n4,Diana").unwrap();

        let inputs = [file1.as_str(), file2.as_str()];
        merge_csv_files(&inputs, &output).unwrap();

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Diana"));

        fs::remove_dir_all(test_dir).unwrap();
    }
}use std::error::Error;
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
                output_writer.write_all(headers.as_bytes())?;
                output_writer.write_all(b"\n")?;
            }
            first_file = false;
        } else if write_header {
            // Skip header for subsequent files
            continue;
        }

        for result in rdr.records() {
            let record = result?;
            output_writer.write_all(record.as_slice().as_bytes())?;
            output_writer.write_all(b"\n")?;
        }
    }

    output_writer.flush()?;
    Ok(())
}