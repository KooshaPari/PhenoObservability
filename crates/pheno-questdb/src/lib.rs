//! QuestDB client for PhenoObservability
//!
//! QuestDB is a time-series database that's 100x faster than InfluxDB.
//! Supports native SQL, Kafka ingest, and HTTP API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::debug;

/// QuestDB client errors
#[derive(Error, Debug)]
pub enum QuestDBError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("parse error: {0}")]
    Parse(String),
}

/// Metric point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub source: String,
    pub trace_id: Option<String>,
    pub labels: HashMap<String, String>,
}

/// QuestDB REST client
pub struct QuestDBClient {
    url: String,
    http_client: reqwest::Client,
}

impl QuestDBClient {
    /// Create new QuestDB client
    pub fn new(url: &str) -> Self {
        Self { url: url.trim_end_matches('/').to_string(), http_client: reqwest::Client::new() }
    }

    /// Insert metric using ILP (InfluxDB Line Protocol)
    pub async fn insert_metric(&self, metric: &Metric) -> Result<(), QuestDBError> {
        let line = format!(
            "metrics,{},name={} value={} {}",
            Self::format_labels(&metric.labels),
            metric.name,
            metric.value,
            metric.timestamp.timestamp_nanos()
        );

        let response = self
            .http_client
            .post(format!("{}/v1/imp", self.url))
            .body(line)
            .send()
            .await
            .map_err(|e| QuestDBError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(QuestDBError::Http(format!("HTTP {}", response.status())));
        }

        debug!("Metric inserted: {}", metric.name);
        Ok(())
    }

    /// Insert log entry
    pub async fn insert_log(&self, log: &LogEntry) -> Result<(), QuestDBError> {
        let trace = log.trace_id.as_deref().unwrap_or("none");
        let line = format!(
            "logs,level={},source={},trace_id={} message='{}' {}",
            log.level,
            log.source,
            trace,
            Self::escape_value(&log.message),
            log.timestamp.timestamp_nanos()
        );

        let response = self
            .http_client
            .post(format!("{}/v1/imp", self.url))
            .body(line)
            .send()
            .await
            .map_err(|e| QuestDBError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(QuestDBError::Http(format!("HTTP {}", response.status())));
        }

        debug!("Log inserted: {}", log.message);
        Ok(())
    }

    /// Execute SQL query
    pub async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        sql: &str,
    ) -> Result<Vec<T>, QuestDBError> {
        let url = format!("{}/v1/query?q={}", self.url, urlencoding::encode(sql));

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| QuestDBError::Http(e.to_string()))?;

        let body = response.text().await.map_err(|e| QuestDBError::Parse(e.to_string()))?;

        let result: QueryResult<T> =
            serde_json::from_str(&body).map_err(|e| QuestDBError::Parse(e.to_string()))?;

        if let Some(err) = result.error {
            return Err(QuestDBError::Parse(err));
        }

        Ok(result.data)
    }

    /// Aggregate metrics
    pub async fn aggregate(
        &self,
        name: &str,
        interval: &str,
    ) -> Result<Vec<AggregatedMetric>, QuestDBError> {
        let sql = format!(
            "SELECT timestamp, avg(value) as avg_value, min(value) as min_value, max(value) as max_value \
             FROM metrics WHERE name = '{}' SAMPLE BY {} ALIGN TO CALENDAR",
            name, interval
        );

        self.query(&sql).await
    }

    /// Format labels for ILP
    fn format_labels(labels: &HashMap<String, String>) -> String {
        labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, Self::escape_tag(v)))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Escape tag value
    fn escape_tag(s: &str) -> String {
        s.replace(',', "\\,").replace('=', "\\=")
    }

    /// Escape value (single quotes)
    fn escape_value(s: &str) -> String {
        s.replace('\'', "''")
    }
}

#[derive(Deserialize)]
struct QueryResult<T> {
    data: Vec<T>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AggregatedMetric {
    pub timestamp: DateTime<Utc>,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_labels() {
        let mut labels = HashMap::new();
        labels.insert("host".to_string(), "server1".to_string());
        labels.insert("region".to_string(), "us-west".to_string());

        let formatted = QuestDBClient::format_labels(&labels);
        assert!(formatted.contains("host=server1"));
        assert!(formatted.contains("region=us-west"));
    }

    #[test]
    fn test_escape_value() {
        let escaped = QuestDBClient::escape_value("Hello's World");
        assert!(escaped.contains("''"));
    }
}
