
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();
    let mut key_counter: HashMap<String, usize> = HashMap::new();

    for file_path in file_paths {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", file_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                let mut final_key = key.clone();
                if merged_map.contains_key(&key) {
                    let count = key_counter.entry(key.clone()).or_insert(1);
                    *count += 1;
                    final_key = format!("{}_{}", key, count);
                }
                merged_map.insert(final_key, value);
            }
        } else {
            eprintln!("Warning: File {} does not contain a JSON object, skipping.", file_path);
        }
    }

    let output_file = File::create(output_path)?;
    serde_json::to_writer_pretty(output_file, &Value::Object(merged_map))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30}"#;
        let json2 = r#"{"name": "Bob", "city": "London"}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_json_files(&input_paths, output_path).unwrap();

        let mut output_content = String::new();
        File::open(output_path)
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();

        let parsed: Value = serde_json::from_str(&output_content).unwrap();
        assert_eq!(parsed["name"], Value::String("Alice".to_string()));
        assert_eq!(parsed["name_2"], Value::String("Bob".to_string()));
        assert_eq!(parsed["age"], Value::Number(30.into()));
        assert_eq!(parsed["city"], Value::String("London".to_string()));
    }
}
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
    dedup_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();
    let mut merged_array = Vec::new();

    for path in input_paths {
        let content = fs::read_to_string(path)?;
        let json_data: Value = serde_json::from_str(&content)?;

        if let Value::Array(arr) = json_data {
            for item in arr {
                if let Some(key_value) = item.get(dedup_key) {
                    if let Some(key_str) = key_value.as_str() {
                        merged_map.insert(key_str.to_string(), item.clone());
                    }
                }
            }
        }
    }

    for (_, value) in merged_map {
        merged_array.push(value);
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let data1 = r#"[{"id": "1", "name": "Alice"}, {"id": "2", "name": "Bob"}]"#;
        let data2 = r#"[{"id": "2", "name": "Robert"}, {"id": "3", "name": "Charlie"}]"#;

        fs::write(&file1, data1).unwrap();
        fs::write(&file2, data2).unwrap();

        merge_json_files(&[&file1, &file2], &output_file, "id").unwrap();

        let result = fs::read_to_string(output_file).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}