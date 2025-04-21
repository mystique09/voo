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

    println!("Welcome to {} {}!\n", crate_name, crate_version);

    loop {
        let input = agent
            .reader()
            .read()
            .map_err(|e| anyhow::anyhow!("Error reading input: {}", e))?;

        let response = agent.client().ask(&input).await;

        match response {
            Ok(response) => {
                if response.starts_with("exit") {
                    println!("Bye!");
                    break;
                }

                println!(
                    "\x1b[34m{}:{}> \x1b[0m{}",
                    crate_name, crate_version, response
                )
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
