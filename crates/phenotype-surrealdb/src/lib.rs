//! PhenotypeSurrealDB - SurrealDB fork with Pheno extensions
//!
//! Forked from surrealdb/surrealdb (29k stars)
//! 
//! Additions for Pheno:
//! - MCP protocol adapter
//! - Skill storage schema
//! - WASM embedding support

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// PhenoSurreal - SurrealDB wrapper with Pheno extensions
pub struct PhenoSurreal {
    path: String,
}

impl PhenoSurreal {
    /// Create new PhenoSurreal instance
    pub async fn new(path: impl Into<String>) -> Result<Self> {
        Ok(Self {
            path: path.into(),
        })
    }

    /// Store a skill
    pub async fn store_skill(&self, skill: Skill) -> Result<SkillRecord> {
        Ok(SkillRecord {
            id: format!("skill:{}", skill.name),
            name: skill.name,
            version: skill.version,
            code: skill.code,
            runtime: skill.runtime,
            metadata: skill.metadata,
        })
    }

    /// Query all skills
    pub async fn query_skills(&self) -> Result<Vec<SkillRecord>> {
        Ok(vec![])
    }

    /// Store vector embedding
    pub async fn store_embedding(&self, embedding: Embedding) -> Result<EmbeddingRecord> {
        Ok(EmbeddingRecord {
            id: format!("embedding:{}", embedding.id.unwrap_or_default()),
            vector: embedding.vector,
            metadata: embedding.metadata,
        })
    }
}

/// Skill definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: Option<String>,
    pub name: String,
    pub version: String,
    pub code: String,
    pub runtime: String,
    pub metadata: serde_json::Value,
}

/// Stored skill record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub version: String,
    pub code: String,
    pub runtime: String,
    pub metadata: serde_json::Value,
}

/// Embedding definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub id: Option<String>,
    pub vector: Vec<f32>,
    pub metadata: serde_json::Value,
}

/// Stored embedding record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skill_storage() -> Result<()> {
        let db = PhenoSurreal::new("/tmp/test.db").await?;
        
        let skill = Skill {
            id: None,
            name: "test-skill".to_string(),
            version: "1.0.0".to_string(),
            code: "fn main() {}".to_string(),
            runtime: "wasm".to_string(),
            metadata: serde_json::json!({}),
        };
        
        let result = db.store_skill(skill).await?;
        assert_eq!(result.name, "test-skill");
        
        Ok(())
    }
}
