use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use domain::models::{
    agent::{AgentClient, AgentError, Content, Part},
    tools::{FunctionDeclaration, Tool},
};

static API_URL: &'static str = "https://generativelanguage.googleapis.com/v1beta/models/";
static MODEL: &'static str = "gemini-2.0-flash-001";

#[allow(dead_code)]
#[derive(Debug)]
pub struct GeminiModel {
    api_key: String,
    reqwest: Arc<reqwest::Client>,
    conversation: Arc<Mutex<ConversationHistory>>,
    tools: Arc<Mutex<GeminiTool>>,
}

impl GeminiModel {
    pub fn new(api_key: String) -> Self {
        let initial_prompt = Content::new(
            vec![Part::new(
                r#"
        You are an expert LLM Agent named VOO, with access to a variety of tools.
        You have access to various tools, and can use them to help you answer questions.
        "#,
            )],
            "model",
        );

        let conversation_history = ConversationHistory::new(vec![initial_prompt]);
        let tools = Arc::new(Mutex::new(GeminiTool {
            function_declarations: vec![],
        }));

        Self {
            api_key,
            conversation: Arc::new(Mutex::new(conversation_history)),
            reqwest: Arc::new(reqwest::Client::new()),
            tools,
        }
    }
}

#[async_trait]
impl AgentClient for GeminiModel {
    async fn ask(&self, prompt: &str) -> Result<Vec<Content>, AgentError> {
        let api_key = &self.api_key;
        let url = format!("{}{}:generateContent?key={}", API_URL, MODEL, api_key);

        let content = Content::new(vec![Part::new(prompt)], "user");
        {
            self.conversation.lock().await.contents.push(content);
        }

        let tools = self.tools.lock().await.clone();
        let history = self.conversation.lock().await.clone();
        let contents = history.contents;

        let prompt = Prompt::new(contents, tools);

        let response = self
            .reqwest
            .post(url)
            .json(&prompt)
            .send()
            .await
            .map_err(|e| AgentError::AgentError(Some(e.to_string())))?;

        let text = response
            .text()
            .await
            .map_err(|e| AgentError::AgentError(Some(e.to_string())))?;

        let response_json = serde_json::from_str::<GeminiResponse>(&text)
            .map_err(|e| AgentError::AgentError(Some(e.to_string())))?;

        if let Some(error) = response_json.error {
            let error_msg = error.message;

            if error_msg.contains("API key expired.") {
                return Err(AgentError::ExpiredApiKey);
            }

            return Err(AgentError::AgentError(Some(error_msg)));
        }

        let contents = response_json
            .candidates
            .unwrap_or_default()
            .iter()
            .map(|candidate| candidate.content.clone())
            .collect::<Vec<Content>>();

        let parts = contents
            .iter()
            .map(|content| content.parts.clone())
            .flatten()
            .collect::<Vec<Part>>();

        let texts = parts
            .iter()
            .map(|part| part.text.clone().unwrap_or_default())
            .collect::<Vec<String>>();

        if texts.is_empty() {
            return Err(AgentError::AgentError(Some(
                "No response from Gemini".to_string(),
            )));
        }

        for response in texts.iter() {
            if response.is_empty() {
                continue;
            }

            let response = format!(r##"{}"##, response);
            _ = self.add_system_prompt(&response).await;
        }

        Ok(contents)
    }

    async fn add_tool(&self, tool: Arc<dyn Tool>) -> Result<(), AgentError> {
        let tool_definition = tool.tool_definition();
        {
            self.tools
                .lock()
                .await
                .function_declarations
                .push(tool_definition.clone());
        }

        Ok(())
    }

    async fn add_system_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        let content = Content::new(vec![Part::new(prompt)], "model");
        {
            self.conversation
                .lock()
                .await
                .contents
                .push(content.clone());
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
    contents: Vec<Content>,
    tools: Vec<GeminiTool>,
}

impl Prompt {
    pub fn new(contents: Vec<Content>, tools: GeminiTool) -> Self {
        Self {
            contents,
            tools: vec![tools],
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Option<Vec<Candidate>>,
    pub usage_metadata: Option<UsageMetadata>,
    pub model_version: Option<String>,
    pub error: Option<GeminiError>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiError {
    pub code: i64,
    pub message: String,
    pub status: String,
    pub details: Vec<Detail>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub reason: Option<String>,
    pub domain: Option<String>,
    pub metadata: Option<Metadata>,
    pub locale: Option<String>,
    pub message: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub service: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Content,
    pub finish_reason: String,
    pub avg_logprobs: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationHistory {
    pub contents: Vec<Content>,
}

impl ConversationHistory {
    pub fn new(contents: Vec<Content>) -> Self {
        Self { contents }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    pub prompt_token_count: i64,
    pub candidates_token_count: i64,
    pub total_token_count: i64,
    pub prompt_tokens_details: Vec<PromptTokensDetail>,
    pub candidates_tokens_details: Vec<CandidatesTokensDetail>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTokensDetail {
    pub modality: String,
    pub token_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidatesTokensDetail {
    pub modality: String,
    pub token_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiTool {
    pub function_declarations: FunctionDeclaration,
}

impl GeminiTool {
    pub fn new(tool: Arc<dyn Tool>) -> Self {
        let function_declarations = vec![tool.tool_definition().clone()];

        Self {
            function_declarations,
        }
    }
}
