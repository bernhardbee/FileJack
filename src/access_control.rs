use crate::error::{FileJackError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Access control policy for filesystem operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    /// List of allowed directories (whitelist)
    pub allowed_paths: Vec<PathBuf>,
    
    /// List of explicitly denied paths (blacklist, takes precedence)
    pub denied_paths: Vec<PathBuf>,
    
    /// List of allowed file extensions (e.g., ["txt", "md", "json"])
    /// Empty means all extensions are allowed
    pub allowed_extensions: Vec<String>,
    
    /// List of denied file extensions (takes precedence over allowed)
    pub denied_extensions: Vec<String>,
    
    /// Maximum file size in bytes (0 means no limit)
    pub max_file_size: u64,
    
    /// Whether symbolic links are allowed
    pub allow_symlinks: bool,
    
    /// Whether hidden files (starting with .) are allowed
    pub allow_hidden_files: bool,
    
    /// Read-only mode (no write operations allowed)
    pub read_only: bool,
}

impl Default for AccessPolicy {
    fn default() -> Self {
        Self {
            allowed_paths: vec![],
            denied_paths: vec![],
            allowed_extensions: vec![],
            denied_extensions: vec![],
            max_file_size: 0,
            allow_symlinks: false,
            allow_hidden_files: false,
            read_only: false,
        }
    }
}

impl AccessPolicy {
    /// Create a new permissive policy (allows everything)
    pub fn permissive() -> Self {
        Self {
            allowed_paths: vec![],
            denied_paths: vec![],
            allowed_extensions: vec![],
            denied_extensions: vec![],
            max_file_size: 0,
            allow_symlinks: true,
            allow_hidden_files: true,
            read_only: false,
        }
    }

    /// Create a restrictive policy with a single allowed directory
    pub fn restricted(allowed_path: PathBuf) -> Self {
        Self {
            allowed_paths: vec![allowed_path],
            denied_paths: vec![],
            allowed_extensions: vec![],
            denied_extensions: vec![],
            max_file_size: 10 * 1024 * 1024, // 10MB default
            allow_symlinks: false,
            allow_hidden_files: false,
            read_only: false,
        }
    }

    /// Create a read-only policy
    pub fn read_only(allowed_path: PathBuf) -> Self {
        let mut policy = Self::restricted(allowed_path);
        policy.read_only = true;
        policy
    }

    /// Validate a path for read access
    pub fn validate_read(&self, path: &Path) -> Result<PathBuf> {
        let canonical = self.canonicalize_path(path)?;
        
        // Check if path is denied
        self.check_denied_paths(&canonical)?;
        
        // Check if path is in allowed directories
        self.check_allowed_paths(&canonical)?;
        
        // Check file extension
        self.check_extension(&canonical)?;
        
        // Check hidden files
        self.check_hidden_files(&canonical)?;
        
        // Check symlinks
        self.check_symlinks(path, &canonical)?;
        
        Ok(canonical)
    }

    /// Validate a path for write access
    pub fn validate_write(&self, path: &Path) -> Result<PathBuf> {
        // Check read-only mode
        if self.read_only {
            return Err(FileJackError::PermissionDenied(
                "Write operations are disabled in read-only mode".to_string()
            ));
        }

        // For write operations, we need to handle non-existent files
        // Find the first existing ancestor directory
        let mut path_to_check = path.to_path_buf();
        while !path_to_check.exists() {
            path_to_check = match path_to_check.parent() {
                Some(parent) => parent.to_path_buf(),
                None => return Err(FileJackError::InvalidPath(
                    "Cannot find existing ancestor directory".to_string()
                )),
            };
        }

        let canonical = self.canonicalize_path(&path_to_check)?;
        
        // Check if path is denied
        self.check_denied_paths(&canonical)?;
        
        // Check if path is in allowed directories
        self.check_allowed_paths(&canonical)?;
        
        // Check file extension
        self.check_extension(path)?;
        
        // Check hidden files
        self.check_hidden_files(path)?;
        
        Ok(path.to_path_buf())
    }

    /// Validate file size
    pub fn validate_file_size(&self, size: u64) -> Result<()> {
        if self.max_file_size > 0 && size > self.max_file_size {
            return Err(FileJackError::PermissionDenied(
                format!("File size {} exceeds maximum allowed size {}", size, self.max_file_size)
            ));
        }
        Ok(())
    }

