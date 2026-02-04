use thiserror::Error;

#[derive(Error, Debug)]
pub enum Lib3mfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(u32),

    #[error("Invalid 3MF structure: {0}")]
    InvalidStructure(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Feature not enabled: {0}. Rebuild with `cargo build --features {1}`")]
    FeatureNotEnabled(String, String),
}

pub type Result<T> = std::result::Result<T, Lib3mfError>;
