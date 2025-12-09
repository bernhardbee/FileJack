use filejack::McpServer;
use serde_json::json;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

fn main() {
    eprintln!("FileJack MCP Server v0.1.0");
    eprintln!("Starting server...");

    // Get base path from environment or use current directory
    let base_path = std::env::var("FILEJACK_BASE_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok());

    if let Some(ref path) = base_path {
        eprintln!("Base path: {}", path.display());
    } else {
        eprintln!("Base path: unrestricted");
    }

    let server = McpServer::new(base_path);
    eprintln!("Server initialized. Waiting for JSON-RPC requests on stdin...");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        match line {
            Ok(request_str) => {
                if request_str.trim().is_empty() {
                    continue;
                }

                eprintln!("Received request: {}", request_str);
                
                let response_str = server.process_request(&request_str);
                
                eprintln!("Sending response: {}", response_str);
                
                if let Err(e) = writeln!(stdout, "{}", response_str) {
                    eprintln!("Error writing response: {}", e);
                    break;
                }
                
                if let Err(e) = stdout.flush() {
                    eprintln!("Error flushing stdout: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": format!("Failed to read input: {}", e)
                    },
                    "id": null
                });
                
                if let Err(e) = writeln!(stdout, "{}", error_response) {
                    eprintln!("Error writing error response: {}", e);
                }
                break;
            }
        }
    }

    eprintln!("Server shutting down...");
}
