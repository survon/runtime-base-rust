use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug)]
pub enum SurvonError {
    #[error("Module error: {0}")]
    ModuleError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Zip error: {0}")]
    ZipError(#[from] ZipError),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

}

impl From<&str> for SurvonError {
    fn from(error: &str) -> Self {
        SurvonError::RuntimeError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SurvonError>;
