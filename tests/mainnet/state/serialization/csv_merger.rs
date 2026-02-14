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
            eprintln!("Warning: Headers in {} differ from first file. Skipping header.", input_path);
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

        let file1_content = "id,name,value\n1,alpha,100\n2,beta,200";
        let file2_content = "id,name,value\n3,gamma,300\n4,delta,400";

        let file1_path = format!("{}/file1.csv", test_dir);
        let file2_path = format!("{}/file2.csv", test_dir);
        let output_path = format!("{}/merged.csv", test_dir);

        fs::write(&file1_path, file1_content).unwrap();
        fs::write(&file2_path, file2_content).unwrap();

        let inputs = [file1_path.as_str(), file2_path.as_str()];
        let result = merge_csv_files(&inputs, &output_path);

        assert!(result.is_ok());
        let merged_content = fs::read_to_string(&output_path).unwrap();
        let expected = "id,name,value\n1,alpha,100\n2,beta,200\n3,gamma,300\n4,delta,400\n";
        assert_eq!(merged_content, expected);

        fs::remove_dir_all(test_dir).unwrap();
    }
}