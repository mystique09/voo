use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use domain::models::{
    agent::{AgentClient, AgentError},
    tools::Tool,
};

static API_URL: &'static str = "https://generativelanguage.googleapis.com/v1beta/models/";
static MODEL: &'static str = "gemini-2.0-flash";

#[allow(dead_code)]
#[derive(Debug)]
pub struct GeminiModel {
    api_key: String,
    reqwest: Arc<reqwest::Client>,
    conversation: Arc<Mutex<ConversationHistory>>,
}

impl GeminiModel {
    pub fn new(api_key: String) -> Self {
        let initial_prompt = Content::new(
            vec![Part::new(
                r#"
        You are an expert LLM Agent named VOO, with access to a variety of tools.
        You have access to various tools, and can use them to help you answer questions.
        When you are asked a normal question, answer normally, but if you need to use a tool, use the following format:
        {
            "name": "tool_name",           
            "input": <input schema> include the inner object, for example:
            {
                "path": "path/to/file.txt"
            }
        }

        The input schema is the input schema for the tool you are using, basically in JSON format.

        **VERY IMPORTANT**
        > Don't say anything else, just the JSON because the agent will use this to parse the response.
        > When showing the output of a tool, show the output and then show the user what they want in a list format, for example:
        > - Item 1
        > - Item 2
        > - Item 3

        **Current Tools Available:**
        read_file, list_files
        "#,
            )],
            "model",
        );

        let conversation_history = ConversationHistory::new(vec![initial_prompt], vec![]);

        Self {
            api_key,
            conversation: Arc::new(Mutex::new(conversation_history)),
            reqwest: Arc::new(reqwest::Client::new()),
        }
    }
}

#[async_trait]
impl AgentClient for GeminiModel {
    async fn ask(&self, prompt: &str) -> Result<Vec<String>, AgentError> {
        let api_key = &self.api_key;
        let url = format!("{}{}:generateContent?key={}", API_URL, MODEL, api_key);

        let content = Content::new(vec![Part::new(prompt)], "user");
        {
            self.conversation.lock().await.contents.push(content);
        }

        let history = self.conversation.lock().await.clone();
        let contents = history.contents;
        let prompt = Prompt::new(contents);

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

        let texts = response_json
            .candidates
            .iter()
            .map(|candidate| candidate.content.clone().parts)
            .map(|parts| parts.iter().map(|part| part.text.clone()).collect())
            .collect::<Vec<String>>();

        for response in texts.iter() {
            let content = Content::new(vec![Part::new(&response)], "model");
            self.conversation.lock().await.contents.push(content);
        }

        Ok(texts)
    }

    async fn add_tool(&self, tool: Arc<dyn Tool>) -> Result<(), AgentError> {
        let tool_info = format!(
            r#"Always follow the input schema!
            Always include your response using the input schema provided and nothing else.
            If a user ask about the tool, just respond normally.
            {}
            "#,
            tool.as_ref()
        );
        let gemini_tool = GeminiTool::new(tool, tool_info);

        {
            self.conversation.lock().await.tools.push(gemini_tool);
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
pub struct ConversationHistory {
    pub contents: Vec<Content>,
    pub tools: Vec<GeminiTool>,
}

impl ConversationHistory {
    pub fn new(contents: Vec<Content>, tools: Vec<GeminiTool>) -> Self {
        Self { contents, tools }
    }
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiTool {
    pub function_declarations: Vec<FunctionDeclaration>,
}

impl GeminiTool {
    pub fn new(tool: Arc<dyn Tool>, info: String) -> Self {
        let function_declarations = vec![FunctionDeclaration {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            parameters: Parameters {
                type_field: "object".to_string(),
                properties: json!([info]),
                required: vec![],
            },
        }];

        Self {
            function_declarations,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDeclaration {
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
    pub location: Location,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
}
