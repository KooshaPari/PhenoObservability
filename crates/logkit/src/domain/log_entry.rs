//! Log Entry

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Level;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub level: Level,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub fields: Vec<(String, serde_json::Value)>,
}

impl LogEntry {
    pub fn new(level: Level, message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            level,
            message: message.into(),
            timestamp: Utc::now(),
            fields: Vec::new(),
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.push((key.into(), value));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_level_message_and_empty_fields() {
        let entry = LogEntry::new(Level::Warn, "disk full");

        assert_eq!(entry.level, Level::Warn);
        assert_eq!(entry.message, "disk full");
        assert!(entry.fields.is_empty());
        assert_ne!(entry.id, Uuid::nil());
        assert!(entry.timestamp <= Utc::now());
    }

    #[test]
    fn with_field_appends_key_value_and_returns_self() {
        let entry = LogEntry::new(Level::Info, "test message")
            .with_field("user_id", serde_json::json!(42))
            .with_field("action", serde_json::json!("login"));

        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields[0].0, "user_id");
        assert_eq!(entry.fields[0].1, serde_json::json!(42));
        assert_eq!(entry.fields[1].0, "action");
        assert_eq!(entry.fields[1].1, serde_json::json!("login"));
        assert_eq!(entry.level, Level::Info);
        assert_eq!(entry.message, "test message");
    }
}
