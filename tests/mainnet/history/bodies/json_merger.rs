
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        
        if let Value::Object(obj) = json {
            merge_objects(&mut merged, obj);
        }
    }
    
    let output_value = Value::Object(merged);
    let output_json = serde_json::to_string_pretty(&output_value)?;
    fs::write(output_path, output_json)?;
    
    Ok(())
}

fn merge_objects(base: &mut Map<String, Value>, new: Map<String, Value>) {
    for (key, new_value) in new {
        match base.get_mut(&key) {
            Some(existing_value) => {
                if let (Value::Object(mut existing_obj), Value::Object(new_obj)) = (existing_value.clone(), new_value) {
                    let mut existing_map = match existing_obj {
                        Value::Object(map) => map,
                        _ => Map::new(),
                    };
                    merge_objects(&mut existing_map, new_obj);
                    base.insert(key, Value::Object(existing_map));
                } else if existing_value != &new_value {
                    base.insert(key.clone() + "_conflict", existing_value.clone());
                    base.insert(key, new_value);
                }
            }
            None => {
                base.insert(key, new_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();
        
        fs::write(&file1, r#"{"common": "value1", "unique1": "data1"}"#).unwrap();
        fs::write(&file2, r#"{"common": "value2", "unique2": "data2"}"#).unwrap();
        
        merge_json_files(&[file1.path(), file2.path()], output.path()).unwrap();
        
        let result_content = fs::read_to_string(output.path()).unwrap();
        let result: Value = serde_json::from_str(&result_content).unwrap();
        
        assert_eq!(result["common"], json!("value2"));
        assert_eq!(result["unique1"], json!("data1"));
        assert_eq!(result["unique2"], json!("data2"));
        assert!(result.get("common_conflict").is_some());
    }
}use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        merged_array.push(json_value);
    }

    let output_json = json!(merged_array);
    let output_dir = Path::new(output_path).parent().unwrap();
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    fs::write(output_path, output_json.to_string())?;
    Ok(())
}