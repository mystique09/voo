use std::{fmt::Display, path::PathBuf};

use async_trait::async_trait;
use domain::models::tools::{Tool, ToolError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadFileTool {
    name: String,
    description: String,
    input_schema: ReadFileInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadFileInput {
    input: Input,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    file_path: String,
}

impl Display for ReadFileTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let input_schema = serde_json::to_string(&self.input_schema).unwrap();
        let name = self.name.clone();
        let description = self.description.clone();

        let about = format!(
            "Name: {}\nDescription: {}\n:{}",
            name, description, input_schema
        );

        write!(f, "{}", about)
    }
}

impl ReadFileTool {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: ReadFileInput {
                input: Input {
                    file_path: "".to_string(),
                },
            },
        }
    }

    pub fn input_schema(&self) -> &ReadFileInput {
        &self.input_schema
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    async fn exec(&self, input: String) -> Result<String, ToolError> {
        let json = input.split("Input: ").last();
        if json.is_none() {
            return Err(ToolError::ToolError("Invalid input".to_string()));
        }

        let json = json.unwrap();
        let input_schema = serde_json::from_str::<ReadFileInput>(&json)
            .map_err(|e| ToolError::ToolError(format!("Invalid input: {}", e)))?;
        let path = input_schema.input.file_path;
        let buf = PathBuf::from(path);
        let content =
            std::fs::read_to_string(buf).map_err(|e| ToolError::FileNotFound(e.to_string()))?;

        Ok(content)
    }

    fn parse_input(&self, input: String) -> Result<(), ToolError> {
        let _ = serde_json::from_str::<ReadFileInput>(&input)
            .map_err(|e| ToolError::ToolError(e.to_string()))?;

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}
