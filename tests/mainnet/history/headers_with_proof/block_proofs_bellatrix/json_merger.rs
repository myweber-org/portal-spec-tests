use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> io::Result<Value> {
    let mut merged_array = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let parsed: Value = serde_json::from_str(&content)?;
        merged_array.push(parsed);
    }

    Ok(json!(merged_array))
}

pub fn write_merged_json<P: AsRef<Path>>(paths: &[P], output_path: P) -> io::Result<()> {
    let merged = merge_json_files(paths)?;
    let json_string = serde_json::to_string_pretty(&merged)?;
    fs::write(output_path, json_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, r#"{"id": 1, "name": "Alice"}"#).unwrap();
        fs::write(&file2, r#"{"id": 2, "name": "Bob"}"#).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_files(&paths).unwrap();

        assert!(result.is_array());
        let array = result.as_array().unwrap();
        assert_eq!(array.len(), 2);
        assert_eq!(array[0]["name"], "Alice");
        assert_eq!(array[1]["name"], "Bob");
    }
}