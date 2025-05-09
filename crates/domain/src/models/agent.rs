use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    io::Write,
    sync::Arc,
};

use async_trait::async_trait;
use tokio::sync::Mutex;

use super::tools::Tool;

#[async_trait]
pub trait AgentClient: Debug + Send + Sync + 'static {
    async fn ask(&self, prompt: &str) -> Result<Vec<String>, AgentError>;
    async fn add_tool(&self, tool: Arc<dyn Tool>) -> Result<(), AgentError>;
    async fn add_system_prompt(&self, prompt: &str) -> Result<(), AgentError>;
}

pub trait InputReader: Debug + Send + Sync + 'static {
    fn read(&self) -> Result<String, AgentError>;
}

#[derive(Debug)]
pub struct Agent {
    reader: Arc<dyn InputReader>,
    client: Arc<dyn AgentClient>,
    tools: Arc<Mutex<HashMap<String, Arc<dyn Tool>>>>,
}

impl Agent {
    pub fn new(client: impl AgentClient + 'static) -> Self {
        Self {
            client: Arc::new(client),
            reader: Arc::new(TerminalInputReader),
            tools: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_tool(&self, tool: Arc<dyn Tool>) -> Result<(), AgentError> {
        self.tools
            .try_lock()
            .unwrap()
            .insert(tool.name().to_string(), tool);

        Ok(())
    }
}

#[derive(Debug)]
pub enum AgentError {
    UserInputError(Option<String>),
    AgentError(Option<String>),
}

impl Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::UserInputError(msg) => match msg {
                Some(msg) => write!(f, "UserInputError: {}", msg),
                None => write!(f, "UserInputError: "),
            },
            AgentError::AgentError(msg) => match msg {
                Some(msg) => write!(f, "AgentError: {}", msg),
                None => write!(f, "AgentError: "),
            },
        }
    }
}

impl Agent {
    pub fn client(&self) -> &Arc<dyn AgentClient> {
        &self.client
    }

    pub fn reader(&self) -> &Arc<dyn InputReader> {
        &self.reader
    }

    pub fn tools(&self) -> &Arc<Mutex<HashMap<String, Arc<dyn Tool>>>> {
        &self.tools
    }
}

#[derive(Debug)]
pub struct TerminalInputReader;

impl InputReader for TerminalInputReader {
    fn read(&self) -> Result<String, AgentError> {
        let mut input = String::new();

        std::io::stdout()
            .write_all("\x1b[32mYOU: \x1b[0m".as_bytes())
            .map_err(|e| AgentError::UserInputError(Some(e.to_string())))?;

        std::io::stdout().flush().map_err(|e| {
            AgentError::UserInputError(Some(format!("Error flushing stdout: {}", e.to_string())))
        })?;

        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| AgentError::UserInputError(Some(e.to_string())))?;

        // let input = String::from_utf8_lossy(&input).to_string();
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockAgentClient {}

    #[derive(Debug)]
    struct MockInputReader;

    #[async_trait]
    impl AgentClient for MockAgentClient {
        async fn ask(&self, prompt: &str) -> Result<Vec<String>, AgentError> {
            Ok(vec![prompt.to_string()])
        }

        async fn add_tool(&self, _tool: Arc<dyn Tool>) -> Result<(), AgentError> {
            Ok(())
        }

        async fn add_system_prompt(&self, _prompt: &str) -> Result<(), AgentError> {
            Ok(())
        }
    }

    impl InputReader for MockInputReader {
        fn read(&self) -> Result<String, AgentError> {
            Ok("test input".to_string())
        }
    }

    #[test]
    fn test_get_user_input() {
        let reader = MockInputReader {};
        let agent = Agent {
            client: Arc::new(MockAgentClient {}),
            reader: Arc::new(reader),
            tools: Arc::new(Mutex::new(HashMap::new())),
        };

        let input = "test input";
        let result = agent.reader().read();
        assert_eq!(result.unwrap(), input);
    }
}
