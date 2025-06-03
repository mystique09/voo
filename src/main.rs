use std::{collections::HashMap, sync::Arc, time::Duration};

use domain::models::{
    agent::{Agent, AgentError, AgentRole, FunctionCall, Part},
    tools::Tool,
};
use models::{
    models::gemini::GeminiModel,
    tools::{list_files::ListFileTool, read_file::ReadFileTool},
};
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::{Layer, layer::SubscriberExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    setup_tracing();

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    let gemini = GeminiModel::new(api_key);
    let read_file_tool = ReadFileTool::new(
        "read_file",
        "Read the contents of a given relative file path. Use this when you want to see what's inside a file. Do not use this with directory names.",
    );
    let list_file_tool = ListFileTool::new(
        "list_files",
        "List the files of a given relative file path. Use this when you want to see what's inside a directory.",
    );

    let read_file_tool: Arc<dyn Tool> = Arc::new(read_file_tool);
    let list_file_tool: Arc<dyn Tool> = Arc::new(list_file_tool);

    let agent = Agent::new(gemini);
    agent
        .add_tool(read_file_tool)
        .await
        .map_err(|e| anyhow::anyhow!("Error adding tool: {}", e))?;
    agent
        .add_tool(list_file_tool)
        .await
        .map_err(|e| anyhow::anyhow!("Error adding tool: {}", e))?;

    let _crate_name = env!("CARGO_PKG_NAME").to_uppercase();
    let _crate_version = env!("CARGO_PKG_VERSION");

    println!("Chat with VOO (use 'ctrl-c' to quit)\n");

    let mut should_read_input = true;

    'main: loop {
        let input = if should_read_input {
            let input = agent
                .reader()
                .read()
                .map_err(|e| anyhow::anyhow!("Error reading input: {}", e))?;

            input
        } else {
            "".to_string()
        };

        if input.starts_with("exit") {
            info!("Bye!");
            break;
        }

        if !should_read_input {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        let response = agent.client().ask(&input).await;
        let agent_tools = agent.tools();

        match response {
            Ok(responses) => {
                for response in responses {
                    let function_calls = response
                        .parts
                        .iter()
                        .map(|part| part.function_call.clone())
                        .collect::<Vec<Option<FunctionCall>>>();
                    let has_function_call = function_calls.iter().any(|call| call.is_some());

                    if has_function_call {
                        let tool_use = loop {
                            let outputs =
                                perform_function_call(agent_tools.clone(), &function_calls).await;

                            match outputs {
                                Ok(outputs) => {
                                    break outputs;
                                }
                                Err(e) => {
                                    error!("\x1b[41mvoo>\x1b[0m {}", e);
                                    let err = format!("Error performing function call: {}", e);
                                    _ = agent
                                        .client()
                                        .add_system_prompt(&err, AgentRole::User)
                                        .await;
                                    should_read_input = false;
                                    continue 'main;
                                }
                            }
                        };

                        for output in &tool_use {
                            if output.is_empty() {
                                continue;
                            }

                            _ = agent
                                .client()
                                .add_system_prompt(&output, AgentRole::User)
                                .await;
                        }

                        if !tool_use.is_empty() {
                            should_read_input = false;
                            continue 'main;
                        }
                    } else {
                        print_response(&agent, &response.parts).await;
                        should_read_input = true;
                    }
                }
            }
            Err(AgentError::ExpiredApiKey) => {
                error!(
                    "\x1b[41mvoo>\x1b[0m API key expired. Please update the API key in the .env file."
                );
            }
            Err(e) => {
                error!("\x1b[41mvoo>\x1b[0m {}", e);
                _ = agent
                    .client()
                    .add_system_prompt(&e.to_string(), AgentRole::User)
                    .await;
            }
        }
    }

    Ok(())
}

async fn print_response(agent: &Agent, parts: &[Part]) {
    for part in parts {
        let text = part.text.as_ref();
        if text.is_none() {
            continue;
        }

        if let Some(text) = text {
            println!("\x1b[32mvoo>\x1b[0m {}", text);
            _ = agent
                .client()
                .add_system_prompt(&text, AgentRole::User)
                .await;
        }
    }
}

async fn perform_function_call(
    agent_tools: Arc<Mutex<HashMap<String, Arc<dyn Tool>>>>,
    function_calls: &[Option<FunctionCall>],
) -> anyhow::Result<Vec<String>> {
    let mut tool_outputs = vec![];
    for (_index, function_call) in function_calls.iter().enumerate() {
        if let Some(function_call) = function_call {
            let tool_name = function_call.name.clone();
            let tool_input = function_call.args.clone();
            let tool_input_str = serde_json::to_string(&tool_input).unwrap();
            let tool = agent_tools.lock().await.get(&tool_name).unwrap().clone();

            println!("\x1b[33m{}> {}\x1b[0m", tool_name, tool_input_str);

            let tool_output = tool.exec(tool_input).await;

            if let Err(e) = tool_output {
                return Err(anyhow::anyhow!("Error executing tool: {}", e));
            }

            let tool_output = tool_output.unwrap();
            let tool_output_str = serde_json::to_string(&tool_output).unwrap();

            tool_outputs.push(tool_output_str);
        }
    }

    Ok(tool_outputs)
}

pub fn setup_tracing() {
    let crate_name = env!("CARGO_CRATE_NAME");
    let crate_version = env!("CARGO_PKG_VERSION");

    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!("RUST_LOG=info,{}=info,domain=info,models=info,tokio=trace,runtime=trace,actix_web=info", crate_name).into()
    });

    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(filter_layer);
    let subscriber = tracing_subscriber::registry().with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("[VOO] {} v{}\n", crate_name, crate_version);
}
