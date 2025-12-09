use filejack::{AccessPolicy, Config, McpServer};
use std::path::PathBuf;

fn main() {
    println!("FileJack Access Control Examples\n");
    
    // Example 1: Restricted policy
    example_restricted_policy();
    
    // Example 2: Read-only policy
    example_read_only_policy();
    
    // Example 3: Extension filtering
    example_extension_filtering();
    
    // Example 4: Custom policy with multiple restrictions
    example_custom_policy();
    
    // Example 5: Load from configuration file
    example_config_file();
}

fn example_restricted_policy() {
    println!("Example 1: Restricted Policy");
    println!("==============================");
    
    let workspace = PathBuf::from("/home/user/workspace");
    let policy = AccessPolicy::restricted(workspace.clone());
    
    println!("Allowed paths: {:?}", policy.allowed_paths);
    println!("Max file size: {} bytes", policy.max_file_size);
    println!("Allow symlinks: {}", policy.allow_symlinks);
    println!("Allow hidden files: {}", policy.allow_hidden_files);
    println!("Read-only: {}\n", policy.read_only);
    
    let _server = McpServer::new(policy);
    println!("Server created with restricted access to {:?}\n", workspace);
}

fn example_read_only_policy() {
    println!("Example 2: Read-Only Policy");
    println!("============================");
    
    let docs = PathBuf::from("/usr/share/doc");
    let policy = AccessPolicy::read_only(docs.clone());
    
    println!("Allowed paths: {:?}", policy.allowed_paths);
    println!("Read-only: {}\n", policy.read_only);
    
    let _server = McpServer::new(policy);
    println!("Server created with read-only access to {:?}\n", docs);
}

fn example_extension_filtering() {
    println!("Example 3: Extension Filtering");
    println!("================================");
    
    let workspace = PathBuf::from("/home/user/data");
    let mut policy = AccessPolicy::restricted(workspace.clone());
    
    // Only allow text files
    policy.allowed_extensions = vec![
        "txt".to_string(),
        "md".to_string(),
        "json".to_string(),
        "csv".to_string(),
    ];
    
    // Deny executable files
    policy.denied_extensions = vec![
        "exe".to_string(),
        "sh".to_string(),
        "bat".to_string(),
    ];
    
    println!("Allowed paths: {:?}", policy.allowed_paths);
    println!("Allowed extensions: {:?}", policy.allowed_extensions);
    println!("Denied extensions: {:?}\n", policy.denied_extensions);
    
    let _server = McpServer::new(policy);
    println!("Server created with extension filtering\n");
}

fn example_custom_policy() {
    println!("Example 4: Custom Policy with Multiple Restrictions");
    println!("====================================================");
    
    let mut policy = AccessPolicy::default();
    
    // Allow multiple directories
    policy.allowed_paths = vec![
        PathBuf::from("/home/user/project1"),
        PathBuf::from("/home/user/project2"),
        PathBuf::from("/var/data/shared"),
    ];
    
    // Deny sensitive subdirectories
    policy.denied_paths = vec![
        PathBuf::from("/home/user/project1/secrets"),
        PathBuf::from("/home/user/project2/.env"),
    ];
    
    // Allow specific extensions
    policy.allowed_extensions = vec![
        "txt".to_string(),
        "md".to_string(),
        "json".to_string(),
        "yaml".to_string(),
    ];
    
    // Set file size limit (2MB)
    policy.max_file_size = 2 * 1024 * 1024;
    
    // Security settings
    policy.allow_symlinks = false;
    policy.allow_hidden_files = false;
    policy.read_only = false;
    
    println!("Allowed paths:");
    for path in &policy.allowed_paths {
        println!("  - {:?}", path);
    }
    println!("\nDenied paths:");
    for path in &policy.denied_paths {
        println!("  - {:?}", path);
    }
    println!("\nAllowed extensions: {:?}", policy.allowed_extensions);
    println!("Max file size: {} bytes", policy.max_file_size);
    println!("Allow symlinks: {}", policy.allow_symlinks);
    println!("Allow hidden files: {}\n", policy.allow_hidden_files);
    
    let _server = McpServer::new(policy);
    println!("Server created with custom policy\n");
}

fn example_config_file() {
    println!("Example 5: Load from Configuration File");
    println!("=========================================");
    
    // Create a configuration
    let workspace = PathBuf::from("/home/user/workspace");
    let mut policy = AccessPolicy::restricted(workspace);
    policy.allowed_extensions = vec!["txt".to_string(), "md".to_string()];
    policy.max_file_size = 5 * 1024 * 1024; // 5MB
    
    let config = Config {
        access_policy: policy,
        server: filejack::ServerConfig {
            name: "MyFileJackServer".to_string(),
            version: "1.0.0".to_string(),
        },
    };
    
    // Save to file (in real usage)
    // config.to_file("filejack.json").unwrap();
    
    println!("Configuration created:");
    println!("Server: {} v{}", config.server.name, config.server.version);
    println!("Allowed paths: {:?}", config.access_policy.allowed_paths);
    println!("Allowed extensions: {:?}", config.access_policy.allowed_extensions);
    println!("Max file size: {} bytes\n", config.access_policy.max_file_size);
    
    // In real usage, load from file:
    // let config = Config::from_file("filejack.json").unwrap();
    let _server = McpServer::new(config.access_policy);
    println!("Server created from configuration\n");
}
