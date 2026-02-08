use serde_json::{Value, Map};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json_array: Vec<Value> = serde_json::from_str(&content)?;

        for item in json_array {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                if !seen_ids.contains(id) {
                    seen_ids.insert(id.to_string());
                    merged_array.push(item);
                }
            } else {
                merged_array.push(item);
            }
        }
    }

    let output_json = serde_json::to_string_pretty(&merged_array)?;
    fs::write(output_path, output_json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#;
        let json2 = r#"[{"id": "2", "name": "Bob"}, {"id": "3", "name": "Charlie"}]"#;

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, json1).unwrap();
        fs::write(&file2, json2).unwrap();

        let paths = [file1.path(), file2.path()];
        merge_json_files(&paths, output_file.path()).unwrap();

        let result = fs::read_to_string(output_file.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}