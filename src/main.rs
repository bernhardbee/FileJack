use filejack::{AccessPolicy, Config, McpServer};
use serde_json::json;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();
    
    info!("FileJack MCP Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Starting server...");

    // Try to load config file first
    let config_path = std::env::var("FILEJACK_CONFIG")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            // Try default config locations
            let current_dir = std::env::current_dir().ok()?;
            let config_file = current_dir.join("filejack.json");
            if config_file.exists() {
                Some(config_file)
            } else {
                None
            }
        });

    let server = if let Some(config_path) = config_path {
        info!("Loading configuration from: {}", config_path.display());
        match Config::from_file(&config_path) {
            Ok(config) => {
                info!("Configuration loaded successfully");
                info!("Server: {} v{}", config.server.name, config.server.version);
                
                // Log policy details
                if !config.access_policy.allowed_paths.is_empty() {
                    info!("Allowed paths:");
                    for path in &config.access_policy.allowed_paths {
                        info!("  - {}", path.display());
                    }
                }
                
                if !config.access_policy.denied_paths.is_empty() {
                    warn!("Denied paths:");
                    for path in &config.access_policy.denied_paths {
                        warn!("  - {}", path.display());
                    }
                }
                
                if !config.access_policy.allowed_extensions.is_empty() {
                    info!("Allowed extensions: {:?}", config.access_policy.allowed_extensions);
                }
                
                if !config.access_policy.denied_extensions.is_empty() {
                    warn!("Denied extensions: {:?}", config.access_policy.denied_extensions);
                }
                
                if config.access_policy.max_file_size > 0 {
                    info!("Max file size: {} bytes", config.access_policy.max_file_size);
                }
                
                info!("Read-only mode: {}", config.access_policy.read_only);
                info!("Allow symlinks: {}", config.access_policy.allow_symlinks);
                info!("Allow hidden files: {}", config.access_policy.allow_hidden_files);
                
                McpServer::new(config.access_policy)
            }
            Err(e) => {
                error!("Error loading config file: {}", e);
                warn!("Falling back to environment-based configuration");
                create_server_from_env()
            }
        }
    } else {
        info!("No config file found, using environment-based configuration");
        create_server_from_env()
    };
                
    info!("Server initialized. Waiting for JSON-RPC requests on stdin...");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        match line {
            Ok(request_str) => {
                if request_str.trim().is_empty() {
                    continue;
                }

                debug!("Received request: {}", request_str);
                
                let response_str = server.process_request(&request_str);
                
                debug!("Sending response: {}", response_str);
                
                if let Err(e) = writeln!(stdout, "{}", response_str) {
                    error!("Error writing response: {}", e);
                    break;
                }
                
                if let Err(e) = stdout.flush() {
                    error!("Error flushing stdout: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": format!("Failed to read input: {}", e)
                    },
                    "id": null
                });
                
                if let Err(e) = writeln!(stdout, "{}", error_response) {
                    error!("Error writing error response: {}", e);
                }
                break;
            }
        }
    }

    info!("Server shutting down...");
}

fn create_server_from_env() -> McpServer {
    // Get base path from environment or use current directory
    let base_path = std::env::var("FILEJACK_BASE_PATH")
        .ok()
        .map(PathBuf::from);

    // Check for read-only mode
    let read_only = std::env::var("FILEJACK_READ_ONLY")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    if let Some(base_path) = base_path {
        info!("Base path: {}", base_path.display());
        info!("Read-only mode: {}", read_only);
        
        let policy = if read_only {
            AccessPolicy::read_only(base_path)
        } else {
            AccessPolicy::restricted(base_path)
        };
        
        McpServer::new(policy)
    } else {
        warn!("Base path: unrestricted (permissive mode)");
        McpServer::new(AccessPolicy::permissive())
    }
}
