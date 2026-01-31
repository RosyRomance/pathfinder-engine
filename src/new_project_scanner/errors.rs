use thiserror::Error;


#[derive(Debug, Error)]
pub enum ScanError {
    #[error("provider error: {0}")]
    Provider(String),

    #[error("invalid config: {0}")]
    Config(String),

    #[error("store error: {0}")]
    Store(String),

    #[error("internal error: {0}")]
    Internal(String),
}