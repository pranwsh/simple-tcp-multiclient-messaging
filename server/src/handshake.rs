use std::sync::Arc;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use common::{AuthRequest, AuthResponse, error::ChatResult};
use crate::user_store::UserStore;

pub struct HandshakeHandler {
    user_store: Arc<UserStore>,
}

impl HandshakeHandler {
    pub fn new(user_store: Arc<UserStore>) -> Self {
        Self { user_store }
    }

    pub async fn handle(
        &self,
        stream: &mut FramedRead<tokio::net::tcp::OwnedReadHalf, LengthDelimitedCodec>,
        sink: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, LengthDelimitedCodec>,
    ) -> ChatResult<(u64, String)> {
        let mut user_info = None;

        loop {
            let raw = match stream.next().await {
                Some(Ok(raw)) => raw,
                Some(Err(e)) => return Err(e.into()),
                None => return Err(common::error::ChatError::Protocol("Connection closed during handshake".into())),
            };

            let req = AuthRequest::decode(&raw)?;
            
            let response = match req {
                AuthRequest::Register { username, password } => {
                    match self.user_store.register(username.clone(), password.clone()) {
                        Ok(id) => {
                            user_info = Some((id, username.clone()));
                            AuthResponse::Success { user_id: id }
                        }
                        Err(reason) => AuthResponse::Failure { reason },
                    }
                }
                AuthRequest::Login { username, password } => {
                    match self.user_store.login(&username, &password) {
                        Ok(id) => {
                            user_info = Some((id, username.clone()));
                            AuthResponse::Success { user_id: id }
                        }
                        Err(reason) => AuthResponse::Failure { reason },
                    }
                }
                AuthRequest::Lookup { username } => {
                    match self.user_store.lookup(&username) {
                        Ok(id) => AuthResponse::LookupResult { user_id: id },
                        Err(reason) => AuthResponse::Failure { reason },
                    }
                }
                AuthRequest::StartChat => {
                    if let Some(info) = user_info.take() {
                        let response = AuthResponse::Success { user_id: info.0 };
                        let response_bytes = response.encode()?;
                        sink.send(Bytes::from(response_bytes)).await?;
                        return Ok(info);
                    } else {
                        AuthResponse::Failure { reason: "Not authenticated".to_string() }
                    }
                }
            };

            let response_bytes = response.encode()?;
            sink.send(Bytes::from(response_bytes)).await?;
        }
    }
}
