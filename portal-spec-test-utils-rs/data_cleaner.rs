use std::collections::HashSet;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
        }
    }

    pub fn process(&mut self, item: T) -> Option<T> {
        if self.seen.insert(item.clone()) {
            Some(item)
        } else {
            None
        }
    }

    pub fn process_batch(&mut self, items: Vec<T>) -> Vec<T> {
        items
            .into_iter()
            .filter_map(|item| self.process(item))
            .collect()
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }

    pub fn count_unique(&self) -> usize {
        self.seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec![1, 2, 2, 3, 4, 4, 4, 5];

        let result = cleaner.process_batch(data);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
        assert_eq!(cleaner.count_unique(), 5);
    }

    #[test]
    fn test_reset() {
        let mut cleaner = DataCleaner::new();
        cleaner.process_batch(vec!["a", "b", "c"]);
        assert_eq!(cleaner.count_unique(), 3);

        cleaner.reset();
        assert_eq!(cleaner.count_unique(), 0);

        let result = cleaner.process_batch(vec!["a", "b", "c"]);
        assert_eq!(result, vec!["a", "b", "c"]);
    }
}
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    sorted_lines.join("\n")
}

pub fn process_from_stdin() -> io::Result<()> {
    let stdin = io::stdin();
    let mut buffer = String::new();
    
    for line in stdin.lock().lines() {
        buffer.push_str(&line?);
        buffer.push('\n');
    }
    
    let cleaned = clean_data(&buffer);
    io::stdout().write_all(cleaned.as_bytes())?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\napple\nbanana";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }
    
    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}