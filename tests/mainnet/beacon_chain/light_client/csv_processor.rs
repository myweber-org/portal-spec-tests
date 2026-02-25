use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn filter_rows<P: AsRef<Path>>(
        &self,
        file_path: P,
        predicate: impl Fn(&[String]) -> bool,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        let mut filtered = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&fields) {
                filtered.push(fields);
            }
        }

        Ok(filtered)
    }

    pub fn count_matching_rows<P: AsRef<Path>>(
        &self,
        file_path: P,
        column_index: usize,
        expected_value: &str,
    ) -> Result<usize, Box<dyn Error>> {
        let filtered = self.filter_rows(file_path, |fields| {
            fields.get(column_index).map_or(false, |val| val == expected_value)
        })?;
        Ok(filtered.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();
        writeln!(temp_file, "Charlie,30,Tokyo").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor
            .filter_rows(temp_file.path(), |fields| fields[1] == "30")
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][0], "Charlie");
    }

    #[test]
    fn test_count_matching_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,status,value").unwrap();
        writeln!(temp_file, "1,active,100").unwrap();
        writeln!(temp_file, "2,inactive,200").unwrap();
        writeln!(temp_file, "3,active,150").unwrap();

        let processor = CsvProcessor::new(',', true);
        let count = processor
            .count_matching_rows(temp_file.path(), 1, "active")
            .unwrap();

        assert_eq!(count, 2);
    }
}