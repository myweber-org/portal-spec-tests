
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonMap = HashMap<String, serde_json::Value>;

pub fn merge_json_files(file_paths: &[impl AsRef<Path>]) -> Result<JsonMap, Box<dyn std::error::Error>> {
    let mut merged = JsonMap::new();

    for path in file_paths {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: serde_json::Value = serde_json::from_str(&contents)?;
        
        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(merged)
}

pub fn write_merged_json(output_path: impl AsRef<Path>, data: &JsonMap) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string_pretty(data)?;
    std::fs::write(output_path, json_string)?;
    Ok(())
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

        writeln!(file1, r#"{"name": "Alice", "age": 30}"#).unwrap();
        writeln!(file2, r#"{"city": "Berlin", "active": true}"#).unwrap();

        let paths = [file1.path(), file2.path()];
        let result = merge_json_files(&paths).unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result.get("name").unwrap(), "Alice");
        assert_eq!(result.get("age").unwrap(), 30);
        assert_eq!(result.get("city").unwrap(), "Berlin");
        assert_eq!(result.get("active").unwrap(), true);
    }

    #[test]
    fn test_write_merged_json() {
        let mut data = JsonMap::new();
        data.insert("test".to_string(), serde_json::Value::Bool(true));
        
        let output_file = NamedTempFile::new().unwrap();
        write_merged_json(output_file.path(), &data).unwrap();
        
        let contents = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(contents.contains("\"test\": true"));
    }
}