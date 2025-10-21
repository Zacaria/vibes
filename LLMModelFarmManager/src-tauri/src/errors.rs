use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

pub type AppResult<T> = Result<T, AppError>;
