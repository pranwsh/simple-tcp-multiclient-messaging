use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::{SinkExt, StreamExt};
use bytes::Bytes;
use crate::error::ClientError;
use common::{AuthRequest, AuthResponse};

pub struct AuthSession {
    framed: Framed<TcpStream, LengthDelimitedCodec>,
}

impl AuthSession {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            framed: Framed::new(stream, LengthDelimitedCodec::new()),
        }
    }

    pub async fn authenticate(&mut self, auth_req: AuthRequest) -> Result<u64, ClientError> {
        let req_bytes = auth_req.encode()
            .map_err(|e| ClientError::Handshake(format!("Failed to encode auth request: {}", e)))?;
        
        self.framed.send(Bytes::from(req_bytes)).await
            .map_err(|e| ClientError::Handshake(format!("Failed to send auth request: {}", e)))?;
        
        match self.framed.next().await {
            Some(Ok(raw)) => {
                match AuthResponse::decode(&raw) {
                    Ok(AuthResponse::Success { user_id }) => {
                        println!("Authentication successful! Your ID is: {}", user_id);
                        Ok(user_id)
                    }
                    Ok(AuthResponse::Failure { reason }) => {
                        Err(ClientError::Handshake(format!("Authentication failed: {}", reason)))
                    }
                    _ => Err(ClientError::Handshake("Unexpected response from server".to_string())),
                }
            }
            Some(Err(e)) => Err(ClientError::Handshake(format!("Handshake error: {}", e))),
            None => Err(ClientError::Handshake("Connection closed during handshake".to_string())),
        }
    }

    pub async fn lookup_username(&mut self, username: String) -> Result<u64, ClientError> {
        let req = AuthRequest::Lookup { username };
        let req_bytes = req.encode()
            .map_err(|e| ClientError::Handshake(format!("Failed to encode lookup request: {}", e)))?;
        
        self.framed.send(Bytes::from(req_bytes)).await
            .map_err(|e| ClientError::Handshake(format!("Failed to send lookup request: {}", e)))?;
        
        match self.framed.next().await {
            Some(Ok(raw)) => {
                match AuthResponse::decode(&raw) {
                    Ok(AuthResponse::LookupResult { user_id }) => {
                        Ok(user_id)
                    }
                    Ok(AuthResponse::Failure { reason }) => {
                        Err(ClientError::Handshake(format!("Lookup failed: {}", reason)))
                    }
                    _ => Err(ClientError::Handshake("Unexpected response from server".to_string())),
                }
            }
            Some(Err(e)) => Err(ClientError::Handshake(format!("Lookup error: {}", e))),
            None => Err(ClientError::Handshake("Connection closed during lookup".to_string())),
        }
    }

    pub async fn start_chat(&mut self) -> Result<(), ClientError> {
        let req = AuthRequest::StartChat;
        let req_bytes = req.encode()
            .map_err(|e| ClientError::Handshake(format!("Failed to encode start_chat request: {}", e)))?;
        
        self.framed.send(Bytes::from(req_bytes)).await
            .map_err(|e| ClientError::Handshake(format!("Failed to send start_chat request: {}", e)))?;
        
        // Wait for server acknowledgment
        match self.framed.next().await {
            Some(Ok(raw)) => {
                match AuthResponse::decode(&raw) {
                    Ok(AuthResponse::Success { .. }) => Ok(()),
                    Ok(AuthResponse::Failure { reason }) => {
                        Err(ClientError::Handshake(format!("Start chat failed: {}", reason)))
                    }
                    _ => Err(ClientError::Handshake("Unexpected response from server during start_chat".to_string())),
                }
            }
            Some(Err(e)) => Err(ClientError::Handshake(format!("Start chat error: {}", e))),
            None => Err(ClientError::Handshake("Connection closed during start_chat".to_string())),
        }
    }

    pub fn into_inner(self) -> TcpStream {
        self.framed.into_inner()
    }
}
