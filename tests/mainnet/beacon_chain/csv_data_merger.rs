
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone)]
struct CsvConfig {
    delimiter: char,
    has_headers: bool,
    output_delimiter: char,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            has_headers: true,
            output_delimiter: ',',
        }
    }
}

struct CsvMerger {
    config: CsvConfig,
    column_mapping: HashMap<String, usize>,
    merged_data: Vec<Vec<String>>,
}

impl CsvMerger {
    fn new(config: CsvConfig) -> Self {
        CsvMerger {
            config,
            column_mapping: HashMap::new(),
            merged_data: Vec::new(),
        }
    }

    fn load_file<P: AsRef<Path>>(&mut self, filepath: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.config.has_headers && self.column_mapping.is_empty() {
            if let Some(Ok(header_line)) = lines.next() {
                self.process_headers(&header_line)?;
            }
        }

        for line_result in lines {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }
            self.process_row(&line)?;
        }

        Ok(())
    }

    fn process_headers(&mut self, header_line: &str) -> Result<(), Box<dyn Error>> {
        let headers: Vec<String> = header_line
            .split(self.config.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        for (index, header) in headers.iter().enumerate() {
            self.column_mapping.insert(header.clone(), index);
        }

        self.merged_data.push(headers);
        Ok(())
    }

    fn process_row(&mut self, row_line: &str) -> Result<(), Box<dyn Error>> {
        let columns: Vec<String> = row_line
            .split(self.config.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if self.merged_data.is_empty() {
            self.merged_data.push(
                (0..columns.len())
                    .map(|i| format!("Column_{}", i + 1))
                    .collect(),
            );
        }

        self.merged_data.push(columns);
        Ok(())
    }

    fn save_to_file<P: AsRef<Path>>(&self, filepath: P) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(filepath)?;
        
        for row in &self.merged_data {
            let line = row.join(&self.config.output_delimiter.to_string());
            writeln!(file, "{}", line)?;
        }
        
        Ok(())
    }

    fn get_column_data(&self, column_name: &str) -> Option<Vec<String>> {
        let column_index = self.column_mapping.get(column_name)?;
        
        let mut result = Vec::new();
        for (row_index, row) in self.merged_data.iter().enumerate() {
            if row_index == 0 && self.config.has_headers {
                continue;
            }
            if let Some(value) = row.get(*column_index) {
                result.push(value.clone());
            }
        }
        
        Some(result)
    }

    fn merge_with(&mut self, other: &CsvMerger) -> Result<(), Box<dyn Error>> {
        if self.merged_data.is_empty() {
            self.merged_data = other.merged_data.clone();
            self.column_mapping = other.column_mapping.clone();
            return Ok(());
        }

        let self_headers = &self.merged_data[0];
        let other_headers = &other.merged_data[0];

        let mut column_alignment = Vec::new();
        for header in other_headers {
            if let Some(&index) = self.column_mapping.get(header) {
                column_alignment.push(Some(index));
            } else {
                column_alignment.push(None);
                self.merged_data[0].push(header.clone());
            }
        }

        for row_index in 1..other.merged_data.len() {
            let other_row = &other.merged_data[row_index];
            let mut new_row = vec!["".to_string(); self.merged_data[0].len()];

            for (col_index, &alignment) in column_alignment.iter().enumerate() {
                if let Some(target_index) = alignment {
                    if target_index < new_row.len() {
                        new_row[target_index] = other_row.get(col_index)
                            .cloned()
                            .unwrap_or_default();
                    }
                } else {
                    if let Some(value) = other_row.get(col_index) {
                        new_row.push(value.clone());
                    }
                }
            }

            self.merged_data.push(new_row);
        }

        self.rebuild_column_mapping();
        Ok(())
    }

    fn rebuild_column_mapping(&mut self) {
        self.column_mapping.clear();
        if !self.merged_data.is_empty() {
            for (index, header) in self.merged_data[0].iter().enumerate() {
                self.column_mapping.insert(header.clone(), index);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = CsvConfig {
        delimiter: ',',
        has_headers: true,
        output_delimiter: '|',
    };

    let mut merger = CsvMerger::new(config);
    
    merger.load_file("data1.csv")?;
    println!("Loaded first file with {} rows", merger.merged_data.len());

    let mut second_merger = CsvMerger::new(CsvConfig::default());
    second_merger.load_file("data2.csv")?;
    println!("Loaded second file with {} rows", second_merger.merged_data.len());

    merger.merge_with(&second_merger)?;
    println!("Merged data contains {} rows", merger.merged_data.len());

    merger.save_to_file("merged_output.csv")?;
    println!("Saved merged data to merged_output.csv");

    if let Some(column_data) = merger.get_column_data("id") {
        println!("Found {} values in 'id' column", column_data.len());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_merger_basic() -> Result<(), Box<dyn Error>> {
        let mut file1 = NamedTempFile::new()?;
        writeln!(file1, "id,name,value")?;
        writeln!(file1, "1,Alice,100")?;
        writeln!(file1, "2,Bob,200")?;

        let config = CsvConfig::default();
        let mut merger = CsvMerger::new(config);
        merger.load_file(file1.path())?;

        assert_eq!(merger.merged_data.len(), 3);
        assert_eq!(merger.column_mapping.len(), 3);
        Ok(())
    }

    #[test]
    fn test_column_extraction() -> Result<(), Box<dyn Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "id,name,value")?;
        writeln!(file, "1,Test,42")?;

        let mut merger = CsvMerger::new(CsvConfig::default());
        merger.load_file(file.path())?;

        let column_data = merger.get_column_data("id").unwrap();
        assert_eq!(column_data, vec!["1"]);

        Ok(())
    }
}