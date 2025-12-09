use thiserror::Error;

/// Errors that can occur in FileJack operations
#[derive(Error, Debug)]
pub enum FileJackError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
}

/// Result type alias for FileJack operations
pub type Result<T> = std::result::Result<T, FileJackError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = FileJackError::FileNotFound("test.txt".to_string());
        assert_eq!(err.to_string(), "File not found: test.txt");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: FileJackError = io_err.into();
        assert!(matches!(err, FileJackError::Io(_)));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json")
            .unwrap_err();
        let err: FileJackError = json_err.into();
        assert!(matches!(err, FileJackError::Json(_)));
    }

    #[test]
    fn test_error_types() {
        let errors = vec![
            FileJackError::FileNotFound("test".to_string()),
            FileJackError::PermissionDenied("test".to_string()),
            FileJackError::InvalidPath("test".to_string()),
            FileJackError::ProtocolError("test".to_string()),
            FileJackError::ToolNotFound("test".to_string()),
            FileJackError::InvalidParameters("test".to_string()),
        ];

        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }
}
