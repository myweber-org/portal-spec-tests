use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

    for (i, input_path) in input_paths.iter().enumerate() {
        let path = Path::new(input_path);
        let mut rdr = csv::Reader::from_path(path)?;
        let headers = rdr.headers()?.clone();

        if i == 0 {
            writer.write_all(headers.as_bytes())?;
            writer.write_all(b"\n")?;
            headers_written = true;
        }

        for result in rdr.records() {
            let record = result?;
            if !headers_written && record != headers {
                return Err("CSV files have different headers".into());
            }
            writer.write_all(record.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_merge_csv_files() {
        let data1 = "name,age\nAlice,30\nBob,25";
        let data2 = "name,age\nCharlie,35\nDiana,28";
        fs::write("test1.csv", data1).unwrap();
        fs::write("test2.csv", data2).unwrap();

        let inputs = vec!["test1.csv", "test2.csv"];
        merge_csv_files(&inputs, "merged.csv").unwrap();

        let merged = fs::read_to_string("merged.csv").unwrap();
        let expected = "name,age\nAlice,30\nBob,25\nCharlie,35\nDiana,28\n";
        assert_eq!(merged, expected);

        fs::remove_file("test1.csv").unwrap();
        fs::remove_file("test2.csv").unwrap();
        fs::remove_file("merged.csv").unwrap();
    }
}