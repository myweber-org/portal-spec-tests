use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<String, Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for path in paths {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let parsed: serde_json::Value = serde_json::from_str(&contents)?;
        merged_array.push(parsed);
    }

    let result = serde_json::to_string_pretty(&merged_array)?;
    Ok(result)
}

pub fn merge_json_with_key<P: AsRef<Path>>(
    paths: &[P],
    merge_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path in paths {
        let contents = fs::read_to_string(path)?;
        let parsed: serde_json::Value = serde_json::from_str(&contents)?;

        if let Some(obj) = parsed.as_object() {
            if let Some(key_value) = obj.get(merge_key) {
                if let Some(key_str) = key_value.as_str() {
                    merged_map.insert(key_str.to_string(), parsed);
                }
            }
        }
    }

    let result_map: HashMap<_, _> = merged_map.into_iter().collect();
    let result = serde_json::to_string_pretty(&result_map)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{{"id": 1, "name": "Alice"}}"#).unwrap();
        writeln!(file2, r#"{{"id": 2, "name": "Bob"}}"#).unwrap();

        let result = merge_json_files(&[file1.path(), file2.path()]).unwrap();
        assert!(result.contains("Alice") && result.contains("Bob"));
    }

    #[test]
    fn test_merge_json_with_key() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        writeln!(file1, r#"{{"id": "user1", "name": "Alice"}}"#).unwrap();
        writeln!(file2, r#"{{"id": "user2", "name": "Bob"}}"#).unwrap();

        let result = merge_json_with_key(&[file1.path(), file2.path()], "id").unwrap();
        assert!(result.contains("user1") && result.contains("user2"));
    }
}