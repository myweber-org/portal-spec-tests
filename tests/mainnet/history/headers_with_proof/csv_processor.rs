use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(Ok(line)) => line.split(',').map(|s| s.trim().to_string()).collect(),
            Some(Err(e)) => return Err(Box::new(e)),
            None => return Err("Empty CSV file".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column<F>(&self, column_index: usize, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&str) -> bool,
    {
        self.records
            .iter()
            .filter(|record| {
                record
                    .get(column_index)
                    .map(|value| predicate(value))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::from_file(temp_file.path()).unwrap();
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.get_headers(), &["name", "age", "city"]);

        let filtered = processor.filter_by_column(1, |age| age.parse::<u32>().unwrap_or(0) > 30);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Paris"]);
    }
}