use std::sync::Arc;

use domain::models::{agent::Agent, tools::Tool};
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
    let tools = vec![read_file_tool];

    let agent = Agent::new(gemini, tools.clone());
    let crate_name = env!("CARGO_PKG_NAME").to_uppercase();
    let crate_version = env!("CARGO_PKG_VERSION");

    println!("Chat with VOO (use 'ctrl-c' to quit)\n");

    for tool in tools.iter() {
        let tool_info = format!(
            "I want you to register this tool into your system. Always follow the input schema!\nAlways include your response using the input schema provided and nothing else. If a user ask about the tool, just respond normally.\n{}",
            tool.as_ref()
        );

        println!("{}", tool_info);

        let response = agent.client().ask(&tool_info).await;

        match response {
            Ok(responses) => {
                for response in responses {
                    println!("{}", response);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

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
                    let is_tool_use = {
                        let mut tool_to_use: Option<Arc<dyn Tool>> = None;

                        for tool in tools.iter() {
                            if response.contains(tool.name()) {
                                tool_to_use = Some(tool.clone());
                                break;
                            }
                        }

                        tool_to_use
                    };

                    println!(
                        "\x1b[34m{}@{}: \x1b[0m{}",
                        crate_name, crate_version, response
                    );

                    if let Some(tool) = is_tool_use {
                        let tool_name = tool.name();
                        let tool_description = tool.description();
                        println!(
                            "Tool to use: {}\nDescription: {}\n",
                            tool_name, tool_description
                        );

                        if let Some(start) = response.find("```json\n") {
                            if let Some(end) = response[start..].find("\n```") {
                                let json_str = &response[start + "```json\n".len()..start + end];
                                let tool_use_input = format!("{}({})", tool.name(), json_str);

                                println!(
                                    "\x1b[33m{}@{}: \x1b[0m{}",
                                    crate_name, crate_version, tool_use_input
                                );

                                let response = tool.exec(json_str.to_string()).await;

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
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
