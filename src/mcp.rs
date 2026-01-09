use crate::access_control::AccessPolicy;
use crate::error::{FileJackError, Result};
use crate::file_ops::{FileReader, FileWriter};
use crate::protocol::{
    JsonRpcRequest, JsonRpcResponse, McpTool, ReadFileParams, WriteFileParams,
    ListDirectoryParams, GetMetadataParams, DeleteFileParams, MoveFileParams, CopyFileParams,
};
use crate::rate_limit::RateLimiter;
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

/// MCP Server for file operations
pub struct McpServer {
    reader: FileReader,
    writer: FileWriter,
    rate_limiter: RateLimiter,
}

impl McpServer {
    /// Create a new MCP Server with an access policy
    pub fn new(policy: AccessPolicy) -> Self {
        Self {
            reader: FileReader::new(policy.clone()),
            writer: FileWriter::new(policy, true),
            rate_limiter: RateLimiter::moderate(),
        }
    }

    /// Create a new MCP Server with custom rate limiter
    pub fn with_rate_limiter(policy: AccessPolicy, rate_limiter: RateLimiter) -> Self {
        Self {
            reader: FileReader::new(policy.clone()),
            writer: FileWriter::new(policy, true),
            rate_limiter,
        }
    }

    /// Get the list of available tools
    pub fn list_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: "read_file".to_string(),
                description: "Read contents from a file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to read"
                        }
                    },
                    "required": ["path"]
                }),
            },
            McpTool {
                name: "write_file".to_string(),
                description: "Write contents to a file".to_string(),
                input_schema: json!({
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
                }),
            },
            McpTool {
                name: "list_directory".to_string(),
                description: "List contents of a directory".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the directory to list"
                        },
                        "recursive": {
                            "type": "boolean",
                            "description": "Whether to list recursively",
                            "default": false
                        }
                    },
                    "required": ["path"]
                }),
            },
            McpTool {
                name: "get_metadata".to_string(),
                description: "Get metadata information about a file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file"
                        }
                    },
                    "required": ["path"]
                }),
            },
            McpTool {
                name: "delete_file".to_string(),
                description: "Delete a file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to delete"
                        }
                    },
                    "required": ["path"]
                }),
            },
            McpTool {
                name: "move_file".to_string(),
                description: "Move or rename a file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "from": {
                            "type": "string",
                            "description": "Source file path"
                        },
                        "to": {
                            "type": "string",
                            "description": "Destination file path"
                        }
                    },
                    "required": ["from", "to"]
                }),
            },
            McpTool {
                name: "copy_file".to_string(),
                description: "Copy a file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "from": {
                            "type": "string",
                            "description": "Source file path"
                        },
                        "to": {
                            "type": "string",
                            "description": "Destination file path"
                        }
                    },
                    "required": ["from", "to"]
                }),
            },
        ]
    }

    /// Handle a tool call
    pub fn handle_tool_call(&self, name: &str, arguments: Value) -> Result<Value> {
        // Log the arguments received for debugging
        debug!(tool = name, "Tool called with arguments: {}", arguments);
        
        match name {
            "read_file" => {
                let params: ReadFileParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        error!("Failed to parse read_file params: {}", e);
                        FileJackError::InvalidParameters(
                            format!("Invalid parameters for read_file: {}. Expected: {{\"path\": \"string\"}}", e)
                        )
                    })?;
                
                info!(path = %params.path, "Reading file");
                let content = self.reader.read_to_string(&params.path)?;
                info!(path = %params.path, size = content.len(), "File read successfully");
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": content
                        }
                    ]
                }))
            }
            "write_file" => {
                let params: WriteFileParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        error!("Failed to parse write_file params: {}", e);
                        FileJackError::InvalidParameters(
                            format!("Invalid parameters for write_file: {}. Expected: {{\"path\": \"string\", \"content\": \"string\"}}", e)
                        )
                    })?;
                
                info!(path = %params.path, size = params.content.len(), "Writing file");
                self.writer.write_string(&params.path, &params.content)?;
                info!(path = %params.path, "File written successfully");
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Successfully wrote {} bytes to {}", params.content.len(), params.path)
                        }
                    ]
                }))
            }
            "list_directory" => {
                let params: ListDirectoryParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        error!("Failed to parse list_directory params: {}", e);
                        FileJackError::InvalidParameters(
                            format!("Invalid parameters for list_directory: {}. Expected: {{\"path\": \"string\", \"recursive\": boolean}}", e)
                        )
                    })?;
                
                info!(path = %params.path, recursive = params.recursive, "Listing directory");
                let entries = self.reader.list_directory(&params.path, params.recursive)?;
                info!(path = %params.path, count = entries.len(), "Directory listed successfully");
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&entries).unwrap()
                        }
                    ]
                }))
            }
            "get_metadata" => {
                let params: GetMetadataParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        error!("Failed to parse get_metadata params: {}", e);
                        FileJackError::InvalidParameters(
                            format!("Invalid parameters for get_metadata: {}. Expected: {{\"path\": \"string\"}}", e)
                        )
                    })?;
                
                info!(path = %params.path, "Getting metadata");
                let metadata = self.reader.get_metadata(&params.path)?;
                info!(path = %params.path, "Metadata retrieved successfully");
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&metadata).unwrap()
                        }
                    ]
                }))
            }
            "delete_file" => {
                let params: DeleteFileParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        error!("Failed to parse delete_file params: {}", e);
                        FileJackError::InvalidParameters(
                            format!("Invalid parameters for delete_file: {}. Expected: {{\"path\": \"string\"}}", e)
                        )
                    })?;
                
                info!(path = %params.path, "Deleting file");
                self.writer.delete_file(&params.path)?;
                info!(path = %params.path, "File deleted successfully");
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Successfully deleted {}", params.path)
                        }
                    ]
                }))
            }
            "move_file" => {
                let params: MoveFileParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        error!("Failed to parse move_file params: {}", e);
                        FileJackError::InvalidParameters(
                            format!("Invalid parameters for move_file: {}. Expected: {{\"from\": \"string\", \"to\": \"string\"}}", e)
                        )
                    })?;
                
                info!(from = %params.from, to = %params.to, "Moving file");
                self.writer.move_file(&params.from, &params.to)?;
                info!(from = %params.from, to = %params.to, "File moved successfully");
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Successfully moved {} to {}", params.from, params.to)
                        }
                    ]
                }))
            }
            "copy_file" => {
                let params: CopyFileParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        error!("Failed to parse copy_file params: {}", e);
                        FileJackError::InvalidParameters(
                            format!("Invalid parameters for copy_file: {}. Expected: {{\"from\": \"string\", \"to\": \"string\"}}", e)
                        )
                    })?;
                
                info!(from = %params.from, to = %params.to, "Copying file");
                let bytes_copied = self.writer.copy_file(&params.from, &params.to)?;
                info!(from = %params.from, to = %params.to, bytes = bytes_copied, "File copied successfully");
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Successfully copied {} to {} ({} bytes)", params.from, params.to, bytes_copied)
                        }
                    ]
                }))
            }
            _ => {
                warn!(tool = name, "Tool not found");
                Err(FileJackError::ToolNotFound(name.to_string()))
            }
        }
    }

    /// Handle a JSON-RPC request
    pub fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        debug!(method = %request.method, id = ?request.id, "Handling request");
        
        match request.method.as_str() {
            "tools/list" => {
                debug!("Listing available tools");
                let tools = self.list_tools();
                let tools_value = serde_json::to_value(&tools).unwrap();
                JsonRpcResponse::success(request.id, json!({"tools": tools_value}))
            }
            "tools/call" => {
                let params = request.params.unwrap_or(json!({}));
                
                debug!("tools/call received params: {}", params);
                
                let tool_name = params.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let arguments = params.get("arguments")
                    .cloned()
                    .unwrap_or(json!({}));
                
                debug!("Extracted tool_name: '{}', arguments: {}", tool_name, arguments);

                match self.handle_tool_call(tool_name, arguments) {
                    Ok(result) => {
                        info!(tool = tool_name, "Tool call successful");
                        JsonRpcResponse::success(request.id, result)
                    }
                    Err(e) => {
                        error!(tool = tool_name, error = %e, "Tool call failed");
                        JsonRpcResponse::error(
                            request.id,
                            -32000,
                            e.to_string(),
                        )
                    }
                }
            }
            "initialize" => {
                info!("Server initialized");
                JsonRpcResponse::success(
                    request.id,
                    json!({
                        "protocolVersion": "1.0",
                        "serverInfo": {
                            "name": "FileJack",
                            "version": "0.1.0"
                        },
                        "capabilities": {
                            "tools": {}
                        }
                    }),
                )
            }
            _ => {
                warn!(method = %request.method, "Method not found");
                JsonRpcResponse::error(
                    request.id,
                    -32601,
                    format!("Method not found: {}", request.method),
                )
            }
        }
    }

    /// Process a JSON-RPC request from a string
    pub fn process_request(&self, request_str: &str) -> String {
        // Check rate limit
        if !self.rate_limiter.check() {
            warn!("Rate limit exceeded");
            let error_response = JsonRpcResponse::error(
                None,
                -32000,
                "Rate limit exceeded. Please slow down requests.".to_string(),
            );
            return serde_json::to_string(&error_response).unwrap();
        }

        match serde_json::from_str::<JsonRpcRequest>(request_str) {
            Ok(request) => {
                // JSON-RPC 2.0: If id is None, it's a notification and should not be responded to
                if request.id.is_none() {
                    // For notifications, we still process them but return empty string
                    // (or could return empty to indicate no response needed)
                    self.handle_request(request);
                    return String::new();
                }
                
                let response = self.handle_request(request);
                serde_json::to_string(&response).unwrap()
            }
            Err(e) => {
                error!("Failed to parse request: {}", e);
                let error_response = JsonRpcResponse::error(
                    None,
                    -32700,
                    format!("Parse error: {}", e),
                );
                serde_json::to_string(&error_response).unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_mcp_server_new() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        assert!(server.list_tools().len() > 0);
    }

    #[test]
    fn test_mcp_server_with_base_path() {
        let temp_dir = TempDir::new().unwrap();
        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let server = McpServer::new(policy);
        assert!(server.list_tools().len() > 0);
    }

    #[test]
    fn test_list_tools() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        let tools = server.list_tools();
        
        assert_eq!(tools.len(), 7); // Updated: read_file, write_file, list_directory, get_metadata, delete_file, move_file, copy_file
        assert!(tools.iter().any(|t| t.name == "read_file"));
        assert!(tools.iter().any(|t| t.name == "write_file"));
        assert!(tools.iter().any(|t| t.name == "list_directory"));
        assert!(tools.iter().any(|t| t.name == "get_metadata"));
        assert!(tools.iter().any(|t| t.name == "delete_file"));
        assert!(tools.iter().any(|t| t.name == "move_file"));
        assert!(tools.iter().any(|t| t.name == "copy_file"));
    }

    #[test]
    fn test_handle_tool_call_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, MCP!").unwrap();

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let server = McpServer::new(policy);
        let result = server.handle_tool_call(
            "read_file",
            json!({"path": file_path.to_str().unwrap()})
        ).unwrap();

        assert_eq!(result["content"][0]["type"], "text");
        assert_eq!(result["content"][0]["text"], "Hello, MCP!");
    }

    #[test]
    fn test_handle_tool_call_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let server = McpServer::new(policy);
        let result = server.handle_tool_call(
            "write_file",
            json!({
                "path": file_path.to_str().unwrap(),
                "content": "MCP write test"
            })
        ).unwrap();

        assert_eq!(result["content"][0]["type"], "text");
        assert!(result["content"][0]["text"].as_str().unwrap().contains("Successfully wrote"));

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "MCP write test");
    }

    #[test]
    fn test_handle_tool_call_invalid_tool() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        let result = server.handle_tool_call("invalid_tool", json!({}));
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileJackError::ToolNotFound(_)));
    }

    #[test]
    fn test_handle_request_tools_list() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_handle_request_tools_call() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Test content").unwrap();

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let server = McpServer::new(policy);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "read_file",
                "arguments": {"path": file_path.to_str().unwrap()}
            })),
            id: Some(json!(2)),
        };

        let response = server.handle_request(request);
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["content"][0]["type"], "text");
        assert_eq!(result["content"][0]["text"], "Test content");
    }

    #[test]
    fn test_handle_request_initialize() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert!(response.result.is_some());
        
        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "1.0");
        assert_eq!(result["serverInfo"]["name"], "FileJack");
    }

    #[test]
    fn test_handle_request_unknown_method() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "unknown/method".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
    }

    #[test]
    fn test_process_request_valid_json() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        let request_str = r#"{"jsonrpc":"2.0","method":"tools/list","id":1}"#;
        
        let response_str = server.process_request(request_str);
        let response: JsonRpcResponse = serde_json::from_str(&response_str).unwrap();
        
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
    }

    #[test]
    fn test_process_request_invalid_json() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        let request_str = r#"{"invalid json"#;
        
        let response_str = server.process_request(request_str);
        let response: JsonRpcResponse = serde_json::from_str(&response_str).unwrap();
        
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32700);
    }

    #[test]
    fn test_process_request_read_write_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("workflow.txt");
        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let server = McpServer::new(policy);

        // Write file
        let write_request = format!(
            r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"Workflow test"}}}}, "id":1}}"#,
            file_path.to_str().unwrap()
        );
        
        let write_response_str = server.process_request(&write_request);
        let write_response: JsonRpcResponse = serde_json::from_str(&write_response_str).unwrap();
        assert!(write_response.result.is_some());

        // Read file
        let read_request = format!(
            r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":2}}"#,
            file_path.to_str().unwrap()
        );
        
        let read_response_str = server.process_request(&read_request);
        let read_response: JsonRpcResponse = serde_json::from_str(&read_response_str).unwrap();
        
        assert!(read_response.result.is_some());
        let result = read_response.result.unwrap();
        assert_eq!(result["content"][0]["type"], "text");
        assert_eq!(result["content"][0]["text"], "Workflow test");
    }

    #[test]
    fn test_handle_tool_call_with_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("subdir").join("nested.txt");

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let server = McpServer::new(policy);
        let result = server.handle_tool_call(
            "write_file",
            json!({
                "path": nested_path.to_str().unwrap(),
                "content": "Nested file content"
            })
        ).unwrap();

        assert_eq!(result["content"][0]["type"], "text");
        assert!(result["content"][0]["text"].as_str().unwrap().contains("Successfully wrote"));
        assert!(nested_path.exists());
    }

    #[test]
    fn test_tools_have_proper_schema() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        let tools = server.list_tools();

        for tool in tools {
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
            assert!(tool.input_schema.is_object());
        }
    }

    #[test]
    fn test_handle_tool_call_with_empty_arguments() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        
        // This should fail with a clear error message about missing path
        let result = server.handle_tool_call("read_file", json!({}));
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        
        match error {
            FileJackError::InvalidParameters(msg) => {
                assert!(msg.contains("path"), "Error message should mention 'path': {}", msg);
                assert!(msg.contains("Invalid parameters"), "Error message should be helpful: {}", msg);
            }
            _ => panic!("Expected InvalidParameters error, got: {:?}", error),
        }
    }

    #[test]
    fn test_handle_tool_call_with_missing_path() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        
        // Missing 'path' field
        let result = server.handle_tool_call("read_file", json!({"wrong_field": "value"}));
        
        assert!(result.is_err());
        match result.unwrap_err() {
            FileJackError::InvalidParameters(msg) => {
                assert!(msg.contains("path"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[test]
    fn test_handle_tool_call_write_file_missing_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let server = McpServer::new(policy);
        
        // Missing 'content' field
        let result = server.handle_tool_call(
            "write_file",
            json!({"path": file_path.to_str().unwrap()})
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            FileJackError::InvalidParameters(msg) => {
                assert!(msg.contains("content") || msg.contains("missing field"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[test]
    fn test_handle_request_tools_call_with_empty_arguments() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        
        // Simulate the exact request that VS Code MCP extension was sending
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "read_file",
                "arguments": {}
            })),
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        
        // Should return an error, not success
        assert!(response.error.is_some());
        assert!(response.result.is_none());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32000);
        assert!(error.message.contains("path"), "Error message should mention missing 'path': {}", error.message);
    }

    #[test]
    fn test_process_request_with_empty_arguments_string() {
        let policy = AccessPolicy::permissive();
        let server = McpServer::new(policy);
        
        // The exact JSON that was failing
        let request_str = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"read_file","arguments":{}}}"#;
        
        let response_str = server.process_request(request_str);
        let response: JsonRpcResponse = serde_json::from_str(&response_str).unwrap();
        
        // Should have an error about missing path
        assert!(response.error.is_some());
        assert!(response.result.is_none());
        
        let error = response.error.unwrap();
        assert!(error.message.contains("path"), "Error should mention 'path': {}", error.message);
    }
}
