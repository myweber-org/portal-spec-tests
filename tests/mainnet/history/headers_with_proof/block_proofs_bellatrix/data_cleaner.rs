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
}use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, items: Vec<T>) -> Vec<T> {
        let mut result = Vec::new();
        for item in items {
            if self.seen.insert(item.clone()) {
                result.push(item);
            }
        }
        result
    }

    pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
        strings
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn merge_cleaners(mut self, other: DataCleaner<T>) -> Self {
        self.seen.extend(other.seen);
        self
    }
}

impl<T> Default for DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        let data = vec![1, 2, 2, 3, 4, 4, 5];
        let result = cleaner.deduplicate(data);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "".to_string(),
            "  ".to_string(),
        ];
        let result = DataCleaner::normalize_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_merge_cleaners() {
        let mut cleaner1 = DataCleaner::new();
        cleaner1.deduplicate(vec![1, 2, 3]);

        let mut cleaner2 = DataCleaner::new();
        cleaner2.deduplicate(vec![3, 4, 5]);

        let merged = cleaner1.merge_cleaners(cleaner2);
        let result = merged.deduplicate(vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(result, vec![6]);
    }
}