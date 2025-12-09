use crate::access_control::AccessPolicy;
use crate::error::{FileJackError, Result};
use crate::file_ops::{FileReader, FileWriter};
use crate::protocol::{
    JsonRpcRequest, JsonRpcResponse, McpTool, ReadFileParams, WriteFileParams,
};
use serde_json::{json, Value};

/// MCP Server for file operations
pub struct McpServer {
    reader: FileReader,
    writer: FileWriter,
}

impl McpServer {
    /// Create a new MCP Server with an access policy
    pub fn new(policy: AccessPolicy) -> Self {
        Self {
            reader: FileReader::new(policy.clone()),
            writer: FileWriter::new(policy, true),
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
        ]
    }

    /// Handle a tool call
    pub fn handle_tool_call(&self, name: &str, arguments: Value) -> Result<Value> {
        match name {
            "read_file" => {
                let params: ReadFileParams = serde_json::from_value(arguments)
                    .map_err(|e| FileJackError::InvalidParameters(e.to_string()))?;
                
                let content = self.reader.read_to_string(&params.path)?;
                Ok(json!({
                    "content": content,
                    "path": params.path
                }))
            }
            "write_file" => {
                let params: WriteFileParams = serde_json::from_value(arguments)
                    .map_err(|e| FileJackError::InvalidParameters(e.to_string()))?;
                
                self.writer.write_string(&params.path, &params.content)?;
                Ok(json!({
                    "success": true,
                    "path": params.path,
                    "bytes_written": params.content.len()
                }))
            }
            _ => Err(FileJackError::ToolNotFound(name.to_string())),
        }
    }

    /// Handle a JSON-RPC request
    pub fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "tools/list" => {
                let tools = self.list_tools();
                let tools_value = serde_json::to_value(&tools).unwrap();
                JsonRpcResponse::success(request.id, json!({"tools": tools_value}))
            }
            "tools/call" => {
                let params = request.params.unwrap_or(json!({}));
                
                let tool_name = params.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let arguments = params.get("arguments")
                    .cloned()
                    .unwrap_or(json!({}));

                match self.handle_tool_call(tool_name, arguments) {
                    Ok(result) => JsonRpcResponse::success(request.id, result),
                    Err(e) => JsonRpcResponse::error(
                        request.id,
                        -32000,
                        e.to_string(),
                    ),
                }
            }
            "initialize" => {
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
            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    /// Process a JSON-RPC request from a string
    pub fn process_request(&self, request_str: &str) -> String {
        match serde_json::from_str::<JsonRpcRequest>(request_str) {
            Ok(request) => {
                let response = self.handle_request(request);
                serde_json::to_string(&response).unwrap()
            }
            Err(e) => {
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
        
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.name == "read_file"));
        assert!(tools.iter().any(|t| t.name == "write_file"));
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

        assert_eq!(result["content"], "Hello, MCP!");
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

        assert_eq!(result["success"], true);
        assert_eq!(result["bytes_written"], 14);

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
        assert_eq!(response.result.unwrap()["content"], "Test content");
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
        assert_eq!(read_response.result.unwrap()["content"], "Workflow test");
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

        assert_eq!(result["success"], true);
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
}
