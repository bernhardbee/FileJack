use filejack::McpServer;
use std::fs;
use tempfile::TempDir;

/// Example: Complete file operations workflow
fn main() {
    println!("FileJack File Operations Example\n");

    // Create a temporary directory for this example
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    println!("Working directory: {}\n", temp_dir.path().display());

    // Create an MCP server with the temp directory as base path
    let server = McpServer::new(Some(temp_dir.path().to_path_buf()));

    // Example 1: Write a simple file
    println!("1. Writing a simple text file:");
    let file_path = temp_dir.path().join("example.txt");
    let write_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"Hello, FileJack!"}}}}, "id":1}}"#,
        file_path.to_str().unwrap()
    );
    let write_response = server.process_request(&write_request);
    println!("   Response: {}\n", write_response);

    // Example 2: Read the file back
    println!("2. Reading the file:");
    let read_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":2}}"#,
        file_path.to_str().unwrap()
    );
    let read_response = server.process_request(&read_request);
    println!("   Response: {}\n", read_response);

    // Example 3: Write file in nested directory
    println!("3. Writing file in nested directory:");
    let nested_path = temp_dir.path().join("subdir").join("nested").join("file.txt");
    let nested_write_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"Nested file content"}}}}, "id":3}}"#,
        nested_path.to_str().unwrap()
    );
    let nested_write_response = server.process_request(&nested_write_request);
    println!("   Response: {}\n", nested_write_response);

    // Verify the nested file was created
    assert!(nested_path.exists(), "Nested file should exist");
    println!("   ✓ Nested file created successfully\n");

    // Example 4: Overwrite existing file
    println!("4. Overwriting existing file:");
    let overwrite_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"Updated content!"}}}}, "id":4}}"#,
        file_path.to_str().unwrap()
    );
    let overwrite_response = server.process_request(&overwrite_request);
    println!("   Response: {}\n", overwrite_response);

    // Verify the file was overwritten
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "Updated content!");
    println!("   ✓ File overwritten successfully\n");

    // Example 5: Handle read error for non-existent file
    println!("5. Reading non-existent file (error handling):");
    let nonexistent_path = temp_dir.path().join("nonexistent.txt");
    let error_read_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":5}}"#,
        nonexistent_path.to_str().unwrap()
    );
    let error_response = server.process_request(&error_read_request);
    println!("   Response: {}\n", error_response);

    // Example 6: Write file with special characters
    println!("6. Writing file with special characters:");
    let special_path = temp_dir.path().join("special.txt");
    let special_content = r#"Line 1
Line 2 with "quotes"
Line 3 with 'apostrophes'
Line 4 with emoji 🚀
Line 5 with tab	character"#;
    
    let special_write_request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"{}"}}}}, "id":6}}"#,
        special_path.to_str().unwrap(),
        special_content.replace('\n', "\\n").replace('\t', "\\t").replace('"', "\\\"")
    );
    let special_write_response = server.process_request(&special_write_request);
    println!("   Response: {}\n", special_write_response);

    // Verify special characters
    let special_file_content = fs::read_to_string(&special_path).expect("Failed to read file");
    assert!(special_file_content.contains("🚀"));
    println!("   ✓ Special characters handled correctly\n");

    println!("All file operations completed successfully!");
    println!("\nFiles created in temporary directory:");
    println!("  - {}", file_path.display());
    println!("  - {}", nested_path.display());
    println!("  - {}", special_path.display());
}
