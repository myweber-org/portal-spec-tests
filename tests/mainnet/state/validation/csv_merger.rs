use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    write_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let file = File::open(input_path)?;
        let mut rdr = csv::Reader::from_reader(file);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_csv_files() {
        let csv1 = "name,age\nAlice,30\nBob,25";
        let csv2 = "name,age\nCharlie,35\nDiana,28";

        let file1 = NamedTempFile::new().unwrap();
        std::fs::write(file1.path(), csv1).unwrap();
        let file2 = NamedTempFile::new().unwrap();
        std::fs::write(file2.path(), csv2).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let input_paths = [file1.path(), file2.path()];

        merge_csv_files(&input_paths, output_file.path(), true).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let expected = "name,age\nAlice,30\nBob,25\nCharlie,35\nDiana,28\n";
        assert_eq!(content, expected);
    }
}