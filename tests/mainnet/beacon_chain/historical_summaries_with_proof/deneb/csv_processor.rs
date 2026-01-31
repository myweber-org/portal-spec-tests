use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    file_path: String,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn aggregate_column(&self, column_index: usize) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut aggregation = HashMap::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if column_index >= parts.len() {
                return Err(format!("Column index {} out of bounds on line {}", column_index, line_num + 1).into());
            }

            let key = parts[column_index].to_string();
            let value: f64 = parts.get(column_index + 1)
                .ok_or_else(|| format!("Missing value column on line {}", line_num + 1))?
                .parse()
                .map_err(|_| format!("Invalid number on line {}", line_num + 1))?;

            *aggregation.entry(key).or_insert(0.0) += value;
        }

        Ok(aggregation)
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Result<Vec<String>, Box<dyn Error>>
    where
        F: Fn(&[&str]) -> bool,
    {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            if predicate(&parts) {
                filtered.push(line);
            }
        }

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aggregation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,5.2").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.aggregate_column(0).unwrap();

        assert_eq!(result.get("A"), Some(&15.7));
        assert_eq!(result.get("B"), Some(&20.3));
    }

    #[test]
    fn test_filter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();
        writeln!(temp_file, "Charlie,35,London").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap());
        let filtered = processor.filter_rows(|parts| parts[2] == "London").unwrap();

        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].contains("Alice"));
        assert!(filtered[1].contains("Charlie"));
    }
}