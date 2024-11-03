# Clixy

Clixy is a versatile CLI tool designed for managing file operations with advanced options. It is built using Rust, ensuring high performance and safety.

## Features

-   **File Verification**: Check the accessibility and integrity of files before performing operations.
-   **Copy Operations**: Copy files from a source to a destination with various options:
    -   Replace existing files.
    -   Copy only new files that do not exist in the destination.
    -   Update files in the destination if they are older than the source files.

## Installation

To install Clixy locally, clone the repository and run the following command:

```sh
cargo install --path .
```

Ensure the Cargo bin directory is in your PATH:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

## Usage

Because this project is still in development, please refer to the help command for usage and options.

```sh
clixy --help
```

## Testing

To run the tests, execute the following command:

```sh
cargo test
```

Note: The `tests` folder contains integration tests. The `test` folder (with no `s`) contains only random files to test commands requiring files like `copy`.