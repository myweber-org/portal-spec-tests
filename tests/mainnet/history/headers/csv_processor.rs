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

    pub fn filter_rows<P, F>(
        &self,
        file_path: P,
        predicate: F,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut result = Vec::new();

        if self.has_header {
            lines.next();
        }

        for line in lines {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&fields) {
                result.push(fields);
            }
        }

        Ok(result)
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|row| row.get(column_index).cloned())
            .collect()
    }
}

pub fn calculate_average(values: &[String]) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0;

    for value in values {
        if let Ok(num) = value.parse::<f64>() {
            sum += num;
            count += 1;
        }
    }

    if count > 0 {
        Some(sum / count as f64)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_and_average() {
        let csv_data = "name,age,salary\nAlice,30,50000\nBob,25,45000\nCharlie,35,60000";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let processor = CsvProcessor::new(',', true);
        let filtered = processor
            .filter_rows(temp_file.path(), |row| {
                row.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age >= 30)
            })
            .unwrap();

        let salaries = processor.extract_column(&filtered, 2);
        let avg_salary = calculate_average(&salaries).unwrap();

        assert_eq!(filtered.len(), 2);
        assert!((avg_salary - 55000.0).abs() < 0.001);
    }
}