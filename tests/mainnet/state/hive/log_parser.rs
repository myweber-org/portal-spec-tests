
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use regex::Regex;

pub struct LogParser {
    error_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        let pattern = r"ERROR|FATAL|CRITICAL|FAILED";
        let error_pattern = Regex::new(pattern).unwrap();
        LogParser { error_pattern }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<String>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if self.error_pattern.is_match(&line) {
                errors.push(line);
            }
        }

        Ok(errors)
    }

    pub fn count_errors<P: AsRef<Path>>(&self, path: P) -> io::Result<usize> {
        let errors = self.parse_file(path)?;
        Ok(errors.len())
    }
}

pub fn analyze_log_directory(dir_path: &str) -> io::Result<Vec<(String, usize)>> {
    let parser = LogParser::new();
    let mut results = Vec::new();

    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
            if let Ok(count) = parser.count_errors(&path) {
                if count > 0 {
                    let filename = path.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    results.push((filename, count));
                }
            }
        }
    }

    results.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(results)
}