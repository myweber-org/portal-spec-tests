use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    skip_header: bool,
) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(File::create(output_path)?);
    let mut first_file = true;

    for (idx, path) in input_paths.iter().enumerate() {
        let mut rdr = csv::Reader::from_path(path)?;
        let headers = rdr.headers()?.clone();

        if first_file || !skip_header {
            writer.write_all(headers.as_bytes())?;
            writer.write_all(b"\n")?;
            first_file = false;
        }

        for result in rdr.records() {
            let record = result?;
            writer.write_all(record.as_slice().as_bytes())?;
            writer.write_all(b"\n")?;
        }

        if idx < input_paths.len() - 1 {
            writer.flush()?;
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
    fn test_merge_with_headers() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        std::fs::write(&file1, "col1,col2\nA,1\nB,2").unwrap();
        std::fs::write(&file2, "col1,col2\nC,3\nD,4").unwrap();

        merge_csv_files(&[file1.path(), file2.path()], output.path(), false).unwrap();

        let mut content = String::new();
        File::open(output.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert_eq!(content, "col1,col2\nA,1\nB,2\ncol1,col2\nC,3\nD,4\n");
    }

    #[test]
    fn test_merge_skip_headers() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        std::fs::write(&file1, "col1,col2\nA,1\nB,2").unwrap();
        std::fs::write(&file2, "col1,col2\nC,3\nD,4").unwrap();

        merge_csv_files(&[file1.path(), file2.path()], output.path(), true).unwrap();

        let mut content = String::new();
        File::open(output.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert_eq!(content, "col1,col2\nA,1\nB,2\nC,3\nD,4\n");
    }
}