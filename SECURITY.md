# Security Considerations for FileJack

## Overview
This document outlines the comprehensive security measures implemented in FileJack to ensure safe file operations and prevent misuse.

## Security Features

### 1. Access Control System
- **Path Whitelisting**: Explicitly define allowed directories through configuration
- **Path Blacklisting**: Deny access to sensitive paths, overriding whitelist
- **Extension Filtering**: Control which file types can be accessed (whitelist and blacklist)
- **File Size Limits**: Prevent resource exhaustion attacks by limiting file sizes
- **Read-Only Mode**: Complete write operation prevention when enabled
- **Symlink Control**: Configure whether symbolic links can be followed
- **Hidden File Control**: Restrict access to hidden files (starting with `.`)

### 2. Path Traversal Prevention
- **Canonical Path Resolution**: All file paths are canonicalized using `Path::canonicalize()` to resolve symlinks and `..` sequences
- **Base Path Enforcement**: When configured with a base path, all operations are restricted to that directory tree
- **Validation Before Operations**: All file paths are validated before any read or write operations
- **Denied Paths Precedence**: Denied paths take precedence over allowed paths to prevent bypass

### 3. Configuration Security
- **Secure Defaults**: Restrictive defaults (no symlinks, no hidden files, limited file size)
- **Configuration File Support**: JSON-based configuration for explicit security policies
- **Environment Variable Fallback**: Support for basic restrictions via environment variables
- **Validation on Load**: Configuration is validated when loaded from file

### 4. Error Information Disclosure
- **Helpful Error Messages**: Error messages are detailed enough for debugging while avoiding sensitive data exposure
- **Parameter Validation Errors**: Clear messages indicate which parameters are missing or invalid with expected format
  - Example: `"Invalid parameters for read_file: missing field 'path'. Expected: {\"path\": \"string\"}"`
- **Access Control Errors**: Specific messages about which policy rule was violated without exposing full system paths
- **Debugging Support**: All errors are logged to stderr for troubleshooting MCP integration issues
- **No Stack Traces in Production**: Error responses don't include internal stack traces

### 5. Input Validation
- **JSON-RPC Validation**: All incoming requests are validated against the JSON-RPC 2.0 specification
- **Parameter Validation**: Tool parameters are validated using serde for type safety
- **Path Validation**: File paths are validated for existence, permissions, and policy compliance
- **Extension Validation**: File extensions are checked against allowed/denied lists

### 6. Memory Safety
- **Rust's Memory Safety**: Leverages Rust's ownership system to prevent buffer overflows and memory corruption
- **No Unsafe Code**: The entire codebase contains zero `unsafe` blocks
- **Bounds Checking**: All array and vector accesses are bounds-checked by Rust

### 7. Dependency Security
- **Minimal Dependencies**: Uses only well-maintained, popular crates:
  - `serde` & `serde_json`: Industry-standard serialization
  - `tokio`: Well-audited async runtime
  - `anyhow` & `thiserror`: Error handling libraries
  - `tempfile`: For testing only (dev-dependency)

## Security Best Practices

### When Deploying FileJack

1. **Always Use Configuration File**: Define explicit access policies
   ```bash
   # Create filejack.json with restrictive policy
   FILEJACK_CONFIG=filejack.json ./filejack
   ```

2. **Use Path Whitelisting**: Only allow access to necessary directories
   ```json
   {
     "access_policy": {
       "allowed_paths": ["/data/workspace"],
       "denied_paths": [],
       "allowed_extensions": ["txt", "md", "json"],
       "denied_extensions": ["exe", "sh", "bat"],
       "max_file_size": 10485760,
       "allow_symlinks": false,
       "allow_hidden_files": false,
       "read_only": false
     }
   }
   ```

3. **Deny Dangerous Extensions**: Always block executables
   ```json
   {
     "access_policy": {
       "denied_extensions": ["exe", "dll", "so", "sh", "bat", "cmd", "ps1"]
     }
   }
   ```

4. **Set File Size Limits**: Prevent resource exhaustion
   ```json
   {
     "access_policy": {
       "max_file_size": 10485760  // 10MB
     }
   }
   ```

5. **Run with Least Privilege**: Run the server with minimal file system permissions
   ```bash
   # Create a dedicated user
   sudo useradd -r -s /bin/false filejack
   
   # Set up workspace with proper permissions
   sudo mkdir -p /data/workspace
   sudo chown filejack:filejack /data/workspace
   
   # Run as that user
   sudo -u filejack FILEJACK_CONFIG=/etc/filejack.json ./filejack
   ```

6. **Use Read-Only Mode**: When write access isn't needed
   ```json
   {
     "access_policy": {
       "read_only": true
     }
   }
   ```

7. **Disable Symlinks**: Unless explicitly required
   ```json
   {
     "access_policy": {
       "allow_symlinks": false
     }
   }
   ```

8. **Block Hidden Files**: Prevent access to configuration files
   ```json
   {
     "access_policy": {
       "allow_hidden_files": false
     }
   }
   ```

3. **Network Isolation**: If exposed over network, use secure tunnels (SSH, VPN) or encryption (TLS)

4. **Input Sanitization**: While FileJack validates paths, consider additional validation at the client level

5. **Audit Logging**: Monitor stderr output for security events and unexpected access patterns

## Known Limitations

1. **No Authentication**: FileJack does not implement authentication. This must be handled at the transport layer.

2. **No Authorization**: Beyond base path restriction, there is no per-user or per-file authorization.

3. **No Rate Limiting**: The server does not implement rate limiting. This should be handled by a proxy or wrapper.

4. **Single Base Path**: Only one base path can be configured. Multiple isolated directories require multiple server instances.

## Threat Model

### Protected Against
- ✅ Path traversal attacks (../ sequences)
- ✅ Symlink-based directory escape
- ✅ Buffer overflow and memory corruption
- ✅ Injection attacks (JSON-RPC validation)
- ✅ Information disclosure (sanitized errors)

### Not Protected Against (Requires Additional Layers)
- ❌ Denial of Service (no rate limiting)
- ❌ Authentication bypass (no authentication)
- ❌ Network eavesdropping (no encryption)
- ❌ Privilege escalation (OS-level concern)

## Security Contact

For security issues, please contact the repository maintainers privately before public disclosure.

## Security Updates

Keep dependencies updated regularly:
```bash
cargo update
cargo audit  # Requires cargo-audit: cargo install cargo-audit
```
