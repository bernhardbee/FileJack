use crate::access_control::AccessPolicy;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for FileJack server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Access control policy
    pub access_policy: AccessPolicy,
    
    /// Server settings
    #[serde(default)]
    pub server: ServerConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name
    #[serde(default = "default_server_name")]
    pub name: String,
    
    /// Server version
    #[serde(default = "default_server_version")]
    pub version: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: default_server_name(),
            version: default_server_version(),
        }
    }
}

fn default_server_name() -> String {
    "FileJack".to_string()
}

fn default_server_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl Config {
    /// Load configuration from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a JSON file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path.as_ref(), json)?;
        Ok(())
    }

    /// Create a default configuration with restricted access to a single directory
    pub fn default_restricted(allowed_path: PathBuf) -> Self {
        Self {
            access_policy: AccessPolicy::restricted(allowed_path),
            server: ServerConfig::default(),
        }
    }

    /// Create a permissive configuration (allows all access)
    pub fn permissive() -> Self {
        Self {
            access_policy: AccessPolicy::permissive(),
            server: ServerConfig::default(),
        }
    }

    /// Create a read-only configuration
    pub fn read_only(allowed_path: PathBuf) -> Self {
        Self {
            access_policy: AccessPolicy::read_only(allowed_path),
            server: ServerConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_server_config() {
        let config = ServerConfig::default();
        assert_eq!(config.name, "FileJack");
        assert!(!config.version.is_empty());
    }

    #[test]
    fn test_config_default_restricted() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default_restricted(temp_dir.path().to_path_buf());
        
        assert_eq!(config.access_policy.allowed_paths.len(), 1);
        assert!(!config.access_policy.read_only);
    }

    #[test]
    fn test_config_permissive() {
        let config = Config::permissive();
        
        assert!(config.access_policy.allow_symlinks);
        assert!(config.access_policy.allow_hidden_files);
    }

    #[test]
    fn test_config_read_only() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::read_only(temp_dir.path().to_path_buf());
        
        assert!(config.access_policy.read_only);
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let original_config = Config::default_restricted(temp_dir.path().to_path_buf());
        original_config.to_file(&config_path).unwrap();
        
        let loaded_config = Config::from_file(&config_path).unwrap();
        
        assert_eq!(loaded_config.server.name, original_config.server.name);
        assert_eq!(
            loaded_config.access_policy.allowed_paths.len(),
            original_config.access_policy.allowed_paths.len()
        );
    }

    #[test]
    fn test_config_with_custom_settings() {
        let temp_dir = TempDir::new().unwrap();
        let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        policy.allowed_extensions = vec!["txt".to_string(), "json".to_string()];
        policy.max_file_size = 5 * 1024 * 1024; // 5MB
        
        let config = Config {
            access_policy: policy,
            server: ServerConfig::default(),
        };
        
        assert_eq!(config.access_policy.allowed_extensions.len(), 2);
        assert_eq!(config.access_policy.max_file_size, 5 * 1024 * 1024);
    }

    #[test]
    fn test_config_json_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default_restricted(temp_dir.path().to_path_buf());
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.server.name, config.server.name);
    }
}
