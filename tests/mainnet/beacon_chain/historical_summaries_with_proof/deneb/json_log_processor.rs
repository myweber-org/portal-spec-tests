
use serde_json::{Value, Error};
use chrono::{DateTime, Utc};

pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_json_log(json_str: &str) -> Result<LogEntry, String> {
    let parsed: Result<Value, Error> = serde_json::from_str(json_str);
    
    match parsed {
        Ok(data) => {
            let timestamp = match data.get("timestamp") {
                Some(ts) => {
                    let ts_str = ts.as_str().unwrap_or("");
                    DateTime::parse_from_rfc3339(ts_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now())
                }
                None => Utc::now(),
            };

            let level = data.get("level")
                .and_then(|l| l.as_str())
                .unwrap_or("INFO")
                .to_string();

            let message = data.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string();

            let metadata = data.clone();

            Ok(LogEntry {
                timestamp,
                level,
                message,
                metadata,
            })
        }
        Err(e) => Err(format!("Failed to parse JSON: {}", e)),
    }
}

pub fn filter_logs_by_level(logs: Vec<LogEntry>, min_level: &str) -> Vec<LogEntry> {
    let level_order = vec!["DEBUG", "INFO", "WARN", "ERROR", "FATAL"];
    
    let min_index = level_order.iter()
        .position(|&l| l == min_level)
        .unwrap_or(0);

    logs.into_iter()
        .filter(|log| {
            level_order.iter()
                .position(|&l| l == log.level.as_str())
                .map(|idx| idx >= min_index)
                .unwrap_or(false)
        })
        .collect()
}