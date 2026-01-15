use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvFilter {
    delimiter: char,
    has_header: bool,
}

impl CsvFilter {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvFilter {
            delimiter,
            has_header,
        }
    }

    pub fn filter_by_column<R: BufRead>(
        &self,
        reader: R,
        column_index: usize,
        predicate: impl Fn(&str) -> bool,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut lines = reader.lines();
        let mut results = Vec::new();

        if self.has_header {
            if let Some(header) = lines.next() {
                results.push(header?.split(self.delimiter).map(String::from).collect());
            }
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(self.delimiter).map(String::from).collect();

            if column_index < fields.len() && predicate(&fields[column_index]) {
                results.push(fields);
            }
        }

        Ok(results)
    }

    pub fn process_file(
        &self,
        file_path: &str,
        column_index: usize,
        predicate: impl Fn(&str) -> bool,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        self.filter_by_column(reader, column_index, predicate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_column() {
        let csv_data = "name,age,city\nAlice,30,London\nBob,25,Paris\nCharlie,35,London";
        let filter = CsvFilter::new(',', true);
        let reader = std::io::Cursor::new(csv_data);

        let result = filter
            .filter_by_column(reader, 2, |city| city == "London")
            .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["name", "age", "city"]);
        assert_eq!(result[1], vec!["Alice", "30", "London"]);
        assert_eq!(result[2], vec!["Charlie", "35", "London"]);
    }

    #[test]
    fn test_filter_without_header() {
        let csv_data = "Alice,30,London\nBob,25,Paris\nCharlie,35,London";
        let filter = CsvFilter::new(',', false);
        let reader = std::io::Cursor::new(csv_data);

        let result = filter
            .filter_by_column(reader, 1, |age| age.parse::<u32>().unwrap() > 30)
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["Charlie", "35", "London"]);
    }
}