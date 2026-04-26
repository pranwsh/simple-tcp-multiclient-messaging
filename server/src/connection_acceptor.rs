use std::sync::Arc;
use crate::client_registry::ClientRegistry;
use crate::user_store::UserStore;
use crate::connection::ClientConnection;

pub struct ConnectionAcceptor {
    listener: tokio::net::TcpListener,
    clients: Arc<ClientRegistry>,
    user_store: Arc<UserStore>,
}

impl ConnectionAcceptor {
    pub async fn bind(
        addr: &str,
        clients: Arc<ClientRegistry>,
        user_store: Arc<UserStore>,
    ) -> Result<Self, std::io::Error> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        Ok(Self {
            listener,
            clients,
            user_store,
        })
    }

    pub async fn run(self) {
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => {
                    if let Err(e) = socket.set_nodelay(true) {
                        eprintln!("set_nodelay error: {e}");
                        continue;
                    }
                    let clients = self.clients.clone();
                    let user_store = self.user_store.clone();
                    tokio::spawn(async move {
                        ClientConnection::new(socket, clients, user_store).run().await;
                    });
                }
                Err(e) => eprintln!("accept error: {e}"),
            }
        }
    }
}
