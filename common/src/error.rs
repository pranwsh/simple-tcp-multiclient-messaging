use thiserror::Error;
use bincode::error::{DecodeError, EncodeError};

#[derive(Error, Debug)]
pub enum ChatError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encoding error: {0}")]
    Encode(#[from] EncodeError),

    #[error("Decoding error: {0}")]
    Decode(#[from] DecodeError),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Recipient not found: {0}")]
    RecipientNotFound(u64),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type ChatResult<T> = Result<T, ChatError>;