    fn canonicalize_path(&self, path: &Path) -> Result<PathBuf> {
        path.canonicalize().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FileJackError::FileNotFound(path.display().to_string())
            } else {
                FileJackError::Io(e)
            }
        })
    }

    fn check_denied_paths(&self, canonical: &Path) -> Result<()> {
        for denied in &self.denied_paths {
            if let Ok(denied_canonical) = denied.canonicalize() {
                if canonical.starts_with(&denied_canonical) || canonical == denied_canonical {
                    return Err(FileJackError::PermissionDenied(
                        format!("Access to {} is explicitly denied", canonical.display())
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_allowed_paths(&self, canonical: &Path) -> Result<()> {
        // If allowed_paths is empty, all paths are allowed (unless denied)
        if self.allowed_paths.is_empty() {
            return Ok(());
        }

        for allowed in &self.allowed_paths {
            if let Ok(allowed_canonical) = allowed.canonicalize() {
                if canonical.starts_with(&allowed_canonical) || canonical == allowed_canonical {
                    return Ok(());
                }
            }
        }

        Err(FileJackError::PermissionDenied(
            format!("Path {} is not in any allowed directory", canonical.display())
        ))
    }

    fn check_extension(&self, path: &Path) -> Result<()> {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            
            // Check denied extensions first
            if !self.denied_extensions.is_empty() {
                for denied_ext in &self.denied_extensions {
                    if ext_str == denied_ext.to_lowercase() {
                        return Err(FileJackError::PermissionDenied(
                            format!("File extension .{} is not allowed", ext_str)
                        ));
                    }
                }
            }
            
            // Check allowed extensions
            if !self.allowed_extensions.is_empty() {
                let allowed = self.allowed_extensions.iter()
                    .any(|allowed_ext| ext_str == allowed_ext.to_lowercase());
                
                if !allowed {
                    return Err(FileJackError::PermissionDenied(
                        format!("File extension .{} is not in allowed extensions", ext_str)
                    ));
                }
            }
        } else if !self.allowed_extensions.is_empty() {
            // File has no extension but allowed_extensions is specified
            return Err(FileJackError::PermissionDenied(
                "Files without extensions are not allowed".to_string()
            ));
        }
        
        Ok(())
    }

    fn check_hidden_files(&self, path: &Path) -> Result<()> {
        if !self.allow_hidden_files {
            if let Some(filename) = path.file_name() {
                if filename.to_string_lossy().starts_with('.') {
                    return Err(FileJackError::PermissionDenied(
                        "Access to hidden files is not allowed".to_string()
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_symlinks(&self, original: &Path, canonical: &Path) -> Result<()> {
        if !self.allow_symlinks && original != canonical {
            // Path was resolved from a symlink
            if original.read_link().is_ok() {
                return Err(FileJackError::PermissionDenied(
                    "Symbolic links are not allowed".to_string()
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_default_policy() {
        let policy = AccessPolicy::default();
        assert!(!policy.allow_symlinks);
        assert!(!policy.allow_hidden_files);
        assert!(!policy.read_only);
    }

    #[test]
    fn test_permissive_policy() {
        let policy = AccessPolicy::permissive();
        assert!(policy.allow_symlinks);
        assert!(policy.allow_hidden_files);
        assert!(!policy.read_only);
    }

    #[test]
    fn test_restricted_policy() {
        let temp_dir = TempDir::new().unwrap();
        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        
        assert_eq!(policy.allowed_paths.len(), 1);
        assert!(!policy.allow_symlinks);
        assert!(!policy.allow_hidden_files);
        assert_eq!(policy.max_file_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_read_only_policy() {
        let temp_dir = TempDir::new().unwrap();
        let policy = AccessPolicy::read_only(temp_dir.path().to_path_buf());
        
        assert!(policy.read_only);
    }

    #[test]
    fn test_validate_read_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        assert!(policy.validate_read(&test_file).is_ok());
    }

    #[test]
    fn test_validate_read_denied_path() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path().join("allowed");
        fs::create_dir(&allowed_dir).unwrap();
        
        let denied_dir = temp_dir.path().join("denied");
        fs::create_dir(&denied_dir).unwrap();
        let denied_file = denied_dir.join("secret.txt");
        fs::write(&denied_file, "secret").unwrap();

        let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        policy.denied_paths = vec![denied_dir];

        assert!(policy.validate_read(&denied_file).is_err());
    }

    #[test]
    fn test_validate_write_read_only() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        let policy = AccessPolicy::read_only(temp_dir.path().to_path_buf());
        assert!(policy.validate_write(&test_file).is_err());
    }

    #[test]
    fn test_allowed_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let txt_file = temp_dir.path().join("test.txt");
        let exe_file = temp_dir.path().join("test.exe");
        fs::write(&txt_file, "test").unwrap();
        fs::write(&exe_file, "test").unwrap();

        let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        policy.allowed_extensions = vec!["txt".to_string(), "md".to_string()];

        assert!(policy.validate_read(&txt_file).is_ok());
        assert!(policy.validate_read(&exe_file).is_err());
    }

    #[test]
    fn test_denied_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let txt_file = temp_dir.path().join("test.txt");
        let exe_file = temp_dir.path().join("test.exe");
        fs::write(&txt_file, "test").unwrap();
        fs::write(&exe_file, "test").unwrap();

        let mut policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        policy.denied_extensions = vec!["exe".to_string(), "sh".to_string()];

        assert!(policy.validate_read(&txt_file).is_ok());
        assert!(policy.validate_read(&exe_file).is_err());
    }

    #[test]
    fn test_hidden_files() {
        let temp_dir = TempDir::new().unwrap();
        let visible_file = temp_dir.path().join("visible.txt");
        let hidden_file = temp_dir.path().join(".hidden.txt");
        fs::write(&visible_file, "test").unwrap();
        fs::write(&hidden_file, "test").unwrap();

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());

        assert!(policy.validate_read(&visible_file).is_ok());
        assert!(policy.validate_read(&hidden_file).is_err());
    }

    #[test]
    fn test_file_size_validation() {
        let mut policy = AccessPolicy::default();
        policy.max_file_size = 1024; // 1KB

        assert!(policy.validate_file_size(500).is_ok());
        assert!(policy.validate_file_size(1024).is_ok());
        assert!(policy.validate_file_size(2048).is_err());
    }

    #[test]
    fn test_path_outside_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path().join("allowed");
        fs::create_dir(&allowed_dir).unwrap();
        
        let outside_dir = temp_dir.path().join("outside");
        fs::create_dir(&outside_dir).unwrap();
        let outside_file = outside_dir.join("test.txt");
        fs::write(&outside_file, "test").unwrap();

        let policy = AccessPolicy::restricted(allowed_dir);
        assert!(policy.validate_read(&outside_file).is_err());
    }
}
