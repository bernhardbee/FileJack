use filejack::{AccessPolicy, McpServer};
use std::env;

/// Example: Creating an MCP server with custom configuration
fn main() {
    println!("FileJack Basic Usage Example\n");

    // Example 1: Create server without restrictions (permissive mode)
    println!("1. Creating server with permissive policy:");
    let policy_permissive = AccessPolicy::permissive();
    let server_unrestricted = McpServer::new(policy_permissive);
    println!("   ✓ Server created (permissive mode)\n");

    // Example 2: Create server with restricted access
    println!("2. Creating server with restricted access:");
    let temp_dir = env::temp_dir();
    let policy_restricted = AccessPolicy::restricted(temp_dir.clone());
    let _server_restricted = McpServer::new(policy_restricted);
    println!("   ✓ Server created with base path: {}\n", temp_dir.display());

    // Example 3: List available tools
    println!("3. Listing available tools:");
    let tools = server_unrestricted.list_tools();
    for tool in &tools {
        println!("   - {}: {}", tool.name, tool.description);
    }
    println!("   ✓ Found {} tools\n", tools.len());

    // Example 4: Process an initialize request
    println!("4. Processing initialize request:");
    let init_request = r#"{"jsonrpc":"2.0","method":"initialize","id":1}"#;
    let init_response = server_unrestricted.process_request(init_request);
    println!("   Request:  {}", init_request);
    println!("   Response: {}\n", init_response);

    // Example 5: Process a tools/list request
    println!("5. Processing tools/list request:");
    let list_request = r#"{"jsonrpc":"2.0","method":"tools/list","id":2}"#;
    let list_response = server_unrestricted.process_request(list_request);
    println!("   Request:  {}", list_request);
    println!("   Response length: {} bytes\n", list_response.len());

    // Example 6: Demonstrate error handling
    println!("6. Error handling example:");
    let error_request = r#"{"jsonrpc":"2.0","method":"invalid_method","id":3}"#;
    let error_response = server_unrestricted.process_request(error_request);
    println!("   Request:  {}", error_request);
    println!("   Response: {}\n", error_response);

    println!("Example completed successfully!");
}
