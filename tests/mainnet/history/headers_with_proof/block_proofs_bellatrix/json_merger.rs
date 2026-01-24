
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        let resolved = resolve_conflict(&key, existing, &value);
                        merged.insert(key, resolved);
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;

    Ok(())
}

fn resolve_conflict(key: &str, existing: &Value, new: &Value) -> Value {
    match (existing, new) {
        (Value::Array(a), Value::Array(b)) => {
            let mut combined = a.clone();
            combined.extend(b.clone());
            Value::Array(combined)
        },
        (Value::Number(_), Value::Number(_)) => new.clone(),
        (Value::String(_), Value::String(_)) => new.clone(),
        (Value::Bool(_), Value::Bool(_)) => new.clone(),
        _ => {
            eprintln!("Conflict on key '{}': keeping original value", key);
            existing.clone()
        }
    }
}