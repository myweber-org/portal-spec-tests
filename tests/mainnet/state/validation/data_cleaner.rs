use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn process_batch(&mut self, items: Vec<&str>) -> Vec<String> {
        items
            .into_iter()
            .filter(|item| self.deduplicate(item))
            .map(|item| self.normalize_text(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "apple", "APPLE", "Banana", "banana"];
        let result = cleaner.process_batch(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub struct DataCleaner;

impl DataCleaner {
    pub fn clean_data(input: &str) -> String {
        let lines: Vec<&str> = input.lines().collect();
        let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
        
        let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
        sorted_lines.sort();
        
        sorted_lines.join("\n")
    }
    
    pub fn process_stream() -> io::Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut output = stdout.lock();
        
        let mut lines = Vec::new();
        for line in stdin.lock().lines() {
            lines.push(line?);
        }
        
        let cleaned = Self::clean_data(&lines.join("\n"));
        writeln!(output, "{}", cleaned)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\nbanana\napple";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(DataCleaner::clean_data(input), expected);
    }
    
    #[test]
    fn test_empty_input() {
        assert_eq!(DataCleaner::clean_data(""), "");
    }
}