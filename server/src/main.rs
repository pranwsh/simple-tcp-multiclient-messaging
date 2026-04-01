use bytes::Bytes;
use common::ClientMessage;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

type ClientMap = Arc<DashMap<Vec<u8>, mpsc::Sender<Bytes>>>;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("listening on port 8000");

    let clients: ClientMap = Arc::new(DashMap::new());

    loop {
        let (socket, _) = listener.accept().await?;
        socket.set_nodelay(true)?;

        let clients = Arc::clone(&clients);

        tokio::spawn(async move {
            let framed = Framed::new(
                socket,
                LengthDelimitedCodec::builder()
                    .max_frame_length(64 * 1024)
                    .new_codec(),
            );

            let (mut sink, mut stream) = framed.split();

            let client_id = match stream.next().await {
                Some(Ok(raw)) => raw.freeze(),
                Some(Err(e)) => {
                    eprintln!("handshake err {}", e);
                    return;
                }
                None => {
                    eprintln!("connection closed during handshake");
                    return;
                }
            };

            let (client_tx, mut client_rx) = mpsc::channel::<Bytes>(256);
            clients.insert(client_id.to_vec(), client_tx);

            let send_task = tokio::spawn(async move {
                while let Some(bytes) = client_rx.recv().await {
                    if let Err(e) = sink.send(bytes).await {
                        eprintln!("send error {e}");
                        break;
                    }
                }
            });

            while let Some(result) = stream.next().await {
                let raw_message = match result {
                    Ok(bytes) => bytes.freeze(),
                    Err(e) => {
                        eprintln!("recipient not found {}", e);
                        break;
                    }
                };

                match ClientMessage::decode(&raw_message) {
                    Ok(message) => {
                        let key = message.get_recipient_id().to_be_bytes();

                        let sender = match clients.get(key.as_ref()) {
                            Some(entry) => entry.clone(),
                            None => {
                                eprintln!("recipient not found");
                                continue;
                            }
                        };

                        if sender.send(raw_message).await.is_err() {
                            eprintln!("recipient channel closed");
                        }
                    }

                    Err(e) => eprintln!("decode error {e}"),
                }
            }

            clients.remove(&client_id.to_vec());
            send_task.abort();
        });
    }
}
