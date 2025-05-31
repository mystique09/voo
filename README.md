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

YOU: Hi!

voo> Hi there! How can I help you today?
```

## Contributing

Contributions are welcome! Please submit a pull request with your changes.