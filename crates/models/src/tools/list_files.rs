use std::fmt::Display;

use async_trait::async_trait;
use domain::models::tools::{Tool, ToolDefinition, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub struct ListFileTool {
    name: String,
    description: String,
    input_schema: ListFileInput,
    tool_definition: ToolDefinition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListFileInput {
    input: ListFileInputInner,
}

impl Default for ListFileInput {
    fn default() -> Self {
        Self {
            input: ListFileInputInner {
                path: ".".to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListFileInputInner {
    pub path: String,
}

impl Display for ListFileTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let input_schema = serde_json::to_string(&self.input_schema).unwrap();
        let name = self.name.clone();
        let description = self.description.clone();
        let tool_definition = serde_json::to_string(&self.tool_definition).unwrap();

        let about = format!(
            r#"Name: {}
            Description: {}
            Input: {}
            Tool Definition: {}
            "#,
            name, description, input_schema, tool_definition
        );

        write!(f, "{}", about)
    }
}

impl ListFileTool {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: ListFileInput::default(),
            tool_definition: ToolDefinition {
                name: name.to_string(),
                description: description.to_string(),
                parameters: serde_json::from_str(
                    r#"{
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The path to list files from"
                            }
                        },
                        "required": ["path"]
                    }"#,
                )
                .unwrap(),
            },
        }
    }

    pub fn input_schema(&self) -> &ListFileInput {
        &self.input_schema
    }
}

#[async_trait]
impl Tool for ListFileTool {
    async fn exec(&self, input: Value) -> Result<String, ToolError> {
        let input = serde_json::from_value::<ListFileInputInner>(input)
            .map_err(|e| ToolError::ToolError(format!("Invalid input: {}", e)))?;

        let path = input.path;
        let entries = std::fs::read_dir(&path)
            .map_err(|e| ToolError::ListFile(format!("{}: {}", path, e)))?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| ToolError::ListFile(format!("{}: {}", path, e)))?;
            let file_type = entry.file_type().map_err(|e| {
                ToolError::ListFile(format!(
                    "Error getting file type for {:?}: {}",
                    entry.path(),
                    e
                ))
            })?;

            let full_path = {
                if file_type.is_dir() {
                    format!("{}/", entry.path().to_string_lossy().to_string())
                } else {
                    entry.path().to_string_lossy().to_string()
                }
            };
            files.push(full_path);
        }

        let files_str = files.join(", ");

        Ok(files_str)
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
