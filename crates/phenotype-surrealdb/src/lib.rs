//! PhenotypeSurrealDB - SurrealDB client with Pheno extensions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// SurrealDB client with Pheno extensions
pub struct PhenoSurreal {
    client: Arc<RwLock<surrealdb::Surreal<surrealdb::client::Client>>>,
}

impl PhenoSurreal {
    /// Connect to SurrealDB server
    pub async fn connect(endpoint: &str, namespace: &str, database: &str) -> Result<Self> {
        let client = surrealdb::Surreal::new(endpoint).await?;
        client.use_ns(namespace).use_db(database).await?;
        Ok(Self {
            client: Arc::new(RwLock::new(client)),
        })
    }

    /// Query skills
    pub async fn query_skills<T: for<'de> Deserialize<'de>>(&self) -> Result<Vec<T>> {
        let client = self.client.read().await;
        let skills: Vec<T> = client.select("skill").await?;
        Ok(skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_connect() {
        let _ = PhenoSurreal::connect("ws://localhost:8000", "pheno", "main").await;
    }
}
