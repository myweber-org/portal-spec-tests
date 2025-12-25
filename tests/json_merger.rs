use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in input_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str).into());
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain an object at the root level".into());
        }
    }

    let merged_value = Value::Object(merged_map);
    let serialized = serde_json::to_string_pretty(&merged_value)?;
    fs::write(output_path, serialized)?;

    Ok(())
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

type JsonValue = serde_json::Value;

pub fn merge_json_files(file_paths: &[&str]) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut merged = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_data: JsonValue = serde_json::from_str(&contents)?;
        merged.insert(file_name, json_data);
    }

    Ok(serde_json::to_value(merged)?)
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

        writeln!(file1, r#"{{ "key1": "value1" }}"#).unwrap();
        writeln!(file2, r#"{{ "key2": 42 }}"#).unwrap();

        let result = merge_json_files(&[
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .unwrap();

        assert!(result.get("file1").is_some());
        assert!(result.get("file2").is_some());
        assert_eq!(
            result["file1"]["key1"].as_str().unwrap(),
            "value1"
        );
        assert_eq!(
            result["file2"]["key2"].as_u64().unwrap(),
            42
        );
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut merged_map = HashMap::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let json_value: serde_json::Value = serde_json::from_str(&contents)?;

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        } else {
            return Err("Each JSON file must contain a JSON object".into());
        }
    }

    Ok(serde_json::Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let json1 = r#"{"name": "Alice", "age": 30}"#;
        let json2 = r#"{"city": "London", "active": true}"#;

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(json1.as_bytes()).unwrap();
        file2.write_all(json2.as_bytes()).unwrap();

        let paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let result_obj = result.as_object().unwrap();

        assert_eq!(result_obj.get("name").unwrap(), "Alice");
        assert_eq!(result_obj.get("age").unwrap(), 30);
        assert_eq!(result_obj.get("city").unwrap(), "London");
        assert_eq!(result_obj.get("active").unwrap(), true);
    }
}use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn merge_json_files(file_paths: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged_map = Map::new();

    for path_str in file_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                merged_map.insert(key, value);
            }
        }
    }

    Ok(Value::Object(merged_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_json_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        let data1 = json!({"name": "Alice", "age": 30});
        let data2 = json!({"city": "Berlin", "active": true});

        write!(file1, "{}", data1).unwrap();
        write!(file2, "{}", data2).unwrap();

        let paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        let result = merge_json_files(&paths).unwrap();
        let expected = json!({
            "name": "Alice",
            "age": 30,
            "city": "Berlin",
            "active": true
        });

        assert_eq!(result, expected);
    }
}use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, extension: &Value, overwrite_arrays: bool) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value, overwrite_arrays);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (Value::Array(base_arr), Value::Array(ext_arr)) if !overwrite_arrays => {
            let mut seen = HashSet::new();
            for item in base_arr.iter() {
                if let Some(obj) = item.as_object() {
                    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                        seen.insert(id.to_string());
                    }
                }
            }
            
            for item in ext_arr {
                if let Some(obj) = item.as_object() {
                    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                        if !seen.contains(id) {
                            base_arr.push(item.clone());
                        }
                    } else {
                        base_arr.push(item.clone());
                    }
                } else {
                    base_arr.push(item.clone());
                }
            }
        }
        (base, extension) if overwrite_arrays || !matches!(extension, Value::Array(_)) => {
            *base = extension.clone();
        }
        _ => {}
    }
}

pub fn merge_json_with_defaults(base: &mut Value, extension: &Value) {
    merge_json(base, extension, false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let extension = json!({"b": {"d": 3}, "e": 4});
        
        merge_json_with_defaults(&mut base, &extension);
        
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 2);
        assert_eq!(base["b"]["d"], 3);
        assert_eq!(base["e"], 4);
    }

    #[test]
    fn test_array_merge_no_duplicates() {
        let mut base = json!({"items": [{"id": "1", "name": "first"}]});
        let extension = json!({"items": [{"id": "1", "name": "updated"}, {"id": "2", "name": "second"}]});
        
        merge_json_with_defaults(&mut base, &extension);
        
        assert_eq!(base["items"].as_array().unwrap().len(), 2);
        assert_eq!(base["items"][0]["name"], "first");
        assert_eq!(base["items"][1]["name"], "second");
    }
}