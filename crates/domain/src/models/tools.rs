use std::fmt::{Debug, Display};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum ToolError {
    FileNotFound(String),
    ListFile(String),
    ToolError(String),
}

impl Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ToolError::ListFile(path) => write!(f, "List file error: {}", path),
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
    async fn exec(&self, input: Value) -> Result<String, ToolError>;
    fn parse_input(&self, input: String) -> Result<(), ToolError>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn tool_definition(&self) -> &ToolDefinition;
}

pub type FunctionDeclaration = Vec<ToolDefinition>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Parameters,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    #[serde(rename = "type")]
    pub type_field: String,
    pub properties: Value,
    pub required: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub attendees: Attendees,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attendees {
    #[serde(rename = "type")]
    pub r#type: String,
    pub items: Option<Items>,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Items {
    #[serde(rename = "type")]
    pub r#type: String,
}
