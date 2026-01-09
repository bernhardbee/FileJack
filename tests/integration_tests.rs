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

#[test]
fn test_append_file() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let file_path = temp_dir.path().join("append_test.txt");
    
    // First write
    let write_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"Line 1\n"}}}}, "id":1}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&write_request);
    assert!(response.contains("Successfully wrote"));
    
    // Append second line
    let append_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"append_file","arguments":{{"path":"{}","content":"Line 2\n"}}}}, "id":2}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&append_request);
    assert!(response.contains("Successfully appended"));
    
    // Append third line
    let append_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"append_file","arguments":{{"path":"{}","content":"Line 3\n"}}}}, "id":3}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&append_request);
    assert!(response.contains("Successfully appended"));
    
    // Verify content
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Line 1\nLine 2\nLine 3\n");
}

#[test]
fn test_file_exists() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let file_path = temp_dir.path().join("exists_test.txt");
    
    // Check non-existent file
    let exists_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"file_exists","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&exists_request);
    assert!(response.contains("false"));
    
    // Create file
    let write_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"test"}}}}, "id":2}}"#,
        file_path.to_str().unwrap()
    );
    server.process_request(&write_request);
    
    // Check existing file
    let exists_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"file_exists","arguments":{{"path":"{}"}}}}, "id":3}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&exists_request);
    assert!(response.contains("true"));
}

#[test]
fn test_create_directory() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let dir_path = temp_dir.path().join("test_dir");
    
    // Create directory
    let create_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"create_directory","arguments":{{"path":"{}","recursive":false}}}}, "id":1}}"#,
        dir_path.to_str().unwrap()
    );
    let response = server.process_request(&create_request);
    assert!(response.contains("Successfully created directory"));
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

#[test]
fn test_create_directory_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let dir_path = temp_dir.path().join("parent").join("child").join("grandchild");
    
    // Create nested directories
    let create_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"create_directory","arguments":{{"path":"{}","recursive":true}}}}, "id":1}}"#,
        dir_path.to_str().unwrap()
    );
    let response = server.process_request(&create_request);
    assert!(response.contains("Successfully created directory"));
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

#[test]
fn test_remove_directory() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let dir_path = temp_dir.path().join("remove_test");
    fs::create_dir(&dir_path).unwrap();
    assert!(dir_path.exists());
    
    // Remove empty directory
    let remove_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"remove_directory","arguments":{{"path":"{}","recursive":false}}}}, "id":1}}"#,
        dir_path.to_str().unwrap()
    );
    let response = server.process_request(&remove_request);
    assert!(response.contains("Successfully removed directory"));
    assert!(!dir_path.exists());
}

#[test]
fn test_remove_directory_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let dir_path = temp_dir.path().join("remove_recursive_test");
    fs::create_dir(&dir_path).unwrap();
    fs::write(dir_path.join("file.txt"), "content").unwrap();
    assert!(dir_path.exists());
    
    // Remove directory with contents
    let remove_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"remove_directory","arguments":{{"path":"{}","recursive":true}}}}, "id":1}}"#,
        dir_path.to_str().unwrap()
    );
    let response = server.process_request(&remove_request);
    assert!(response.contains("Successfully removed directory"));
    assert!(!dir_path.exists());
}

#[test]
fn test_read_lines() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let file_path = temp_dir.path().join("lines_test.txt");
    fs::write(&file_path, "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n").unwrap();
    
    // Read lines 2-4
    let read_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_lines","arguments":{{"path":"{}","start_line":2,"end_line":4}}}}, "id":1}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&read_request);
    assert!(response.contains("Line 2"));
    assert!(response.contains("Line 3"));
    assert!(response.contains("Line 4"));
    assert!(!response.contains("Line 1"));
    assert!(!response.contains("Line 5"));
}

#[test]
fn test_read_lines_tail() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let file_path = temp_dir.path().join("tail_test.txt");
    fs::write(&file_path, "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n").unwrap();
    
    // Read last 2 lines
    let read_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_lines","arguments":{{"path":"{}","tail":2}}}}, "id":1}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&read_request);
    assert!(response.contains("Line 4"));
    assert!(response.contains("Line 5"));
    assert!(!response.contains("Line 1"));
    assert!(!response.contains("Line 2"));
    assert!(!response.contains("Line 3"));
}

#[test]
fn test_search_files() {
    let temp_dir = TempDir::new().unwrap();
    let mut policy = AccessPolicy::permissive();
    policy.allowed_paths = vec![temp_dir.path().to_path_buf()];
    let server = McpServer::new(policy);
    
    // Create test files
    fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
    fs::write(temp_dir.path().join("test2.txt"), "content").unwrap();
    fs::write(temp_dir.path().join("other.log"), "content").unwrap();
    
    // Search for .txt files
    let search_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"search_files","arguments":{{"path":"{}","pattern":"*.txt","recursive":false}}}}, "id":1}}"#,
        temp_dir.path().to_str().unwrap()
    );
    let response = server.process_request(&search_request);
    // Check for full paths since search_files returns full paths
    assert!(response.contains("test1.txt"));
    assert!(response.contains("test2.txt"));
    // Verify it doesn't contain the log file
    assert!(!response.contains("other.log"));
}

#[test]
fn test_search_files_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let mut policy = AccessPolicy::permissive();
    policy.allowed_paths = vec![temp_dir.path().to_path_buf()];
    let server = McpServer::new(policy);
    
    // Create nested structure
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(temp_dir.path().join("root.log"), "content").unwrap();
    fs::write(subdir.join("nested.log"), "content").unwrap();
    
    // Search recursively for .log files
    let search_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"search_files","arguments":{{"path":"{}","pattern":"*.log","recursive":true}}}}, "id":1}}"#,
        temp_dir.path().to_str().unwrap()
    );
    let response = server.process_request(&search_request);
    // Check that both files are found
    assert!(response.contains("root.log"));
    assert!(response.contains("nested.log"));
}

#[test]
fn test_grep_file() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let file_path = temp_dir.path().join("grep_test.txt");
    fs::write(&file_path, "Line 1: INFO message\nLine 2: DEBUG message\nLine 3: ERROR occurred\nLine 4: INFO again\nLine 5: DEBUG trace\n").unwrap();
    
    // Search for ERROR
    let grep_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"grep_file","arguments":{{"path":"{}","pattern":"ERROR"}}}}, "id":1}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&grep_request);
    assert!(response.contains("ERROR occurred"));
    assert!(response.contains("line_number"));
}

#[test]
fn test_grep_file_with_context() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let file_path = temp_dir.path().join("grep_context_test.txt");
    fs::write(&file_path, "Line 1\nLine 2\nLine 3: MATCH\nLine 4\nLine 5\n").unwrap();
    
    // Search with context
    let grep_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"grep_file","arguments":{{"path":"{}","pattern":"MATCH","context_lines":1}}}}, "id":1}}"#,
        file_path.to_str().unwrap()
    );
    let response = server.process_request(&grep_request);
    assert!(response.contains("MATCH"));
    assert!(response.contains("context_before"));
    assert!(response.contains("context_after"));
}

#[test]
fn test_tools_list_includes_new_tools() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    let list_request = r#"{"jsonrpc":"2.0","method":"tools/list","id":1}"#;
    let response = server.process_request(list_request);
    
    // Check for new tools
    assert!(response.contains("append_file"));
    assert!(response.contains("file_exists"));
    assert!(response.contains("create_directory"));
    assert!(response.contains("remove_directory"));
    assert!(response.contains("read_lines"));
    assert!(response.contains("search_files"));
    assert!(response.contains("grep_file"));
}
