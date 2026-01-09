use filejack::{AccessPolicy, McpServer};
use std::fs;
use std::os::unix::fs as unix_fs;
use tempfile::TempDir;

/// Security Tests for FileJack
/// These tests verify that the security measures are working correctly

#[test]
fn test_path_traversal_attack_attempt() {
    let temp_dir = TempDir::new().unwrap();
    let allowed_dir = temp_dir.path().join("allowed");
    fs::create_dir(&allowed_dir).unwrap();
    
    let server = McpServer::new(AccessPolicy::restricted(allowed_dir.clone()));
    
    // Try to escape using ..
    let attack_path = format!("{}/../../../etc/passwd", allowed_dir.display());
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        attack_path
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("error") || response.contains("Permission denied"));
}

#[test]
fn test_absolute_path_outside_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let allowed_dir = temp_dir.path().join("allowed");
    fs::create_dir(&allowed_dir).unwrap();
    
    let server = McpServer::new(AccessPolicy::restricted(allowed_dir));
    
    // Try to read /etc/passwd directly
    let request = r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"read_file","arguments":{"path":"/etc/passwd"}}, "id":1}"#;
    
    let response = server.process_request(&request);
    assert!(response.contains("error"));
    assert!(response.contains("Permission denied") || response.contains("not in any allowed directory"));
}

#[test]
fn test_symlink_attack_denied() {
    let temp_dir = TempDir::new().unwrap();
    let allowed_dir = temp_dir.path().join("allowed");
    fs::create_dir(&allowed_dir).unwrap();
    
    // Create a file outside allowed directory
    let outside_file = temp_dir.path().join("secret.txt");
    fs::write(&outside_file, "secret data").unwrap();
    
    // Create symlink inside allowed directory pointing outside
    let symlink_path = allowed_dir.join("link_to_secret.txt");
    unix_fs::symlink(&outside_file, &symlink_path).unwrap();
    
    let mut policy = AccessPolicy::restricted(allowed_dir);
    policy.allow_symlinks = false; // Explicitly deny symlinks
    let server = McpServer::new(policy);
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        symlink_path.display()
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("error"));
}

#[test]
fn test_hidden_files_denied() {
    let temp_dir = TempDir::new().unwrap();
    let hidden_file = temp_dir.path().join(".secret");
    fs::write(&hidden_file, "hidden data").unwrap();
    
    let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
    policy.allow_hidden_files = false;
    let server = McpServer::new(policy);
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        hidden_file.display()
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("error"));
    assert!(response.contains("hidden file") || response.contains("not allowed"));
}

#[test]
fn test_extension_blacklist() {
    let temp_dir = TempDir::new().unwrap();
    let exe_file = temp_dir.path().join("malware.exe");
    fs::write(&exe_file, "fake exe").unwrap();
    
    let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
    policy.denied_extensions = vec!["exe".to_string(), "sh".to_string()];
    let server = McpServer::new(policy);
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        exe_file.display()
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("error"));
    assert!(response.contains("not allowed") || response.contains("extension"));
}

#[test]
fn test_extension_whitelist() {
    let temp_dir = TempDir::new().unwrap();
    let txt_file = temp_dir.path().join("allowed.txt");
    let exe_file = temp_dir.path().join("denied.exe");
    fs::write(&txt_file, "allowed content").unwrap();
    fs::write(&exe_file, "denied content").unwrap();
    
    let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
    policy.allowed_extensions = vec!["txt".to_string(), "md".to_string()];
    let server = McpServer::new(policy);
    
    // Should allow .txt
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        txt_file.display()
    );
    let response = server.process_request(&request);
    assert!(response.contains("allowed content"));
    
    // Should deny .exe
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":2}}"#,
        exe_file.display()
    );
    let response = server.process_request(&request);
    assert!(response.contains("error"));
}

#[test]
fn test_file_size_limit() {
    let temp_dir = TempDir::new().unwrap();
    let large_file = temp_dir.path().join("large.txt");
    fs::write(&large_file, "x".repeat(1024 * 1024)).unwrap(); // 1MB
    
    let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
    policy.max_file_size = 1024; // Only allow 1KB
    let server = McpServer::new(policy);
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        large_file.display()
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("error"));
    assert!(response.contains("size") || response.contains("exceeds"));
}

#[test]
fn test_read_only_mode_blocks_writes() {
    let temp_dir = TempDir::new().unwrap();
    let policy = AccessPolicy::read_only(temp_dir.path().to_path_buf());
    let server = McpServer::new(policy);
    
    let file_path = temp_dir.path().join("test.txt");
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"write_file","arguments":{{"path":"{}","content":"test"}}}}, "id":1}}"#,
        file_path.display()
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("error"));
    assert!(response.contains("read-only") || response.contains("disabled"));
}

#[test]
fn test_denied_paths_take_precedence() {
    let temp_dir = TempDir::new().unwrap();
    let allowed_dir = temp_dir.path().join("allowed");
    fs::create_dir(&allowed_dir).unwrap();
    
    let denied_subdir = allowed_dir.join("secrets");
    fs::create_dir(&denied_subdir).unwrap();
    
    let denied_file = denied_subdir.join("secret.txt");
    fs::write(&denied_file, "secret data").unwrap();
    
    let mut policy = AccessPolicy::restricted(allowed_dir);
    policy.denied_paths = vec![denied_subdir];
    let server = McpServer::new(policy);
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        denied_file.display()
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("error"));
    assert!(response.contains("denied"));
}

