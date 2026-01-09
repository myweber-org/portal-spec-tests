use std::collections::HashSet;

pub fn clean_string_data(strings: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for s in strings {
        let normalized = s.trim().to_lowercase();
        if !normalized.is_empty() && seen.insert(normalized.clone()) {
            result.push(normalized);
        }
    }

    result.sort();
    result
}