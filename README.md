# Voo
My own attempt on making an AI agent that can work in the terminal.

>**NOTE:** This is a work in progress.

Currently, the AI has two working function calls:
- **read_file** - Reads a file and returns the contents
- **list_files** - Lists all files in a given directory

## Configuration

The following environment variables can be used to configure the application, create a `.env` file in the root of the project and add the following variables:

*   `GEMINI_API_KEY`: The API key for the Gemini language model.
*   `RUST_LOG`: Configures the level of logging detail.

## Installation
You need to have Rust installed on your system to run this application.
To install the application, run the following command:
```bash
cargo install --path .
# or
cargo build --release
# or if you don't want to clone the repository
cargo install --git https://github.com/mystique09/voo
```

## Usage
To run the application, run the following command:
```nushell
voo

--- 

Chat with VOO (use 'ctrl-c' to quit)

YOU: search for main.rs file and summarize its content
list_files> {"path":"f:/Dev/voo"}
list_files> {"path":"f:/Dev/voo/src/"}
read_file> {"path":"f:/Dev/voo/src/main.rs"}
voo> main.rs Summary:

The `main.rs` file is the entry point of the VOO application, a chatbot powered by the Gemini language model.

Key functionalities:

1.  **Initialization**:
    *   Loads environment variables from a `.env` file using `dotenvy`.
    *   Sets up tracing for logging using `tracing` and `tracing_subscriber`.
    *   Retrieves the Gemini API key from the environment variables.
    *   Initializes the `GeminiModel` with the API key.
    *   Creates and configures `ReadFileTool` and `ListFileTool` for interacting with the file system.

2.  **Agent Setup**:
    *   Creates an `Agent` instance, injecting the `GeminiModel`.
    *   Adds the `ReadFileTool` and `ListFileTool` to the agent, enabling it to read files and list directory contents.

3.  **Chat Loop**:
    *   Enters a main loop that continuously reads user input.
    *   Sends the input to the Gemini model for a response.
    *   Handles function calls requested by the model using available tools.
    *   Prints the model's responses to the console.
    *   Handles potential errors such as expired API keys or tool execution failures.

4.  **Function Call Handling**:
    *   The `perform_function_call` function executes tools based on the function calls requested by the Gemini model.
    *   It retrieves the appropriate tool, executes it with the provided arguments, and returns the output.

5.  **Response Printing**:
    *   The `print_response` function formats and prints the responses from the Gemini model to the console, also adding the responses to the agent's system prompt.

6.  **Tracing Setup**:
    *   The `setup_tracing` function configures the tracing subscriber for logging application events.

In essence, the `main.rs` file orchestrates the initialization of the Gemini model, the setup of available tools, and the main chat loop for interacting with the user, providing a conversational interface to the VOO agent.
```

## Contributing

Contributions are welcome! Please submit a pull request with your changes.