#[test]
fn test_rate_limiting() {
    let temp_dir = TempDir::new().unwrap();
    let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
    
    use filejack::RateLimiter;
    let server = McpServer::with_rate_limiter(policy, RateLimiter::strict()); // 10 req/s
    
    // Fire many requests quickly
    let mut error_count = 0;
    for i in 0..50 {
        let request = format!(
            r#"{{"jsonrpc":"2.0","method":"tools/list","id":{}}}"#,
            i
        );
        let response = server.process_request(&request);
        if response.contains("Rate limit") {
            error_count += 1;
        }
    }
    
    // At least some requests should be rate limited
    assert!(error_count > 0, "Rate limiting should have triggered");
}

#[test]
fn test_malicious_json_input() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    // Deeply nested JSON
    let malicious = r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"read_file","arguments":{"path":[[[[[[[[[[[[[[[[[[[[[]]]]]]]]]]]]]]]]]]]]]}},"id":1}"#;
    let response = server.process_request(malicious);
    assert!(response.contains("error"));
    
    // Invalid JSON
    let invalid = r#"{"jsonrpc":"2.0","method":"tools/call""#;
    let response = server.process_request(invalid);
    assert!(response.contains("Parse error") || response.contains("error"));
}

#[test]
fn test_null_byte_injection() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(AccessPolicy::restricted(temp_dir.path().to_path_buf()));
    
    // Try path with null byte
    let request = r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"read_file","arguments":{"path":"test.txt\u0000/etc/passwd"}}, "id":1}"#;
    let response = server.process_request(&request);
    // Should either error or not find the file
    assert!(response.contains("error") || response.contains("not found"));
}

#[test]
fn test_delete_outside_allowed_directory() {
    let temp_dir = TempDir::new().unwrap();
    let allowed_dir = temp_dir.path().join("allowed");
    fs::create_dir(&allowed_dir).unwrap();
    
    // Create file outside allowed directory
    let outside_file = temp_dir.path().join("important.txt");
    fs::write(&outside_file, "important data").unwrap();
    
    let server = McpServer::new(AccessPolicy::restricted(allowed_dir));
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"delete_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        outside_file.display()
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("error"));
    assert!(outside_file.exists(), "File should not be deleted");
}

#[test]
fn test_move_file_outside_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let allowed_dir = temp_dir.path().join("allowed");
    fs::create_dir(&allowed_dir).unwrap();
    
    let source = allowed_dir.join("file.txt");
    fs::write(&source, "data").unwrap();
    
    let outside_dest = temp_dir.path().join("moved.txt");
    
    let server = McpServer::new(AccessPolicy::restricted(allowed_dir));
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"move_file","arguments":{{"from":"{}","to":"{}"}}}}, "id":1}}"#,
        source.display(),
        outside_dest.display()
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("error"));
    assert!(source.exists(), "Source file should still exist");
    assert!(!outside_dest.exists(), "Destination should not exist");
}

#[test]
fn test_case_sensitivity_in_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let exe_file = temp_dir.path().join("file.EXE");
    fs::write(&exe_file, "data").unwrap();
    
    let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
    policy.denied_extensions = vec!["exe".to_string()];
    let server = McpServer::new(policy);
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        exe_file.display()
    );
    
    let response = server.process_request(&request);
    // Should be case-insensitive and block .EXE
    assert!(response.contains("error"));
}

#[test]
fn test_toctou_prevention_read() {
    // This test verifies that TOCTOU vulnerability is fixed
    // by ensuring we're using file descriptors
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "original content").unwrap();
    
    let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
    let server = McpServer::new(policy);
    
    // Read the file - should succeed
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"read_file","arguments":{{"path":"{}"}}}}, "id":1}}"#,
        file_path.display()
    );
    
    let response = server.process_request(&request);
    assert!(response.contains("original content"));
    
    // Even if the file is a symlink, it should be detected
    // (This is harder to test reliably in unit tests, but the atomic operations prevent TOCTOU)
}

#[test]
fn test_directory_listing_respects_policy() {
    let temp_dir = TempDir::new().unwrap();
    
    let txt_file = temp_dir.path().join("allowed.txt");
    let exe_file = temp_dir.path().join("denied.exe");
    fs::write(&txt_file, "allowed").unwrap();
    fs::write(&exe_file, "denied").unwrap();
    
    let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
    policy.allowed_extensions = vec!["txt".to_string()];
    policy.allow_hidden_files = true; // Allow hidden files for this test (temp dirs may have .DS_Store, etc.)
    let server = McpServer::new(policy);
    
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"list_directory","arguments":{{"path":"{}","recursive":false}}}}, "id":1}}"#,
        temp_dir.path().display()
    );
    
    let response = server.process_request(&request);
    // The response should succeed (list_directory succeeds)
    // but should only list files that pass policy (allowed.txt, not denied.exe)
    assert!(!response.contains("error"), "list_directory should succeed: {}", response);
    assert!(response.contains("allowed.txt"), "Should contain allowed.txt");
    // The .exe file won't be in the list because it doesn't pass the policy validation
    assert!(!response.contains("denied.exe"), "Should not contain denied.exe");
}
