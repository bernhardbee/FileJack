# FileJack

A high-performance Model Context Protocol (MCP) server for file reading and writing operations, written in Rust.

## Overview

FileJack is an MCP server that provides secure file I/O capabilities through a JSON-RPC 2.0 interface. It enables clients to read from and write to files with optional path restrictions for enhanced security.

## Features

- ✅ **MCP Protocol Compliant**: Implements the Model Context Protocol specification
- ✅ **JSON-RPC 2.0**: Standard JSON-RPC interface for communication
- ✅ **Secure Operations**: Optional base path restriction to limit file access
- ✅ **Comprehensive Testing**: 40+ unit tests and 8 integration tests
- ✅ **Error Handling**: Detailed error reporting with proper error codes
- ✅ **Auto-create Directories**: Automatically creates parent directories when writing files
- ✅ **UTF-8 Support**: Full Unicode support including emojis

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building from Source

```bash
git clone https://github.com/bernhardbee/FileJack.git
cd FileJack
cargo build --release
```

The compiled binary will be available at `target/release/filejack`.

## Usage

### Running the Server

Start the server and it will listen for JSON-RPC requests on stdin:

```bash
./target/release/filejack
```

#### With Base Path Restriction

Set the `FILEJACK_BASE_PATH` environment variable to restrict file operations to a specific directory:

```bash
FILEJACK_BASE_PATH=/path/to/allowed/directory ./target/release/filejack
```

### Available Tools

#### 1. read_file

Read contents from a file.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "read_file",
    "arguments": {
      "path": "/path/to/file.txt"
    }
  },
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": "File contents here...",
    "path": "/path/to/file.txt"
  },
  "id": 1
}
```

#### 2. write_file

Write contents to a file (creates parent directories if needed).

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "write_file",
    "arguments": {
      "path": "/path/to/file.txt",
      "content": "Content to write"
    }
  },
  "id": 2
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "success": true,
    "path": "/path/to/file.txt",
    "bytes_written": 16
  },
  "id": 2
}
```

### MCP Methods

#### initialize

Initialize the MCP server connection.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "1.0",
    "serverInfo": {
      "name": "FileJack",
      "version": "0.1.0"
    },
    "capabilities": {
      "tools": {}
    }
  },
  "id": 1
}
```

#### tools/list

List all available tools.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "id": 2
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "name": "read_file",
        "description": "Read contents from a file",
        "input_schema": {
          "type": "object",
          "properties": {
            "path": {
              "type": "string",
              "description": "Path to the file to read"
            }
          },
          "required": ["path"]
        }
      },
      {
        "name": "write_file",
        "description": "Write contents to a file",
        "input_schema": {
          "type": "object",
          "properties": {
            "path": {
              "type": "string",
              "description": "Path to the file to write"
            },
            "content": {
              "type": "string",
              "description": "Content to write to the file"
            }
          },
          "required": ["path", "content"]
        }
      }
    ]
  },
  "id": 2
}
```

## Architecture

### Project Structure

```
FileJack/
├── src/
│   ├── lib.rs           # Library exports
│   ├── main.rs          # Binary entry point
│   ├── error.rs         # Error types and handling
│   ├── protocol.rs      # JSON-RPC and MCP protocol structures
│   ├── file_ops.rs      # File reader and writer implementations
│   └── mcp.rs           # MCP server implementation
├── tests/
│   └── integration_tests.rs  # Integration tests
├── Cargo.toml           # Project configuration
└── README.md            # This file
```

### Core Components

1. **FileReader**: Handles reading operations with path validation
2. **FileWriter**: Handles writing operations with auto-directory creation
3. **McpServer**: Orchestrates MCP protocol handling and tool dispatch
4. **Protocol Structures**: JSON-RPC and MCP type definitions
5. **Error System**: Comprehensive error types with proper conversions

## Testing

### Run All Tests

```bash
cargo test
```

### Run Unit Tests Only

```bash
cargo test --lib
```

### Run Integration Tests Only

```bash
cargo test --test integration_tests
```

### Test Coverage

The project includes:
- **40 unit tests** covering all core functionality
- **8 integration tests** for end-to-end workflows
- Tests for error conditions and edge cases
- Tests for special characters, large files, and nested directories

## Error Codes

FileJack uses standard JSON-RPC 2.0 error codes:

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON received |
| -32600 | Invalid request | JSON-RPC request is invalid |
| -32601 | Method not found | Requested method doesn't exist |
| -32000 | Server error | File operation failed (see error message) |

## Security

- **Path Validation**: All file paths are validated before operations
- **Base Path Restriction**: Optional directory boundary enforcement
- **No Symlink Following**: Canonical paths are used to prevent traversal attacks
- **Error Sanitization**: Error messages don't expose system internals

## Performance

- Written in Rust for maximum performance
- Zero-copy operations where possible
- Efficient JSON parsing with serde_json
- Async-ready architecture (tokio support included)

## Development

### Building

```bash
cargo build
```

### Running in Development Mode

```bash
cargo run
```

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Authors

FileJack Contributors
