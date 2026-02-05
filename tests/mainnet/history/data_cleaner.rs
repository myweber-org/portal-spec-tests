use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_lowercase())
            .collect();
        writer.write_record(&cleaned_record)?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "Name, Age, City").unwrap();
        writeln!(input_file, " John , 25, New York").unwrap();
        writeln!(input_file, "Alice, 30 , London ").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        clean_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        )
        .unwrap();

        let mut reader = Reader::from_path(output_file.path()).unwrap();
        let records: Vec<_> = reader.records().collect::<Result<_, _>>().unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["john", "25", "new york"]);
        assert_eq!(records[1], vec!["alice", "30", "london"]);
    }
}