use serde_json::{Map, Value};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, deep: bool) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if deep {
                    if let Some(base_value) = base_map.get_mut(key) {
                        merge_json(base_value, update_value, deep);
                    } else {
                        base_map.insert(key.clone(), update_value.clone());
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => *base = update.clone(),
    }
}

pub fn merge_json_with_conflict_resolution(
    base: &mut Value,
    update: &Value,
    conflict_strategy: ConflictStrategy,
) -> HashSet<String> {
    let mut conflicts = HashSet::new();
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    if base_value != update_value {
                        conflicts.insert(key.clone());
                        match conflict_strategy {
                            ConflictStrategy::KeepBase => {}
                            ConflictStrategy::UseUpdate => {
                                *base_value = update_value.clone();
                            }
                            ConflictStrategy::MergeDeep => {
                                merge_json(base_value, update_value, true);
                            }
                        }
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => {
            if base != update {
                conflicts.insert("root".to_string());
                *base = update.clone();
            }
        }
    }
    conflicts
}

pub enum ConflictStrategy {
    KeepBase,
    UseUpdate,
    MergeDeep,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": 1, "b": {"inner": 2}});
        let update = json!({"b": {"new": 3}, "c": 4});
        merge_json(&mut base, &update, false);
        assert_eq!(base["b"], json!({"new": 3}));
        assert_eq!(base["c"], 4);
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({"a": 1, "b": {"inner": 2}});
        let update = json!({"b": {"new": 3}, "c": 4});
        merge_json(&mut base, &update, true);
        assert_eq!(base["b"]["inner"], 2);
        assert_eq!(base["b"]["new"], 3);
    }

    #[test]
    fn test_conflict_detection() {
        let mut base = json!({"a": 1, "b": 2});
        let update = json!({"a": 99, "c": 3});
        let conflicts = merge_json_with_conflict_resolution(
            &mut base,
            &update,
            ConflictStrategy::KeepBase,
        );
        assert!(conflicts.contains("a"));
        assert!(!conflicts.contains("c"));
        assert_eq!(base["a"], 1);
    }
}