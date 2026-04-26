use std::sync::Arc;
mod client_registry;
mod connection_acceptor;
mod user_store;
mod handshake;
mod connection;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let config = common::config::ChatConfig::default();
    let clients = Arc::new(client_registry::ClientRegistry::new());
    let user_store = Arc::new(user_store::UserStore::new("users.csv"));
    let acceptor = connection_acceptor::ConnectionAcceptor::bind(&config.server_addr, clients, user_store).await?;
    println!("listening on {}", config.server_addr);
    acceptor.run().await;
    Ok(())
}
