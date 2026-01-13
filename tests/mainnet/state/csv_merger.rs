use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files(
    input_paths: &[String],
    output_path: &str,
    skip_header: bool,
) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut is_first_file = true;

    for (index, input_path) in input_paths.iter().enumerate() {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(format!("Input file not found: {}", input_path).into());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if skip_header && index > 0 {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            if is_first_file || !line.trim().is_empty() {
                writeln!(writer, "{}", line)?;
            }
        }

        is_first_file = false;
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
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(file1.path(), "id,name\n1,alice\n2,bob").unwrap();
        std::fs::write(file2.path(), "id,name\n3,charlie\n4,diana").unwrap();

        let inputs = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        merge_csv_files(&inputs, output_file.path().to_str().unwrap(), true).unwrap();

        let mut content = String::new();
        std::fs::File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        let expected = "id,name\n1,alice\n2,bob\n3,charlie\n4,diana\n";
        assert_eq!(content, expected);
    }
}