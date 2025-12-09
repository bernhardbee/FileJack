use crate::error::{FileJackError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// FileReader handles reading operations from the filesystem
#[derive(Debug, Clone)]
pub struct FileReader {
    base_path: Option<PathBuf>,
}

impl FileReader {
    /// Create a new FileReader with optional base path restriction
    pub fn new(base_path: Option<PathBuf>) -> Self {
        Self { base_path }
    }

    /// Validate that the path is within allowed bounds
    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        let canonical = path.canonicalize().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FileJackError::FileNotFound(path.display().to_string())
            } else {
                FileJackError::Io(e)
            }
        })?;

        if let Some(ref base) = self.base_path {
            let base_canonical = base.canonicalize()?;
            if !canonical.starts_with(&base_canonical) {
                return Err(FileJackError::PermissionDenied(
                    format!("Path {} is outside allowed directory", path.display())
                ));
            }
        }

        Ok(canonical)
    }

    /// Read file contents as a string
    pub fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        fs::read_to_string(&validated_path).map_err(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    FileJackError::FileNotFound(validated_path.display().to_string())
                }
                std::io::ErrorKind::PermissionDenied => {
                    FileJackError::PermissionDenied(validated_path.display().to_string())
                }
                _ => FileJackError::Io(e),
            }
        })
    }

    /// Read file contents as bytes
    pub fn read_to_bytes<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let validated_path = self.validate_path(path.as_ref())?;
        
        fs::read(&validated_path).map_err(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    FileJackError::FileNotFound(validated_path.display().to_string())
                }
                std::io::ErrorKind::PermissionDenied => {
                    FileJackError::PermissionDenied(validated_path.display().to_string())
                }
                _ => FileJackError::Io(e),
            }
        })
    }

    /// Check if a file exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().exists()
    }
}

/// FileWriter handles writing operations to the filesystem
#[derive(Debug, Clone)]
pub struct FileWriter {
    base_path: Option<PathBuf>,
    create_dirs: bool,
}

impl FileWriter {
    /// Create a new FileWriter with optional base path restriction
    pub fn new(base_path: Option<PathBuf>, create_dirs: bool) -> Self {
        Self {
            base_path,
            create_dirs,
        }
    }

    /// Validate that the path is within allowed bounds
    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // For writing, we need to handle non-existent files
        let parent = path.parent().ok_or_else(|| {
            FileJackError::InvalidPath("Path has no parent directory".to_string())
        })?;

        if let Some(ref base) = self.base_path {
            let base_canonical = base.canonicalize()?;
            
            // If parent exists, canonicalize it
            if parent.exists() {
                let parent_canonical = parent.canonicalize()?;
                if !parent_canonical.starts_with(&base_canonical) {
                    return Err(FileJackError::PermissionDenied(
                        format!("Path {} is outside allowed directory", path.display())
                    ));
                }
            } else {
                // For non-existent parents, check the base path itself
                if !parent.starts_with(base) {
                    return Err(FileJackError::PermissionDenied(
                        format!("Path {} is outside allowed directory", path.display())
                    ));
                }
            }
        }

        Ok(path.to_path_buf())
    }

    /// Write string content to a file
    pub fn write_string<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        let validated_path = self.validate_path(path.as_ref())?;

        if self.create_dirs {
            if let Some(parent) = validated_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(&validated_path, content).map_err(|e| {
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
        })
    }

    /// Write bytes to a file
    pub fn write_bytes<P: AsRef<Path>>(&self, path: P, content: &[u8]) -> Result<()> {
        let validated_path = self.validate_path(path.as_ref())?;

        if self.create_dirs {
            if let Some(parent) = validated_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(&validated_path, content).map_err(|e| {
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
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_reader_new() {
        let reader = FileReader::new(None);
        assert!(reader.base_path.is_none());

        let temp_dir = TempDir::new().unwrap();
        let reader = FileReader::new(Some(temp_dir.path().to_path_buf()));
        assert!(reader.base_path.is_some());
    }

    #[test]
    fn test_file_reader_read_to_string() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let reader = FileReader::new(Some(temp_dir.path().to_path_buf()));
        let content = reader.read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_file_reader_read_to_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");
        let data = vec![0u8, 1, 2, 3, 4];
        fs::write(&file_path, &data).unwrap();

        let reader = FileReader::new(Some(temp_dir.path().to_path_buf()));
        let content = reader.read_to_bytes(&file_path).unwrap();
        assert_eq!(content, data);
    }

    #[test]
    fn test_file_reader_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let reader = FileReader::new(Some(temp_dir.path().to_path_buf()));
        let result = reader.read_to_string(&file_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileJackError::FileNotFound(_)));
    }

    #[test]
    fn test_file_reader_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let reader = FileReader::new(None);
        assert!(reader.exists(&file_path));
        assert!(!reader.exists(temp_dir.path().join("nonexistent.txt")));
    }

    #[test]
    fn test_file_writer_new() {
        let writer = FileWriter::new(None, false);
        assert!(writer.base_path.is_none());
        assert!(!writer.create_dirs);

        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(Some(temp_dir.path().to_path_buf()), true);
        assert!(writer.base_path.is_some());
        assert!(writer.create_dirs);
    }

    #[test]
    fn test_file_writer_write_string() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");

        let writer = FileWriter::new(Some(temp_dir.path().to_path_buf()), false);
        writer.write_string(&file_path, "Test content").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[test]
    fn test_file_writer_write_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.bin");
        let data = vec![10u8, 20, 30, 40];

        let writer = FileWriter::new(Some(temp_dir.path().to_path_buf()), false);
        writer.write_bytes(&file_path, &data).unwrap();

        let content = fs::read(&file_path).unwrap();
        assert_eq!(content, data);
    }

    #[test]
    fn test_file_writer_create_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("subdir").join("output.txt");

        let writer = FileWriter::new(Some(temp_dir.path().to_path_buf()), true);
        writer.write_string(&file_path, "Nested content").unwrap();

        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Nested content");
    }

    #[test]
    fn test_file_writer_append_string() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("append.txt");

        let writer = FileWriter::new(Some(temp_dir.path().to_path_buf()), false);
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

        let writer = FileWriter::new(Some(temp_dir.path().to_path_buf()), false);
        let result = writer.write_string(&file_path, "Should fail");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_reader_permission_boundary() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_file = temp_dir.path().join("allowed.txt");
        fs::write(&allowed_file, "allowed content").unwrap();

        let reader = FileReader::new(Some(temp_dir.path().to_path_buf()));
        
        // Should succeed - file is within base path
        assert!(reader.read_to_string(&allowed_file).is_ok());
    }

    #[test]
    fn test_file_writer_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("overwrite.txt");

        let writer = FileWriter::new(Some(temp_dir.path().to_path_buf()), false);
        writer.write_string(&file_path, "Original").unwrap();
        writer.write_string(&file_path, "Overwritten").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Overwritten");
    }
}
