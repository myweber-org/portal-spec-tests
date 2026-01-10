
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut conflict_log = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if merged.contains_key(&key) {
                    let existing = merged.get(&key).unwrap();
                    if existing != &value {
                        conflict_log.push(format!(
                            "Conflict at key '{}': existing {:?}, new {:?}",
                            key, existing, value
                        ));
                        merged.insert(key, Value::Array(vec![existing.clone(), value]));
                    }
                } else {
                    merged.insert(key, value);
                }
            }
        }
    }

    let result = Value::Object(merged);
    let output = serde_json::to_string_pretty(&result)?;
    fs::write(output_path, output)?;

    if !conflict_log.is_empty() {
        let log_content = conflict_log.join("\n");
        fs::write("merge_conflicts.log", log_content)?;
    }

    Ok(())
}

pub fn find_unique_keys<P: AsRef<Path>>(paths: &[P]) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let mut all_keys = HashSet::new();
    let mut common_keys = HashSet::new();
    let mut first = true;

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(obj) = json {
            let current_keys: HashSet<String> = obj.keys().cloned().collect();
            all_keys.extend(current_keys.iter().cloned());

            if first {
                common_keys = current_keys;
                first = false;
            } else {
                common_keys = common_keys.intersection(&current_keys).cloned().collect();
            }
        }
    }

    let unique_keys: HashSet<String> = all_keys.difference(&common_keys).cloned().collect();
    Ok(unique_keys)
}