# Security Considerations for FileJack

## Overview
This document outlines the security measures implemented in FileJack to ensure safe file operations.

## Security Features

### 1. Path Traversal Prevention
- **Canonical Path Resolution**: All file paths are canonicalized using `Path::canonicalize()` to resolve symlinks and `..` sequences
- **Base Path Enforcement**: When configured with a base path, all operations are restricted to that directory tree
- **Validation Before Operations**: All file paths are validated before any read or write operations

### 2. Error Information Disclosure
- **Sanitized Error Messages**: Error messages provide useful information without exposing sensitive system details
- **Specific Error Types**: Different error types (FileNotFound, PermissionDenied, etc.) allow proper handling without information leakage

### 3. Input Validation
- **JSON-RPC Validation**: All incoming requests are validated against the JSON-RPC 2.0 specification
- **Parameter Validation**: Tool parameters are validated using serde for type safety
- **Path Validation**: File paths are validated for existence and permissions before operations

### 4. Memory Safety
- **Rust's Memory Safety**: Leverages Rust's ownership system to prevent buffer overflows and memory corruption
- **No Unsafe Code**: The entire codebase contains zero `unsafe` blocks
- **Bounds Checking**: All array and vector accesses are bounds-checked by Rust

### 5. Dependency Security
- **Minimal Dependencies**: Uses only well-maintained, popular crates:
  - `serde` & `serde_json`: Industry-standard serialization
  - `tokio`: Well-audited async runtime
  - `anyhow` & `thiserror`: Error handling libraries
  - `tempfile`: For testing only (dev-dependency)

## Security Best Practices

### When Deploying FileJack

1. **Always Set Base Path**: Use `FILEJACK_BASE_PATH` environment variable to restrict file access
   ```bash
   FILEJACK_BASE_PATH=/path/to/safe/directory ./filejack
   ```

2. **Run with Least Privilege**: Run the server with minimal file system permissions
   ```bash
   # Create a dedicated user
   sudo useradd -r -s /bin/false filejack
   
   # Run as that user
   sudo -u filejack FILEJACK_BASE_PATH=/data ./filejack
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
