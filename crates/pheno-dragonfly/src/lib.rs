#![doc = "Dragonfly client for PhenoObservability - Redis-compatible, multi-threaded cache"]

use chrono::{DateTime, Utc};
use phenotype_errors::RepositoryError;
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Dragonfly client result type
pub type Result<T> = std::result::Result<T, RepositoryError>;

/// Session data stored in Dragonfly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub data: HashMap<String, String>,
    pub ttl_seconds: u64,
}

/// Metric cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCache {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Dragonfly client for caching and sessions
#[derive(Clone)]
pub struct DragonflyClient {
    conn: ConnectionManager,
}

impl DragonflyClient {
    /// Create new Dragonfly client
    pub async fn new(url: &str) -> Result<Self> {
        info!("Connecting to Dragonfly: {}", url);
        let client = Client::open(url).map_err(|e| RepositoryError::Connection(e.to_string()))?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| RepositoryError::Connection(e.to_string()))?;
        info!("Dragonfly connection established");
        Ok(Self { conn })
    }

    /// Set a session with TTL
    pub async fn set_session(&self, session: &Session) -> Result<()> {
        let key = format!("session:{}", session.id);
        let value = serde_json::to_string(session)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
        let mut conn = self.conn.clone();
        let _: () = conn
            .set_ex(&key, &value, session.ttl_seconds)
            .await
            .map_err(|e| RepositoryError::Query(e.to_string()))?;
        debug!("Session stored: {}", session.id);
        Ok(())
    }

    /// Get session by ID
    pub async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        let key = format!("session:{}", id);
        let mut conn = self.conn.clone();
        let value: Option<String> =
            conn.get(&key).await.map_err(|e| RepositoryError::Query(e.to_string()))?;
        match value {
            Some(v) => {
                let session: Session = serde_json::from_str(&v)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// Set with custom TTL
    pub async fn set_with_ttl(
        &self,
        key: &str,
        value: &[u8],
        ttl_seconds: u64,
    ) -> Result<()> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .set_ex(key, value, ttl_seconds)
            .await
            .map_err(|e| RepositoryError::Query(e.to_string()))?;
        Ok(())
    }

    /// Get value
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut conn = self.conn.clone();
        let value: Option<String> =
            conn.get(key).await.map_err(|e| RepositoryError::Query(e.to_string()))?;
        Ok(value.map(|v| v.into_bytes()))
    }

    /// Increment counter (atomic)
    pub async fn incr(&self, key: &str, amount: i64) -> Result<i64> {
        let mut conn = self.conn.clone();
        let result: i64 =
            conn.incr(key, amount).await.map_err(|e| RepositoryError::Query(e.to_string()))?;
        Ok(result)
    }

    /// Health check
    pub async fn ping(&self) -> Result<bool> {
        let mut conn = self.conn.clone();
        let result: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| RepositoryError::Query(e.to_string()))?;
        Ok(result == "PONG")
    }
}
