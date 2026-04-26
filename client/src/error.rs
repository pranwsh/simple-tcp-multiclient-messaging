use std::fmt;

#[derive(Debug)]
pub enum ClientError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Connection(String),
    Handshake(String),
    Send(String),
    Receive(String),
    Channel(String),
    Codec(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Io(e) => write!(f, "IO error: {}", e),
            ClientError::Parse(e) => write!(f, "Parse error: {}", e),
            ClientError::Connection(s) => write!(f, "Connection error: {}", s),
            ClientError::Handshake(s) => write!(f, "Handshake error: {}", s),
            ClientError::Send(s) => write!(f, "Send error: {}", s),
            ClientError::Receive(s) => write!(f, "Receive error: {}", s),
            ClientError::Channel(s) => write!(f, "Channel error: {}", s),
            ClientError::Codec(s) => write!(f, "Codec error: {}", s),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClientError::Io(e) => Some(e),
            ClientError::Parse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::Io(e)
    }
}

impl From<std::num::ParseIntError> for ClientError {
    fn from(e: std::num::ParseIntError) -> Self {
        ClientError::Parse(e)
    }
}
