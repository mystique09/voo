use std::fmt::{Debug, Display};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ToolError {
    FileNotFound(String),
    ToolError(String),
}

impl Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ToolError::ToolError(msg) => write!(f, "Tool error: {}", msg),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolNameInput {
    pub name: String,
}

#[async_trait]
pub trait Tool: Display + Debug + Send + Sync {
    async fn exec(&self, input: String) -> Result<String, ToolError>;
    fn parse_input(&self, input: String) -> Result<(), ToolError>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}
