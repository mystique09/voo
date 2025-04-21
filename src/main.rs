use domain::models::agent::Agent;
use models::models::gemini::GeminiModel;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    let gemini = GeminiModel::new(api_key);
    let agent = Agent::new(gemini);
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
                    println!(
                        "\x1b[34m{}:{}> \x1b[0m{}",
                        crate_name, crate_version, response
                    );
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
