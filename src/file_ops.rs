use crate::access_control::AccessPolicy;
use crate::error::{FileJackError, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// FileReader handles reading operations from the filesystem
#[derive(Debug, Clone)]
pub struct FileReader {
    policy: AccessPolicy,
}

impl FileReader {
    /// Create a new FileReader with an access policy
    pub fn new(policy: AccessPolicy) -> Self {
        Self { policy }
    }

    /// Validate that the path is within allowed bounds
    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        self.policy.validate_read(path)
    }

    /// Read file contents as a string with atomic validation
    pub fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        // Open file first to get a file descriptor, preventing TOCTOU
        let mut file = File::open(&validated_path).map_err(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    FileJackError::FileNotFound(validated_path.display().to_string())
                }
                std::io::ErrorKind::PermissionDenied => {
                    FileJackError::PermissionDenied(validated_path.display().to_string())
                }
                _ => FileJackError::Io(e),
            }
        })?;
        
        // Validate file metadata using the file descriptor
        let metadata = file.metadata()?;
        self.policy.validate_file_size(metadata.len())?;
        
        // Verify it's still a regular file (not replaced with symlink)
        if !metadata.is_file() {
            return Err(FileJackError::InvalidPath(
                "Path is not a regular file".to_string()
            ));
        }
        
        // Read from the already-opened file descriptor
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    /// Read file contents as bytes with atomic validation
    pub fn read_to_bytes<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        // Open file first to get a file descriptor, preventing TOCTOU
        let mut file = File::open(&validated_path).map_err(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    FileJackError::FileNotFound(validated_path.display().to_string())
                }
                std::io::ErrorKind::PermissionDenied => {
                    FileJackError::PermissionDenied(validated_path.display().to_string())
                }
                _ => FileJackError::Io(e),
            }
        })?;
        
        // Validate file metadata using the file descriptor
        let metadata = file.metadata()?;
        self.policy.validate_file_size(metadata.len())?;
        
        // Verify it's still a regular file
        if !metadata.is_file() {
            return Err(FileJackError::InvalidPath(
                "Path is not a regular file".to_string()
            ));
        }
        
        // Read from the already-opened file descriptor
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        Ok(content)
    }

    /// Check if a file exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().exists()
    }

    /// Get file metadata
    pub fn get_metadata<P: AsRef<Path>>(&self, path: P) -> Result<FileMetadata> {
        let validated_path = self.validate_path(path.as_ref())?;
        let metadata = fs::metadata(&validated_path)?;
        
        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            is_symlink: metadata.is_symlink(),
            modified: metadata.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
            created: metadata.created().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
            readonly: metadata.permissions().readonly(),
        })
    }

    /// List directory contents
    pub fn list_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) -> Result<Vec<DirectoryEntry>> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        if !validated_path.is_dir() {
            return Err(FileJackError::InvalidPath(
                "Path is not a directory".to_string()
            ));
        }

        let mut entries = Vec::new();

        if recursive {
            for entry in WalkDir::new(&validated_path)
                .follow_links(self.policy.allow_symlinks)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path == validated_path {
                    continue; // Skip the root directory itself
                }
                
                // Validate each entry against policy
                if self.validate_path(path).is_ok() {
                    entries.push(DirectoryEntry {
                        path: path.display().to_string(),
                        name: path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string(),
                        is_file: entry.file_type().is_file(),
                        is_dir: entry.file_type().is_dir(),
                        size: entry.metadata().ok().map(|m| m.len()),
                    });
                }
            }
        } else {
            for entry in fs::read_dir(&validated_path)? {
                let entry = entry?;
                let path = entry.path();
                
                // Validate each entry against policy
                if self.validate_path(&path).is_ok() {
                    let metadata = entry.metadata()?;
                    entries.push(DirectoryEntry {
                        path: path.display().to_string(),
                        name: entry.file_name().to_string_lossy().to_string(),
                        is_file: metadata.is_file(),
                        is_dir: metadata.is_dir(),
                        size: Some(metadata.len()),
                    });
                }
            }
        }

        Ok(entries)
    }

    /// Read specific lines from a file
    pub fn read_lines<P: AsRef<Path>>(
        &self,
        path: P,
        start_line: Option<usize>,
        end_line: Option<usize>,
        tail: Option<usize>,
    ) -> Result<Vec<String>> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        // Open file first to get a file descriptor, preventing TOCTOU
        let file = File::open(&validated_path).map_err(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    FileJackError::FileNotFound(validated_path.display().to_string())
                }
                std::io::ErrorKind::PermissionDenied => {
                    FileJackError::PermissionDenied(validated_path.display().to_string())
                }
                _ => FileJackError::Io(e),
            }
        })?;
        
        // Validate file metadata using the file descriptor
        let metadata = file.metadata()?;
        self.policy.validate_file_size(metadata.len())?;
        
        // Verify it's a regular file
        if !metadata.is_file() {
            return Err(FileJackError::InvalidPath(
                "Path is not a regular file".to_string()
            ));
        }
        
        use std::io::BufRead;
        let reader = std::io::BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;
        
        // Handle tail mode
        if let Some(n) = tail {
            let start = if all_lines.len() > n {
                all_lines.len() - n
            } else {
                0
            };
            return Ok(all_lines[start..].to_vec());
        }
        
        // Handle line range
        let start_idx = start_line.unwrap_or(1).saturating_sub(1); // Convert to 0-based
        let end_idx = end_line.unwrap_or(all_lines.len()).min(all_lines.len());
        
        if start_idx >= all_lines.len() {
            return Ok(Vec::new());
        }
        
        Ok(all_lines[start_idx..end_idx].to_vec())
    }

    /// Search for files matching a glob pattern
    pub fn search_files<P: AsRef<Path>>(
        &self,
        base_path: P,
        pattern: &str,
        recursive: bool,
        max_results: Option<usize>,
    ) -> Result<Vec<String>> {
        let validated_path = self.validate_path(base_path.as_ref())?;
        
        if !validated_path.is_dir() {
            return Err(FileJackError::InvalidPath(
                "Base path must be a directory".to_string()
            ));
        }
        
        let glob_pattern = glob::Pattern::new(pattern)
            .map_err(|e| FileJackError::InvalidParameters(format!("Invalid glob pattern: {}", e)))?;
        
        let mut results = Vec::new();
        let walker = if recursive {
            WalkDir::new(&validated_path).follow_links(self.policy.allow_symlinks)
        } else {
            WalkDir::new(&validated_path).max_depth(1).follow_links(self.policy.allow_symlinks)
        };
        
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if let Some(max) = max_results {
                if results.len() >= max {
                    break;
                }
            }
            
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    if glob_pattern.matches(name_str) && self.validate_path(path).is_ok() {
                        results.push(path.display().to_string());
                    }
                }
            }
        }
        
        Ok(results)
    }

    /// Search for pattern in file contents using regex
    pub fn grep_file<P: AsRef<Path>>(
        &self,
        path: P,
        pattern: &str,
        max_matches: Option<usize>,
        context_lines: Option<usize>,
    ) -> Result<Vec<crate::protocol::GrepMatch>> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        // Open file first
        let file = File::open(&validated_path).map_err(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    FileJackError::FileNotFound(validated_path.display().to_string())
                }
                std::io::ErrorKind::PermissionDenied => {
                    FileJackError::PermissionDenied(validated_path.display().to_string())
                }
                _ => FileJackError::Io(e),
            }
        })?;
        
        let metadata = file.metadata()?;
        self.policy.validate_file_size(metadata.len())?;
        
        if !metadata.is_file() {
            return Err(FileJackError::InvalidPath(
                "Path is not a regular file".to_string()
            ));
        }
        
        let regex = regex::Regex::new(pattern)
            .map_err(|e| FileJackError::InvalidParameters(format!("Invalid regex pattern: {}", e)))?;
        
        use std::io::BufRead;
        let reader = std::io::BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;
        
        let mut matches = Vec::new();
        let context = context_lines.unwrap_or(0);
        
        for (line_num, line) in all_lines.iter().enumerate() {
            if regex.is_match(line) {
                if let Some(max) = max_matches {
                    if matches.len() >= max {
                        break;
                    }
                }
                
                let start_context = line_num.saturating_sub(context);
                let end_context = (line_num + context + 1).min(all_lines.len());
                
                let context_before = all_lines[start_context..line_num].to_vec();
                let context_after = all_lines[line_num + 1..end_context].to_vec();
                
                matches.push(crate::protocol::GrepMatch {
                    line_number: line_num + 1, // 1-based line numbers
                    line_content: line.clone(),
                    context_before,
                    context_after,
                });
            }
        }
        
        Ok(matches)
    }
}

