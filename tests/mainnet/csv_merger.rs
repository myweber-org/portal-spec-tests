use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use csv::{ReaderBuilder, WriterBuilder};

pub struct CsvMerger {
    input_paths: Vec<String>,
    output_path: String,
    selected_columns: Option<HashSet<String>>,
}

impl CsvMerger {
    pub fn new(input_paths: Vec<String>, output_path: String) -> Self {
        Self {
            input_paths,
            output_path,
            selected_columns: None,
        }
    }

    pub fn select_columns(&mut self, columns: Vec<&str>) -> &mut Self {
        self.selected_columns = Some(columns.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn merge(&self) -> Result<(), Box<dyn Error>> {
        if self.input_paths.is_empty() {
            return Err("No input files provided".into());
        }

        let output_file = File::create(&self.output_path)?;
        let mut writer = WriterBuilder::new().from_writer(BufWriter::new(output_file));

        let mut headers_written = false;
        let mut common_headers = Vec::new();

        for input_path in &self.input_paths {
            if !Path::new(input_path).exists() {
                return Err(format!("Input file not found: {}", input_path).into());
            }

            let file = File::open(input_path)?;
            let mut reader = ReaderBuilder::new().from_reader(BufReader::new(file));

            let headers = reader.headers()?.clone();

            let filtered_headers: Vec<String> = match &self.selected_columns {
                Some(selected) => headers
                    .iter()
                    .filter(|h| selected.contains(*h))
                    .map(|s| s.to_string())
                    .collect(),
                None => headers.iter().map(|s| s.to_string()).collect(),
            };

            if !headers_written {
                common_headers = filtered_headers.clone();
                writer.write_record(&common_headers)?;
                headers_written = true;
            }

            if filtered_headers != common_headers {
                return Err("Column mismatch between input files".into());
            }

            for result in reader.records() {
                let record = result?;
                let filtered_record: Vec<String> = match &self.selected_columns {
                    Some(selected) => headers
                        .iter()
                        .enumerate()
                        .filter(|(_, h)| selected.contains(*h))
                        .map(|(i, _)| record.get(i).unwrap_or("").to_string())
                        .collect(),
                    None => record.iter().map(|s| s.to_string()).collect(),
                };
                writer.write_record(&filtered_record)?;
            }
        }

        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, "name,age,city\nAlice,30,London\nBob,25,Paris").unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "name,age,city\nCharlie,35,Berlin\nDiana,28,Rome").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let merger = CsvMerger::new(
            vec![
                file1.path().to_str().unwrap().to_string(),
                file2.path().to_str().unwrap().to_string(),
            ],
            output_file.path().to_str().unwrap().to_string(),
        );

        assert!(merger.merge().is_ok());

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("Alice,30,London"));
        assert!(content.contains("Charlie,35,Berlin"));
    }

    #[test]
    fn test_column_selection() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,email,phone\n1,John,john@test.com,123456").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let mut merger = CsvMerger::new(
            vec![file.path().to_str().unwrap().to_string()],
            output_file.path().to_str().unwrap().to_string(),
        );

        merger.select_columns(vec!["name", "email"]);
        assert!(merger.merge().is_ok());

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("name,email"));
        assert!(content.contains("John,john@test.com"));
        assert!(!content.contains("id"));
        assert!(!content.contains("phone"));
    }
}