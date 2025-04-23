use std::sync::Arc;

use domain::models::{
    agent::Agent,
    tools::{Tool, ToolNameInput},
};
use models::{models::gemini::GeminiModel, tools::read_file::ReadFileTool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    let gemini = GeminiModel::new(api_key);
    let read_file_tool = ReadFileTool::new(
        "read_file",
        "Read the contents of a given relative file path. Use this when you want to see what's inside a file. Do not use this with directory names.",
    );

    let read_file_tool: Arc<dyn Tool> = Arc::new(read_file_tool);
    let agent = Agent::new(gemini);
    agent
        .add_tool(read_file_tool)
        .map_err(|e| anyhow::anyhow!("Error adding tool: {}", e))?;

    let crate_name = env!("CARGO_PKG_NAME").to_uppercase();
    let crate_version = env!("CARGO_PKG_VERSION");

    println!("Chat with VOO (use 'ctrl-c' to quit)\n");

    loop {
        let input = agent
            .reader()
            .read()
            .map_err(|e| anyhow::anyhow!("Error reading input: {}", e))?;

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

                        if response.starts_with("```json") {
                            if let Err(e) = tool_name_input {
                                println!(
                                    "\x1b[31m{}@{}: \x1b[0m{}",
                                    crate_name, crate_version, response
                                );

                                let error_message =
                                    format!("Error processing response: {}, prompt: {}", e, input);
                                _ = agent.client().add_system_prompt(&error_message).await;
                                continue;
                            }
                        }

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
                        let tool_description = tool.description();

                        println!(
                            "Tool to use: {}\nDescription: {}\n",
                            tool_name, tool_description
                        );

                        let tool_use_input = format!("{}({})", tool.name(), input);

                        println!(
                            "\x1b[33m{}@{}: \x1b[0m{}",
                            crate_name, crate_version, tool_use_input
                        );

                        let response = tool.exec(input).await;

                        if let Err(e) = &response {
                            eprintln!("Error executing tool: {:?}", e);
                        }

                        if let Ok(response) = response {
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
                        } else {
                            println!("Error executing tool");
                        }
                    }
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
