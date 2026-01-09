use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Tool call parameters for file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

/// File read parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileParams {
    pub path: String,
}

/// File write parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileParams {
    pub path: String,
    pub content: String,
}

/// List directory parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDirectoryParams {
    pub path: String,
    #[serde(default)]
    pub recursive: bool,
}

/// Get metadata parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMetadataParams {
    pub path: String,
}

/// Delete file parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileParams {
    pub path: String,
}

/// Move file parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFileParams {
    pub from: String,
    pub to: String,
}

/// Copy file parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyFileParams {
    pub from: String,
    pub to: String,
}

/// Append file parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendFileParams {
    pub path: String,
    pub content: String,
}

/// File exists parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExistsParams {
    pub path: String,
}

/// Create directory parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDirectoryParams {
    pub path: String,
    #[serde(default)]
    pub recursive: bool,
}

/// Remove directory parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveDirectoryParams {
    pub path: String,
    #[serde(default)]
    pub recursive: bool,
}

/// Read lines parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadLinesParams {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail: Option<usize>,
}

/// Search files parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilesParams {
    pub path: String,
    pub pattern: String,
    #[serde(default = "default_true")]
    pub recursive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<usize>,
}

fn default_true() -> bool {
    true
}

/// Grep file parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepFileParams {
    pub path: String,
    pub pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_matches: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_lines: Option<usize>,
}

/// Grep match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepMatch {
    pub line_number: usize,
    pub line_content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub context_before: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub context_after: Vec<String>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({"name": "read_file", "arguments": {"path": "test.txt"}})),
            id: Some(json!(1)),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"method\":\"tools/call\""));
    }

    #[test]
    fn test_json_rpc_request_deserialization() {
        let json_str = r#"{"jsonrpc":"2.0","method":"tools/list","id":1}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tools/list");
        assert_eq!(request.id, Some(json!(1)));
    }

    #[test]
    fn test_json_rpc_success_response() {
        let response = JsonRpcResponse::success(Some(json!(1)), json!({"status": "ok"}));
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, Some(json!(1)));
    }

    #[test]
    fn test_json_rpc_error_response() {
        let response = JsonRpcResponse::error(Some(json!(1)), -32600, "Invalid request".to_string());
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid request");
    }

    #[test]
    fn test_mcp_tool_serialization() {
        let tool = McpTool {
            name: "read_file".to_string(),
            description: "Read a file".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                }
            }),
        };

        let serialized = serde_json::to_string(&tool).unwrap();
        assert!(serialized.contains("read_file"));
    }

    #[test]
    fn test_read_file_params() {
        let params = ReadFileParams {
            path: "/test/file.txt".to_string(),
        };
        
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["path"], "/test/file.txt");
        
        let deserialized: ReadFileParams = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.path, "/test/file.txt");
    }

    #[test]
    fn test_write_file_params() {
        let params = WriteFileParams {
            path: "/test/file.txt".to_string(),
            content: "Hello, World!".to_string(),
        };
        
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["path"], "/test/file.txt");
        assert_eq!(json["content"], "Hello, World!");
        
        let deserialized: WriteFileParams = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.path, "/test/file.txt");
        assert_eq!(deserialized.content, "Hello, World!");
    }

    #[test]
    fn test_tool_call() {
        let call = ToolCall {
            name: "read_file".to_string(),
            arguments: json!({"path": "test.txt"}),
        };
        
        assert_eq!(call.name, "read_file");
        assert_eq!(call.arguments["path"], "test.txt");
    }
}
