# FileJack

A high-performance Model Context Protocol (MCP) server for file reading and writing operations, written in Rust.

## Overview

FileJack is an MCP server that provides secure file I/O capabilities through a JSON-RPC 2.0 interface. It enables clients to read from and write to files with optional path restrictions for enhanced security.

## Features

- ✅ **MCP Protocol Compliant**: Implements the Model Context Protocol specification
- ✅ **JSON-RPC 2.0**: Standard JSON-RPC interface for communication
- ✅ **Advanced Access Control**: Fine-grained filesystem access control with configurable policies
- ✅ **Secure Operations**: Path-based restrictions, extension filtering, and file size limits
- ✅ **Configuration File Support**: JSON-based configuration for access policies
- ✅ **Comprehensive Testing**: 72 tests (64 unit + 8 integration)
- ✅ **Error Handling**: Detailed error reporting with helpful messages and debugging logs
- ✅ **Auto-create Directories**: Automatically creates parent directories when writing files
- ✅ **UTF-8 Support**: Full Unicode support including emojis
- ✅ **Read-Only Mode**: Support for read-only filesystem access
- ✅ **Extension Whitelisting/Blacklisting**: Control which file types can be accessed
- ✅ **Symlink Control**: Configure whether symbolic links can be followed
- ✅ **Hidden File Control**: Configure access to hidden files

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

#### With Configuration File

Create a `filejack.json` configuration file (see [ACCESS_CONTROL.md](ACCESS_CONTROL.md) for details):

```json
{
  "access_policy": {
    "allowed_paths": ["/home/user/workspace"],
    "denied_paths": [],
    "allowed_extensions": ["txt", "md", "json"],
    "denied_extensions": ["exe", "sh"],
    "max_file_size": 10485760,
    "allow_symlinks": false,
    "allow_hidden_files": false,
    "read_only": false
  }
}
```

Then run:

```bash
./target/release/filejack
# Or specify a custom config path
FILEJACK_CONFIG=/path/to/config.json ./target/release/filejack
```

#### With Environment Variables

Set environment variables for basic configuration:

```bash
# Restrict to specific directory
FILEJACK_BASE_PATH=/path/to/allowed/directory ./target/release/filejack

# Enable read-only mode
FILEJACK_BASE_PATH=/path/to/directory FILEJACK_READ_ONLY=true ./target/release/filejack
```

### Access Control

FileJack includes comprehensive access control to prevent misuse. See [ACCESS_CONTROL.md](ACCESS_CONTROL.md) for detailed documentation on:

- Path-based access control (whitelist/blacklist)
- File extension filtering
- File size limits
- Symbolic link control
- Hidden file access control
- Read-only mode
- Configuration examples and best practices

### Available Tools

#### 1. read_file

Read contents from a file.

**Parameters:**
- `path` (string, required) - Path to the file to read

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

**Parameters:**
- `path` (string, required) - Path to the file to write
- `content` (string, required) - Content to write to the file

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
```
FileJack/
├── src/
│   ├── lib.rs              # Library exports
│   ├── main.rs             # Binary entry point
│   ├── error.rs            # Error types and handling
│   ├── protocol.rs         # JSON-RPC and MCP protocol structures
│   ├── file_ops.rs         # File reader and writer implementations
│   ├── mcp.rs              # MCP server implementation
│   ├── access_control.rs   # Access control policies
│   └── config.rs           # Configuration file handling
├── tests/
│   └── integration_tests.rs  # Integration tests
├── ACCESS_CONTROL.md       # Access control documentation
├── filejack.example.json   # Example configuration file
### Core Components

1. **FileReader**: Handles reading operations with policy-based access control
2. **FileWriter**: Handles writing operations with policy-based access control
3. **AccessPolicy**: Configurable access control policies for filesystem operations
4. **Config**: Configuration file loading and management
5. **McpServer**: Orchestrates MCP protocol handling and tool dispatch
6. **Protocol Structures**: JSON-RPC and MCP type definitions
7. **Error System**: Comprehensive error types with proper conversions
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
## Security

FileJack implements multiple layers of security to prevent filesystem misuse:

### Access Control
- **Path Whitelisting**: Explicitly define allowed directories
- **Path Blacklisting**: Deny access to sensitive paths
- **Extension Filtering**: Control which file types can be accessed
- **File Size Limits**: Prevent resource exhaustion
- **Symlink Protection**: Prevent symlink-based attacks
- **Hidden File Control**: Control access to hidden files
- **Read-Only Mode**: Prevent any write operations

### Path Validation
- Canonical path resolution to prevent traversal attacks
- Strict boundary checking for allowed directories
- Denied paths take precedence over allowed paths

### Configuration Security
- All security policies defined in configuration file
- Environment variable fallback for basic restrictions
- Secure defaults (restrictive by default)

See [ACCESS_CONTROL.md](ACCESS_CONTROL.md) for comprehensive security documentation and best practices.tion
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
| -32602 | Invalid parameters | Missing or invalid tool parameters |
| -32000 | Server error | File operation failed (see error message) |

### Error Messages

FileJack provides detailed error messages for debugging:

- **Missing parameters**: Clearly states which parameter is missing and what format is expected
  - Example: `"Invalid parameters for read_file: missing field 'path'. Expected: {\"path\": \"string\"}"`
  
- **Permission denied**: Indicates which access control rule was violated
  - Example: `"Path /etc/passwd is not in any allowed directory"`
  
- **File not found**: Shows the attempted path
  - Example: `"File not found: /tmp/nonexistent.txt"`

All errors are logged to stderr for debugging MCP integration issues.

## VS Code Integration

FileJack can be used as an MCP server in VS Code. Create a `.vscode/mcp.json` file in your workspace:

```json
{
  "servers": {
    "filejack": {
      "command": "/path/to/filejack",
      "args": [],
      "env": {
        "FILEJACK_CONFIG": "/path/to/filejack.json"
      }
    }
  }
}
```

Or use environment variables for simple configuration:

```json
{
  "servers": {
    "filejack": {
      "command": "/path/to/filejack",
      "args": [],
      "env": {
        "FILEJACK_BASE_PATH": "/workspace",
        "FILEJACK_READ_ONLY": "false"
      }
    }
  }
}
```

After configuration, restart VS Code. FileJack tools will be available to AI assistants in your workspace.

See [ACCESS_CONTROL.md](ACCESS_CONTROL.md) for advanced configuration options.

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
