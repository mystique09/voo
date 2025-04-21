use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use domain::models::agent::{AgentClient, AgentError};

static API_URL: &'static str = "https://generativelanguage.googleapis.com/v1beta/models/";
static MODEL: &'static str = "gemini-2.0-flash";

#[allow(dead_code)]
#[derive(Debug)]
pub struct GeminiModel {
    api_key: String,
    reqwest: Arc<reqwest::Client>,
    conversation: Arc<Mutex<Vec<Content>>>,
}

impl GeminiModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            conversation: Arc::new(Mutex::new(vec![])),
            reqwest: Arc::new(reqwest::Client::new()),
        }
    }
}

#[async_trait]
impl AgentClient for GeminiModel {
    async fn ask(&self, prompt: &str) -> Result<String, AgentError> {
        let api_key = &self.api_key;
        let url = format!("{}{}:generateContent?key={}", API_URL, MODEL, api_key);

        let content = Content::new(vec![Part::new(prompt)], "user");
        {
            self.conversation.lock().await.push(content.clone());
        }

        let history = self.conversation.lock().await.clone();
        let prompt = Prompt::new(history);

        let response = self
            .reqwest
            .post(url)
            .json(&prompt)
            .send()
            .await
            .map_err(|e| AgentError::AgentError(Some(e.to_string())))?;

        let response_json = response
            .json::<GeminiResponse>()
            .await
            .map_err(|e| AgentError::AgentError(Some(e.to_string())))?;

        let text = response_json
            .candidates
            .get(0)
            .unwrap()
            .content
            .parts
            .get(0)
            .unwrap()
            .text
            .clone();

        Ok(text)
    }

    async fn perform_tool(&self, prompt: &str) -> Result<String, AgentError> {
        Ok(prompt.to_string())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
    contents: Vec<Content>,
}

impl Prompt {
    pub fn new(contents: Vec<Content>) -> Self {
        Self { contents }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
    pub usage_metadata: UsageMetadata,
    pub model_version: String,
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
pub struct Content {
    pub parts: Vec<Part>,
    pub role: String,
}

impl Content {
    pub fn new(parts: Vec<Part>, role: &str) -> Self {
        Self {
            parts,
            role: role.to_string(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub text: String,
}

impl Part {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
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
