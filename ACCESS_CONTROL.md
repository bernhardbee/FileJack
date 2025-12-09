# FileJack Access Control

FileJack includes a comprehensive access control system to ensure secure and limited filesystem access. This prevents misuse and ensures processes only access authorized parts of the filesystem.

## Features

### Access Control Policies

FileJack supports fine-grained access control through configurable policies:

1. **Path-based Access Control**
   - Whitelist allowed directories
   - Blacklist denied paths (takes precedence)
   - Prevents path traversal attacks

2. **File Extension Filtering**
   - Whitelist allowed file extensions
   - Blacklist dangerous extensions (.exe, .sh, etc.)

3. **File Size Limits**
   - Configurable maximum file size
   - Prevents resource exhaustion

4. **Security Features**
   - Symbolic link control
   - Hidden file access control
   - Read-only mode

## Configuration

### Using a Configuration File

Create a `filejack.json` configuration file:

```json
{
  "access_policy": {
    "allowed_paths": [
      "/home/user/documents",
      "/home/user/projects"
    ],
    "denied_paths": [
      "/home/user/documents/private"
    ],
    "allowed_extensions": [
      "txt", "md", "json", "xml", "csv", "log"
    ],
    "denied_extensions": [
      "exe", "sh", "bat", "cmd"
    ],
    "max_file_size": 10485760,
    "allow_symlinks": false,
    "allow_hidden_files": false,
    "read_only": false
  },
  "server": {
    "name": "FileJack",
    "version": "0.1.0"
  }
}
```

### Configuration Options

#### Access Policy

- **allowed_paths** (array of paths): List of directories that can be accessed. Empty array means all paths are allowed (unless denied).
- **denied_paths** (array of paths): Paths that are explicitly forbidden. Takes precedence over allowed_paths.
- **allowed_extensions** (array of strings): File extensions that are permitted. Empty means all extensions allowed.
- **denied_extensions** (array of strings): File extensions that are forbidden. Takes precedence over allowed_extensions.
- **max_file_size** (number): Maximum file size in bytes. 0 means no limit.
- **allow_symlinks** (boolean): Whether symbolic links can be followed.
- **allow_hidden_files** (boolean): Whether hidden files (starting with `.`) can be accessed.
- **read_only** (boolean): If true, all write operations are disabled.

#### Server Configuration

- **name** (string): Server name (default: "FileJack")
- **version** (string): Server version (default: package version)

### Environment Variables

If no configuration file is provided, FileJack can be configured via environment variables:

- `FILEJACK_CONFIG`: Path to configuration file
- `FILEJACK_BASE_PATH`: Base directory for file operations
- `FILEJACK_READ_ONLY`: Set to "true" for read-only mode

**Example:**
```bash
export FILEJACK_BASE_PATH="/home/user/workspace"
export FILEJACK_READ_ONLY="false"
./filejack
```

### Configuration File Loading

FileJack looks for configuration files in this order:

1. Path specified in `FILEJACK_CONFIG` environment variable
2. `filejack.json` in current directory
3. Falls back to environment-based configuration

## Usage Examples

### Example 1: Restricted Access

Allow access only to a specific project directory:

```json
{
  "access_policy": {
    "allowed_paths": ["/home/user/myproject"],
    "denied_paths": [],
    "allowed_extensions": [],
    "denied_extensions": ["exe", "sh"],
    "max_file_size": 5242880,
    "allow_symlinks": false,
    "allow_hidden_files": false,
    "read_only": false
  }
}
```

### Example 2: Read-Only Mode

Provide read-only access to documentation:

```json
{
  "access_policy": {
    "allowed_paths": ["/usr/share/doc"],
    "denied_paths": [],
    "allowed_extensions": ["txt", "md", "html", "pdf"],
    "denied_extensions": [],
    "max_file_size": 10485760,
    "allow_symlinks": true,
    "allow_hidden_files": false,
    "read_only": true
  }
}
```

### Example 3: Multiple Allowed Directories

Access multiple project directories with restrictions:

```json
{
  "access_policy": {
    "allowed_paths": [
      "/home/user/project1",
      "/home/user/project2",
      "/var/data/shared"
    ],
    "denied_paths": [
      "/home/user/project1/secrets"
    ],
    "allowed_extensions": ["txt", "md", "json", "yaml"],
    "denied_extensions": [],
    "max_file_size": 2097152,
    "allow_symlinks": false,
    "allow_hidden_files": false,
    "read_only": false
  }
}
```

### Example 4: Permissive Mode (Development)

For development/testing (use with caution):

```json
{
  "access_policy": {
    "allowed_paths": [],
    "denied_paths": ["/etc", "/sys", "/proc"],
    "allowed_extensions": [],
    "denied_extensions": ["exe", "dll", "so"],
    "max_file_size": 0,
    "allow_symlinks": true,
    "allow_hidden_files": true,
    "read_only": false
  }
}
```

## Security Best Practices

1. **Principle of Least Privilege**: Only grant access to directories that are absolutely necessary
2. **Deny Dangerous Extensions**: Always block executable extensions in production
3. **Use Read-Only Mode**: When write access isn't needed
4. **Set File Size Limits**: Prevent resource exhaustion
5. **Disable Symlinks**: Unless explicitly required
6. **Block Hidden Files**: Unless needed for specific use cases
7. **Use Denied Paths**: Explicitly block sensitive directories
8. **Regular Audits**: Review and update access policies regularly

## Programmatic Usage

### Creating Access Policies in Code

```rust
use filejack::{AccessPolicy, Config, McpServer};
use std::path::PathBuf;

// Restricted policy
let policy = AccessPolicy::restricted(PathBuf::from("/home/user/workspace"));

// Read-only policy
let policy = AccessPolicy::read_only(PathBuf::from("/usr/share/doc"));

// Custom policy
let mut policy = AccessPolicy::default();
policy.allowed_paths = vec![PathBuf::from("/home/user/data")];
policy.allowed_extensions = vec!["txt".to_string(), "json".to_string()];
policy.max_file_size = 1024 * 1024; // 1MB

// Create server with policy
let server = McpServer::with_policy(policy);
```

### Loading Configuration

```rust
use filejack::Config;

// Load from file
let config = Config::from_file("filejack.json")?;
let server = McpServer::with_policy(config.access_policy);

// Create and save configuration
let config = Config::default_restricted(PathBuf::from("/workspace"));
config.to_file("filejack.json")?;
```

## Error Handling

FileJack will return permission denied errors when:

- Attempting to access paths outside allowed directories
- Trying to read/write files with forbidden extensions
- Accessing denied paths
- Exceeding file size limits
- Accessing hidden files when not allowed
- Following symlinks when not permitted
- Writing files in read-only mode

These errors are returned as JSON-RPC error responses with appropriate error codes and messages.

## Testing Access Control

Use the included examples to test access control:

```bash
# Create a test configuration
cat > test_config.json << EOF
{
  "access_policy": {
    "allowed_paths": ["./test_data"],
    "allowed_extensions": ["txt"],
    "max_file_size": 1024,
    "read_only": false
  }
}
EOF

# Run with configuration
FILEJACK_CONFIG=test_config.json cargo run
```

## Migration Guide

If you're using the legacy base_path configuration:

**Before:**
```bash
FILEJACK_BASE_PATH="/workspace" ./filejack
```

**After (Environment):**
```bash
FILEJACK_BASE_PATH="/workspace" ./filejack
```

**After (Config File):**
```json
{
  "access_policy": {
    "allowed_paths": ["/workspace"]
  }
}
```

The legacy environment variable is still supported, but the new configuration system provides much more control and flexibility.
