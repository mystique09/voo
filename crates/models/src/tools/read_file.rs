use std::{fmt::Display, path::PathBuf};

use async_trait::async_trait;
use domain::models::tools::{Tool, ToolDefinition, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub struct ReadFileTool {
    name: String,
    description: String,
    input_schema: ReadFileInput,
    tool_definition: ToolDefinition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadFileInput {
    input: Input,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub path: String,
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
                    path: "".to_string(),
                },
            },
            tool_definition: ToolDefinition {
                name: name.to_string(),
                description: description.to_string(),
                parameters: serde_json::from_str(
                    r#"{
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The path to read the file from"
                            }
                        },
                        "required": ["path"]
                    }"#,
                )
                .unwrap(),
            },
        }
    }

    pub fn input_schema(&self) -> &ReadFileInput {
        &self.input_schema
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    async fn exec(&self, input: Value) -> Result<String, ToolError> {
        let input = serde_json::from_value::<Input>(input)
            .map_err(|e| ToolError::ToolError(e.to_string()))?;
        let path = input.path;
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

    fn tool_definition(&self) -> &ToolDefinition {
        &self.tool_definition
    }
}
