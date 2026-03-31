use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use common::ClientMessage;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use tokio::{
    net::TcpListener,
    sync::{RwLock, mpsc},
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tcp listener init
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("listening on port 8000");

    // encoded messages buffer mpsc
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);

    // hashmap to store client_id and respective datasink
    let clients: Arc<
        RwLock<
            HashMap<Vec<u8>, SplitSink<Framed<tokio::net::TcpStream, LengthDelimitedCodec>, Bytes>>,
        >,
    > = Arc::new(RwLock::new(HashMap::new()));

    // send received encoded message to channel
    let clients_for_receive = Arc::clone(&clients);
    let receive_to_channel = tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let tx = tx.clone();
            let clients_for_receive = Arc::clone(&clients_for_receive);
            tokio::spawn(async move {
                // frame tcp connection to client
                let framed = Framed::new(socket, LengthDelimitedCodec::new());
                let (sink, mut stream) = framed.split();

                // put client_id, and client address into db
                match stream.next().await {
                    Some(Ok(raw_id)) => {
                        let mut clients_map = clients_for_receive.write().await;
                        clients_map.insert(raw_id.to_vec(), sink);
                    }
                    Some(Err(e)) => eprintln!("stream error: {e}"),
                    None => eprintln!("stream ended unexpectedly"),
                }

                while let Some(Ok(raw_received)) = stream.next().await {
                    let msg = &raw_received;
                    if tx.send(msg.to_vec()).await.is_err() {
                        eprint!("channel closed");
                    }
                }
            });
        }
    });

    // route messages from channel to target client
    let clients_for_route = Arc::clone(&clients);
    let route_message = tokio::spawn(async move {
        while let Some(bytes) = rx.recv().await {
            match ClientMessage::decode(&bytes) {
                Ok(message) => {
                    let to_id = message.get_recipient_id();
                    let key = to_id.to_be_bytes();
                    let mut map = clients_for_route.write().await;
                    match map.get_mut(key.as_ref()) {
                        Some(sink) => {
                            if let Err(e) = sink.send(bytes.into()).await {
                                eprintln!("send error: {e}");
                            }
                        }
                        None => eprintln!("recipient not found"),
                    }
                }
                Err(e) => eprint!("{e}"),
            }
        }
    });

    let _ = tokio::join!(receive_to_channel, route_message);

    Ok(())
}
