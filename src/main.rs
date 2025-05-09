use std::{sync::Arc, time::Duration};

use domain::models::{
    agent::Agent,
    tools::{Tool, ToolNameInput},
};
use models::{
    models::gemini::GeminiModel,
    tools::{list_files::ListFileTool, read_file::ReadFileTool},
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

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
        .map_err(|e| anyhow::anyhow!("Error adding tool: {}", e))?;
    agent
        .add_tool(list_file_tool)
        .map_err(|e| anyhow::anyhow!("Error adding tool: {}", e))?;

    let crate_name = env!("CARGO_PKG_NAME").to_uppercase();
    let crate_version = env!("CARGO_PKG_VERSION");

    println!("Chat with VOO (use 'ctrl-c' to quit)\n");
    let mut is_retry = false;
    let max_retry = 3;
    let mut retry_attempt = 0;

    'main: loop {
        if retry_attempt >= max_retry {
            is_retry = false;
            retry_attempt = 0;
        }

        let input = if !is_retry {
            let input = agent
                .reader()
                .read()
                .map_err(|e| anyhow::anyhow!("Error reading input: {}", e))?;

            input
        } else {
            retry_attempt += 1;
            "Retry.".to_string()
        };

        if input.starts_with("exit") {
            println!("Bye!");
            break;
        }

        let response = agent.client().ask(&input).await;

        match response {
            Ok(responses) => {
                for response in responses {
                    let (is_tool_use, input) = {
                        let mut tool: Option<Arc<dyn Tool>> = None;
                        let mut input_json: Option<String> = None;

                        let json_str = response.replace("```json", "").replace("```", "");
                        let tool_name_input = serde_json::from_str::<ToolNameInput>(&json_str);

                        if let Ok(tool_input) = tool_name_input {
                            let tools = agent.tools().try_lock().unwrap();
                            let get_tool = tools.get(&tool_input.name);

                            if let Some(tool_found) = get_tool {
                                tool = Some(tool_found.clone());
                                input_json = Some(json_str);
                            }
                        }

                        (tool, input_json)
                    };

                    println!(
                        "\x1b[34m{}@{}: \x1b[0m{}",
                        crate_name, crate_version, response
                    );

                    if let (Some(tool), Some(input)) = (is_tool_use, input) {
                        let tool_name = tool.name();

                        let response = loop {
                            println!(
                                "\x1b[33m{}@{}: \x1b[0mExecuting tool: {}(..)\n",
                                crate_name, crate_version, tool_name,
                            );

                            let tool_use_input = format!("{}({})", tool.name(), input);
                            println!("\x1b[33mtool: \x1b[0m{}", tool_use_input);

                            match tool.exec(input.clone()).await {
                                Ok(response) => {
                                    is_retry = false;
                                    break response;
                                }
                                Err(e) => {
                                    let error = format!(
                                        r#"
                                    Error executing tool {}; error={}
                                    "#,
                                        tool_name, e
                                    );
                                    _ = agent.client().add_system_prompt(&error);
                                    println!(
                                        "\x1b[33m{}@{}: \x1b[0mError executing tool {}; error={}\n",
                                        crate_name, crate_version, tool_name, e
                                    );
                                    sleep(Duration::from_secs(1)).await;
                                    println!(
                                        "\x1b[33m{}@{}: \x1b[0mRetrying {}(..)...\n",
                                        crate_name, crate_version, tool_name
                                    );

                                    is_retry = true;

                                    if retry_attempt >= max_retry {
                                        println!(
                                            "\x1b[33m{}@{}: \x1b[0mMax retry attempts reached. Exiting...\n",
                                            crate_name, crate_version,
                                        );
                                        continue 'main;
                                    }

                                    continue 'main;
                                }
                            }
                        };

                        let tool_use_response = agent.client().ask(&response).await;

                        if let Err(e) = &tool_use_response {
                            eprintln!("Error executing tool: {:?}", e);
                        }

                        if let Ok(tool_use_responses) = tool_use_response {
                            for tool_use_response in tool_use_responses {
                                println!(
                                    "\x1b[32m{}@{}: \x1b[0m{}",
                                    crate_name, crate_version, tool_use_response
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
