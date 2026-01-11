use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let mut rdr = csv::Reader::from_path(input_path)?;
        let headers = rdr.headers()?.clone();

        if index == 0 {
            writer.write_all(headers.as_bytes())?;
            writer.write_all(b"\n")?;
            headers_written = true;
        } else if headers != rdr.headers()? {
            eprintln!("Warning: Headers in {} differ from first file. Skipping header.", input_path);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_merge_csv_files() {
        let test_data1 = "id,name,value\n1,test1,100\n2,test2,200";
        let test_data2 = "id,name,value\n3,test3,300\n4,test4,400";

        fs::write("test1.csv", test_data1).unwrap();
        fs::write("test2.csv", test_data2).unwrap();

        let inputs = vec!["test1.csv", "test2.csv"];
        merge_csv_files(&inputs, "merged.csv").unwrap();

        let merged_content = fs::read_to_string("merged.csv").unwrap();
        let expected = "id,name,value\n1,test1,100\n2,test2,200\n3,test3,300\n4,test4,400\n";
        assert_eq!(merged_content, expected);

        fs::remove_file("test1.csv").unwrap();
        fs::remove_file("test2.csv").unwrap();
        fs::remove_file("merged.csv").unwrap();
    }
}