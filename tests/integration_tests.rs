use filejack::{AccessPolicy, McpServer};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_complete_mcp_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));

    // Test initialize
    let init_request = r#"{"jsonrpc":"2.0","method":"initialize","id":1}"#;
    let init_response = server.process_request(init_request);
    assert!(init_response.contains("FileJack"));
    assert!(init_response.contains("protocolVersion"));

    // Test tools/list
    let list_request = r#"{"jsonrpc":"2.0","method":"tools/list","id":2}"#;
    let list_response = server.process_request(list_request);
    assert!(list_response.contains("read_file"));
    assert!(list_response.contains("write_file"));

    // Test write operation
    let file_path = temp_dir.path().join("integration_test.txt");
    let write_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"Integration test content"}}}}, "id":3}}"#,
        file_path.to_str().unwrap()
    );
    let write_response = server.process_request(&write_request);
    assert!(write_response.contains("Successfully wrote"));

    // Verify file was created
    assert!(file_path.exists());

    // Test read operation
    let read_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":4}}"#,
        file_path.to_str().unwrap()
    );
    let read_response = server.process_request(&read_request);
    assert!(read_response.contains("Integration test content"));
}

#[test]
fn test_multiple_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));

    // Create multiple files
    for i in 1..=5 {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));
        let write_request = format!(
            r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"Content {}"}}}}, "id":{}}}"#,
            file_path.to_str().unwrap(),
            i,
            i
        );
        let response = server.process_request(&write_request);
        assert!(response.contains("Successfully wrote"));
    }

    // Read all files
    for i in 1..=5 {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));
        let read_request = format!(
            r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":{}}}"#,
            file_path.to_str().unwrap(),
            i + 10
        );
        let response = server.process_request(&read_request);
        assert!(response.contains(&format!("Content {}", i)));
    }
}

#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));

    // Test reading non-existent file
    let nonexistent_path = temp_dir.path().join("nonexistent.txt");
    let read_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        nonexistent_path.to_str().unwrap()
    );
    let response = server.process_request(&read_request);
    assert!(response.contains("error"));
    assert!(response.contains("File not found") || response.contains("not found"));

    // Test invalid tool name
    let invalid_tool_request = r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"invalid_tool","arguments":{}}, "id":2}"#;
    let response = server.process_request(invalid_tool_request);
    assert!(response.contains("error"));

    // Test invalid JSON
    let invalid_json = r#"{"invalid": json}"#;
    let response = server.process_request(invalid_json);
    assert!(response.contains("Parse error") || response.contains("error"));
}

#[test]
fn test_nested_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));

    let nested_path = temp_dir.path().join("level1").join("level2").join("file.txt");
    let write_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"Nested content"}}}}, "id":1}}"#,
        nested_path.to_str().unwrap()
    );
    
    let response = server.process_request(&write_request);
    assert!(response.contains("Successfully wrote"));
    assert!(nested_path.exists());

    // Verify we can read the nested file
    let read_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":2}}"#,
        nested_path.to_str().unwrap()
    );
    let response = server.process_request(&read_request);
    assert!(response.contains("Nested content"));
}

#[test]
fn test_large_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));

    // Create a large content string (1MB)
    let large_content = "x".repeat(1024 * 1024);
    let file_path = temp_dir.path().join("large_file.txt");
    
    let write_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"{}"}}}}, "id":1}}"#,
        file_path.to_str().unwrap(),
        large_content
    );
    
    let response = server.process_request(&write_request);
    assert!(response.contains("Successfully wrote"));
    assert!(response.contains(&format!("{} bytes", large_content.len())));

    // Read the large file - don't print response to avoid huge output
    let read_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":2}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&read_request);
    // Check structure without printing full content
    assert!(response.contains(r#""content":"#));
    assert!(response.contains(r#""type":"text""#));
    assert!(response.len() > 1024 * 1024); // Response should be large
}

#[test]
fn test_file_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    let file_path = temp_dir.path().join("overwrite_test.txt");

    // Write initial content
    let write1_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"Original content"}}}}, "id":1}}"#,
        file_path.to_str().unwrap()
    );
    server.process_request(&write1_request);

    // Overwrite with new content
    let write2_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"New content"}}}}, "id":2}}"#,
        file_path.to_str().unwrap()
    );
    server.process_request(&write2_request);

    // Verify new content
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "New content");
}

#[test]
fn test_special_characters_in_content() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    let file_path = temp_dir.path().join("special_chars.txt");

    let special_content = "Line1\nLine2\tTabbed\r\nWindows line\n\"Quoted\" and 'apostrophe' content\n🚀 Emoji support!";
    
    let write_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"{}"}}}}, "id":1}}"#,
        file_path.to_str().unwrap(),
        special_content.replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t").replace('"', "\\\"")
    );
    
    let response = server.process_request(&write_request);
    assert!(response.contains("Successfully wrote"));

    // Read and verify
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("Line1"));
    assert!(content.contains("🚀"));
}

#[test]
fn test_concurrent_operations_simulation() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));

    // Simulate multiple concurrent operations by executing them sequentially
    // In a real scenario, this would use async/threading
    let operations = vec![
        ("file1.txt", "Content 1"),
        ("file2.txt", "Content 2"),
        ("file3.txt", "Content 3"),
    ];

    for (filename, content) in operations.iter() {
        let file_path = temp_dir.path().join(filename);
        let write_request = format!(
            r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"{}"}}}}, "id":1}}"#,
            file_path.to_str().unwrap(),
            content
        );
        let response = server.process_request(&write_request);
        assert!(response.contains("Successfully wrote"));
    }

    // Verify all files exist
    for (filename, expected_content) in operations.iter() {
        let file_path = temp_dir.path().join(filename);
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(&content, expected_content);
    }
}
