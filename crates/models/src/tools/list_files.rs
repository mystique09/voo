use std::fmt::Display;

use async_trait::async_trait;
use domain::models::tools::{Tool, ToolError};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ListFileTool {
    name: String,
    description: String,
    input_schema: ListFileInput,
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

        let about = format!(
            r#"Name: {}
            Description: {}
            {}
            "#,
            name, description, input_schema
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
        }
    }

    pub fn input_schema(&self) -> &ListFileInput {
        &self.input_schema
    }
}

#[async_trait]
impl Tool for ListFileTool {
    async fn exec(&self, input: String) -> Result<String, ToolError> {
        let json = input.split("Input: ").last();
        if json.is_none() {
            return Err(ToolError::ToolError("Invalid input".to_string()));
        }

        let json = json.unwrap();
        let input_schema = serde_json::from_str::<ListFileInput>(&json)
            .map_err(|e| ToolError::ToolError(format!("Invalid input: {}", e)))?;

        let path = input_schema.input.path;
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

            if file_type.is_dir() {
                continue;
            }

            let full_path = entry.path();
            let full_path_str = full_path.to_string_lossy().to_string();
            files.push(full_path_str);
        }

        let files_str = files.join(", ");
        println!("Files: {}", json);

        Ok(files_str)
    }

    fn parse_input(&self, input: String) -> Result<(), ToolError> {
        let _ = serde_json::from_str::<ListFileInput>(&input)
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
