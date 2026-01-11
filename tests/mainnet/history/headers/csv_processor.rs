use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            headers: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_line) = lines.next() {
            let header_line = header_line?;
            self.headers = header_line.split(',').map(|s| s.trim().to_string()).collect();
        }

        for line in lines {
            let line = line?;
            let record: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if record.len() == self.headers.len() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(index) => index,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }

    pub fn get_column_summary(&self, column_name: &str) -> Option<(usize, String)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let values: Vec<&String> = self.records
            .iter()
            .filter_map(|record| record.get(column_index))
            .collect();

        if values.is_empty() {
            return None;
        }

        let unique_count = values.iter().collect::<std::collections::HashSet<_>>().len();
        let sample_value = values[0].clone();

        Some((unique_count, sample_value))
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn header_count(&self) -> usize {
        self.headers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,Alice,30").unwrap();
        writeln!(temp_file, "2,Bob,25").unwrap();
        writeln!(temp_file, "3,Alice,35").unwrap();
        temp_file
    }

    #[test]
    fn test_load_and_filter() {
        let temp_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.header_count(), 3);
        
        let alice_records = processor.filter_by_column("name", "Alice");
        assert_eq!(alice_records.len(), 2);
        
        let summary = processor.get_column_summary("name").unwrap();
        assert_eq!(summary.0, 2);
    }
}