mod connection;
mod io;
mod auth;
mod handshake;
mod error;
mod tui;

use crate::connection::ClientConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = common::config::ChatConfig::default();
    let connection = ClientConnection::new(&config.server_addr);
    
    if let Err(e) = connection.run_interactive().await {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
