use std::fmt::{Debug, Display};

use async_trait::async_trait;

#[derive(Debug)]
pub enum ToolError {
    FileNotFound(String),
    ToolError(String),
}

#[async_trait]
pub trait Tool: Display + Debug + Send + Sync {
    async fn exec(&self, input: String) -> Result<String, ToolError>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}
