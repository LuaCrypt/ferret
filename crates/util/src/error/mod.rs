use thiserror::Error;

pub type Result<T> = std::result::Result<T, FerretError>;

#[derive(Debug, Error)]
pub enum FerretError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unsupported lua: {0}")]
    Unsupported(String),
    #[error("compile error: {0}")]
    Compile(String),
    #[error("io error: {0}")]
    Io(String),
}