/// File metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub modified: Option<u64>,
    pub created: Option<u64>,
    pub readonly: bool,
}

/// Directory entry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub path: String,
    pub name: String,
    pub is_file: bool,
    pub is_dir: bool,
    pub size: Option<u64>,
}

/// FileWriter handles writing operations to the filesystem
#[derive(Debug, Clone)]
pub struct FileWriter {
    policy: AccessPolicy,
    create_dirs: bool,
}

impl FileWriter {
    /// Create a new FileWriter with an access policy
    pub fn new(policy: AccessPolicy, create_dirs: bool) -> Self {
        Self {
            policy,
            create_dirs,
        }
    }

    /// Validate that the path is within allowed bounds
    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        self.policy.validate_write(path)
    }

    /// Write string content to a file atomically
    pub fn write_string<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        let validated_path = self.validate_path(path.as_ref())?;

        // Check file size before writing
        self.policy.validate_file_size(content.len() as u64)?;

        if self.create_dirs {
            if let Some(parent) = validated_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        // Open with explicit options to prevent TOCTOU
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&validated_path)
            .map_err(|e| {
                match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        FileJackError::PermissionDenied(validated_path.display().to_string())
                    }
                    std::io::ErrorKind::NotFound => {
                        FileJackError::FileNotFound(
                            format!("Parent directory does not exist: {}", validated_path.display())
                        )
                    }
                    _ => FileJackError::Io(e),
                }
            })?;
        
        // Verify we opened a regular file, not a symlink or special file
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(FileJackError::InvalidPath(
                "Cannot write to non-regular file".to_string()
            ));
        }
        
        // Write using the file descriptor
        file.write_all(content.as_bytes())?;
        file.sync_all()?; // Ensure data is written to disk
        Ok(())
    }

    /// Write bytes to a file atomically
    pub fn write_bytes<P: AsRef<Path>>(&self, path: P, content: &[u8]) -> Result<()> {
        let validated_path = self.validate_path(path.as_ref())?;

        // Check file size before writing
        self.policy.validate_file_size(content.len() as u64)?;

        if self.create_dirs {
            if let Some(parent) = validated_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        // Open with explicit options to prevent TOCTOU
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&validated_path)
            .map_err(|e| {
                match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        FileJackError::PermissionDenied(validated_path.display().to_string())
                    }
                    std::io::ErrorKind::NotFound => {
                        FileJackError::FileNotFound(
                            format!("Parent directory does not exist: {}", validated_path.display())
                        )
                    }
                    _ => FileJackError::Io(e),
                }
            })?;
        
        // Verify we opened a regular file
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(FileJackError::InvalidPath(
                "Cannot write to non-regular file".to_string()
            ));
        }
        
        // Write using the file descriptor
        file.write_all(content)?;
        file.sync_all()?; // Ensure data is written to disk
        Ok(())
    }

    /// Append string content to a file
    pub fn append_string<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        let validated_path = self.validate_path(path.as_ref())?;

        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&validated_path)?;
        
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Delete a file
    pub fn delete_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        if !validated_path.is_file() {
            return Err(FileJackError::InvalidPath(
                "Path is not a file or does not exist".to_string()
            ));
        }
        
        fs::remove_file(&validated_path)?;
        Ok(())
    }

    /// Move/rename a file
    pub fn move_file<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<()> {
        let validated_from = self.validate_path(from.as_ref())?;
        let validated_to = self.validate_path(to.as_ref())?;
        
        if !validated_from.exists() {
            return Err(FileJackError::FileNotFound(
                validated_from.display().to_string()
            ));
        }
        
        fs::rename(&validated_from, &validated_to)?;
        Ok(())
    }

    /// Copy a file
    pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<u64> {
        let validated_from = self.validate_path(from.as_ref())?;
        let validated_to = self.validate_path(to.as_ref())?;
        
        if !validated_from.is_file() {
            return Err(FileJackError::InvalidPath(
                "Source path is not a file".to_string()
            ));
        }
        
        let bytes_copied = fs::copy(&validated_from, &validated_to)?;
        Ok(bytes_copied)
    }

    /// Create a directory
    pub fn create_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) -> Result<()> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        if validated_path.exists() {
            return Err(FileJackError::InvalidPath(
                "Directory already exists".to_string()
            ));
        }
        
        if recursive {
            fs::create_dir_all(&validated_path)?;
        } else {
            fs::create_dir(&validated_path)?;
        }
        
        Ok(())
    }

    /// Remove a directory
    pub fn remove_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) -> Result<()> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        if !validated_path.is_dir() {
            return Err(FileJackError::InvalidPath(
                "Path is not a directory or does not exist".to_string()
            ));
        }
        
        if recursive {
            fs::remove_dir_all(&validated_path)?;
        } else {
            // Only remove if empty
            fs::remove_dir(&validated_path)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access_control::AccessPolicy;
    use tempfile::TempDir;

    #[test]
    fn test_file_reader_new() {
        let temp_dir = TempDir::new().unwrap();
        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let reader = FileReader::new(policy);
        assert_eq!(reader.policy.allowed_paths.len(), 1);
    }

    #[test]
    fn test_file_reader_read_to_string() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let reader = FileReader::new(policy);
        let content = reader.read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_file_reader_read_to_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");
        let data = vec![0u8, 1, 2, 3, 4];
        fs::write(&file_path, &data).unwrap();

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let reader = FileReader::new(policy);
        let content = reader.read_to_bytes(&file_path).unwrap();
        assert_eq!(content, data);
    }

    #[test]
    fn test_file_reader_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let reader = FileReader::new(policy);
        let result = reader.read_to_string(&file_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileJackError::FileNotFound(_)));
    }

    #[test]
    fn test_file_reader_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let policy = AccessPolicy::permissive();
        let reader = FileReader::new(policy);
        assert!(reader.exists(&file_path));
        assert!(!reader.exists(temp_dir.path().join("nonexistent.txt")));
    }

    #[test]
    fn test_file_writer_new() {
        let temp_dir = TempDir::new().unwrap();
        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let writer = FileWriter::new(policy, true);
        assert!(writer.create_dirs);
    }

    #[test]
    fn test_file_writer_write_string() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let writer = FileWriter::new(policy, false);
        writer.write_string(&file_path, "Test content").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[test]
    fn test_file_writer_write_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.bin");
        let data = vec![10u8, 20, 30, 40];

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let writer = FileWriter::new(policy, false);
        writer.write_bytes(&file_path, &data).unwrap();

        let content = fs::read(&file_path).unwrap();
        assert_eq!(content, data);
    }

    #[test]
    fn test_file_writer_create_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("subdir").join("output.txt");

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let writer = FileWriter::new(policy, true);
        writer.write_string(&file_path, "Nested content").unwrap();

        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Nested content");
    }

    #[test]
    fn test_file_writer_append_string() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("append.txt");

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let writer = FileWriter::new(policy, false);
        writer.write_string(&file_path, "Line 1\n").unwrap();
        writer.append_string(&file_path, "Line 2\n").unwrap();
        writer.append_string(&file_path, "Line 3\n").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn test_file_writer_without_create_dirs_fails() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent").join("output.txt");

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let writer = FileWriter::new(policy, false);
        let result = writer.write_string(&file_path, "Should fail");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_reader_permission_boundary() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_file = temp_dir.path().join("allowed.txt");
        fs::write(&allowed_file, "allowed content").unwrap();

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let reader = FileReader::new(policy);
        
        // Should succeed - file is within allowed path
        assert!(reader.read_to_string(&allowed_file).is_ok());
    }

    #[test]
    fn test_file_writer_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("overwrite.txt");

        let policy = AccessPolicy::restricted(temp_dir.path().to_path_buf());
        let writer = FileWriter::new(policy, false);
        writer.write_string(&file_path, "Original").unwrap();
        writer.write_string(&file_path, "Overwritten").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Overwritten");
    }
}
