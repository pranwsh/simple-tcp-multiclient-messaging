use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use crate::error::ClientError;
use common::AuthRequest;

pub struct Auth;

impl Auth {
    pub async fn prompt_auth() -> Result<AuthRequest, ClientError> {
        let mut stdin = BufReader::new(tokio::io::stdin());
        let mut buf = String::new();

        println!("1. Register");
        println!("2. Login");
        print!("Choose an option: ");
        let _ = tokio::io::stdout().flush(); // stdout is not flushed automatically
        stdin.read_line(&mut buf).await?;
        let choice = buf.trim();

        let is_register = match choice {
            "1" => true,
            "2" => false,
            _ => return Err(ClientError::Handshake("Invalid choice".to_string())),
        };

        buf.clear();
        println!("Enter username:");
        stdin.read_line(&mut buf).await?;
        let username = buf.trim().to_string();

        buf.clear();
        println!("Enter password:");
        stdin.read_line(&mut buf).await?;
        let password = buf.trim().to_string();

        if is_register {
            Ok(AuthRequest::Register { username, password })
        } else {
            Ok(AuthRequest::Login { username, password })
        }
    }

    pub async fn prompt_recipient() -> Result<String, ClientError> {
        let mut stdin = BufReader::new(tokio::io::stdin());
        let mut buf = String::new();

        println!("enter recipient username");
        stdin.read_line(&mut buf).await?;
        let to_username = buf.trim().to_string();

        Ok(to_username)
    }
}
