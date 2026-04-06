//! PhenotypeMCPServer - MCP Protocol Server inspired by fastmcp

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum MCPServerError {
    #[error("tool not found: {0}")]
    ToolNotFound(String),
    #[error("resource not found: {0}")]
    ResourceNotFound(String),
    #[error("handler failed: {0}")]
    HandlerFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ContentItem>,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub text: Option<String>,
}

pub struct ToolHandler {
    pub tool: Tool,
    pub handler: Arc<dyn Fn(Value) -> Result<Value, MCPServerError> + Send + Sync>,
}

pub struct MCPServer {
    tools: Arc<RwLock<HashMap<String, ToolHandler>>>,
    resources: Arc<RwLock<HashMap<String, Resource>>>,
}

impl MCPServer {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_tool<F>(&self, name: String, description: String, schema: Value, handler: F)
    where
        F: Fn(Value) -> Result<Value, MCPServerError> + Send + Sync + 'static,
    {
        let tool = Tool {
            name: name.clone(),
            description,
            input_schema: schema,
        };
        self.tools.write().await.insert(name, ToolHandler {
            tool,
            handler: Arc::new(handler),
        });
    }

    pub async fn list_tools(&self) -> Vec<Tool> {
        self.tools.read().await.values().map(|h| h.tool.clone()).collect()
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<ToolResult, MCPServerError> {
        let tools = self.tools.read().await;
        let handler = tools.get(name).ok_or(MCPServerError::ToolNotFound(name.to_string()))?;
        let result = (handler.handler)(arguments).map_err(|e| MCPServerError::HandlerFailed(e.to_string()))?;
        Ok(ToolResult {
            content: vec![ContentItem {
                content_type: "text".to_string(),
                text: Some(result.to_string()),
            }],
            is_error: false,
        })
    }

    pub async fn register_resource(&self, resource: Resource) {
        self.resources.write().await.insert(resource.uri.clone(), resource);
    }

    pub async fn list_resources(&self) -> Vec<Resource> {
        self.resources.read().await.values().cloned().collect()
    }

    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContent, MCPServerError> {
        let resources = self.resources.read().await;
        let resource = resources.get(uri).ok_or(MCPServerError::ResourceNotFound(uri.to_string()))?;
        Ok(ResourceContent {
            uri: uri.to_string(),
            text: Some(format!("Resource: {}", resource.name)),
        })
    }
}

impl Default for MCPServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = MCPServer::new();
        assert!(server.list_tools().await.is_empty());
    }

    #[tokio::test]
    async fn test_register_and_call() {
        let server = MCPServer::new();
        server.register_tool(
            "echo".to_string(),
            "Echo back input".to_string(),
            serde_json::json!({"type": "object"}),
            |args| Ok(args),
        ).await;
        assert_eq!(server.list_tools().await.len(), 1);
        let result = server.call_tool("echo", serde_json::json!({"test": "value"})).await.unwrap();
        assert!(!result.is_error);
    }
}
