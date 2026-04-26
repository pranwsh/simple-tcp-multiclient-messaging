use std::sync::Arc;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::client_registry::ClientRegistry;
use crate::user_store::UserStore;
use crate::handshake::HandshakeHandler;
use common::ClientMessage;

pub struct ClientConnection {
    socket: TcpStream,
    clients: Arc<ClientRegistry>,
    user_store: Arc<UserStore>,
}

impl ClientConnection {
    pub fn new(socket: TcpStream, clients: Arc<ClientRegistry>, user_store: Arc<UserStore>) -> Self {
        Self {
            socket,
            clients,
            user_store,
        }
    }

    pub async fn run(self) {
        let (read_half, write_half) = self.socket.into_split();
        let mut stream = FramedRead::new(read_half, Self::make_codec());
        let mut sink = FramedWrite::new(write_half, Self::make_codec());

        let handshake = HandshakeHandler::new(self.user_store.clone());
        let (user_id, _username) = match handshake.handle(&mut stream, &mut sink).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("handshake failed: {}", e);
                return;
            }
        };

        let (client_tx, client_rx) = mpsc::channel::<Bytes>(256);
        self.clients.register(user_id, client_tx);

        let send_task = Self::spawn_send_task(sink, client_rx);
        Self::recv_loop(&mut stream, &self.clients).await;

        self.clients.deregister(user_id);
        send_task.abort();
    }

    fn make_codec() -> LengthDelimitedCodec {
        LengthDelimitedCodec::builder()
            .max_frame_length(64 * 1024)
            .new_codec()
    }

    fn spawn_send_task(
        mut sink: FramedWrite<tokio::net::tcp::OwnedWriteHalf, LengthDelimitedCodec>,
        mut rx: mpsc::Receiver<Bytes>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(bytes) = rx.recv().await {
                if let Err(e) = sink.send(bytes).await {
                    eprintln!("send error: {e}");
                    break;
                }
            }
        })
    }

    async fn recv_loop(
        stream: &mut FramedRead<tokio::net::tcp::OwnedReadHalf, LengthDelimitedCodec>,
        clients: &Arc<ClientRegistry>,
    ) {
        while let Some(result) = stream.next().await {
            let raw = match result {
                Ok(bytes) => bytes.freeze(),
                Err(e) => {
                    eprintln!("stream error: {e}");
                    break;
                }
            };

            // Header-only decoding for routing
            match ClientMessage::decode_recipient_id(&raw) {
                Ok(recipient_id) => {
                    let sender = match clients.get_sender(recipient_id) {
                        Some(s) => s,
                        None => {
                            eprintln!("recipient not found: {}", recipient_id);
                            continue;
                        }
                    };
                    if sender.send(raw).await.is_err() {
                        eprintln!("recipient channel closed");
                    }
                }
                Err(e) => eprintln!("decode error (recipient_id): {e}"),
            }
        }
    }
}
