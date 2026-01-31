use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;
        
        if let serde_json::Value::Array(arr) = json_value {
            merged_array.extend(arr);
        } else {
            merged_array.push(json_value);
        }
    }

    let output_json = serde_json::to_string_pretty(&merged_array)?;
    fs::write(output_path, output_json)?;
    
    Ok(())
}

pub fn merge_json_with_deduplication(
    file_paths: &[&str], 
    output_path: &str, 
    key_field: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let mut unique_items = HashMap::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;
        
        let items = match json_value {
            serde_json::Value::Array(arr) => arr,
            _ => vec![json_value],
        };

        for item in items {
            if let Some(key) = item.get(key_field) {
                if let Some(key_str) = key.as_str() {
                    unique_items.insert(key_str.to_string(), item);
                }
            }
        }
    }

    let deduplicated_array: Vec<_> = unique_items.into_values().collect();
    let output_json = serde_json::to_string_pretty(&deduplicated_array)?;
    fs::write(output_path, output_json)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        fs::write(file1.path(), r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(file2.path(), r#"[{"id": 3}, {"id": 4}]"#).unwrap();

        merge_json_files(
            &[file1.path().to_str().unwrap(), file2.path().to_str().unwrap()],
            output.path().to_str().unwrap()
        ).unwrap();

        let content = fs::read_to_string(output.path()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}