use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogAnalyzer {
    error_counts: HashMap<String, u32>,
    warning_counts: HashMap<String, u32>,
    total_lines: u32,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_counts: HashMap::new(),
            warning_counts: HashMap::new(),
            total_lines: 0,
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.total_lines += 1;

            if line.contains("ERROR") {
                let error_type = Self::extract_error_type(&line);
                *self.error_counts.entry(error_type).or_insert(0) += 1;
            } else if line.contains("WARNING") {
                let warning_type = Self::extract_warning_type(&line);
                *self.warning_counts.entry(warning_type).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    fn extract_error_type(line: &str) -> String {
        line.split_whitespace()
            .skip_while(|word| !word.contains("ERROR"))
            .nth(1)
            .unwrap_or("unknown")
            .to_string()
    }

    fn extract_warning_type(line: &str) -> String {
        line.split_whitespace()
            .skip_while(|word| !word.contains("WARNING"))
            .nth(1)
            .unwrap_or("unknown")
            .to_string()
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary");
        println!("====================");
        println!("Total lines processed: {}", self.total_lines);
        println!("\nError breakdown:");
        
        if self.error_counts.is_empty() {
            println!("  No errors found");
        } else {
            for (error_type, count) in &self.error_counts {
                println!("  {}: {}", error_type, count);
            }
        }

        println!("\nWarning breakdown:");
        if self.warning_counts.is_empty() {
            println!("  No warnings found");
        } else {
            for (warning_type, count) in &self.warning_counts {
                println!("  {}: {}", warning_type, count);
            }
        }

        let total_errors: u32 = self.error_counts.values().sum();
        let total_warnings: u32 = self.warning_counts.values().sum();
        println!("\nTotals: {} errors, {} warnings", total_errors, total_warnings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut log_data = Vec::new();
        writeln!(log_data, "2024-01-15 10:30:00 INFO Application started").unwrap();
        writeln!(log_data, "2024-01-15 10:31:00 ERROR Database connection failed").unwrap();
        writeln!(log_data, "2024-01-15 10:32:00 WARNING High memory usage detected").unwrap();
        writeln!(log_data, "2024-01-15 10:33:00 ERROR File not found").unwrap();
        writeln!(log_data, "2024-01-15 10:34:00 INFO Processing completed").unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&log_data).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(temp_file.path()).unwrap();

        assert_eq!(analyzer.total_lines, 5);
        assert_eq!(analyzer.error_counts.get("Database"), Some(&1));
        assert_eq!(analyzer.error_counts.get("File"), Some(&1));
        assert_eq!(analyzer.warning_counts.get("High"), Some(&1));
    }
}