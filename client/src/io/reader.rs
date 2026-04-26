use common::ClientMessage;
use futures::StreamExt;
use crate::error::ClientError;
use tokio::sync::mpsc;

pub async fn run(
    mut stream: impl StreamExt<Item = Result<bytes::BytesMut, std::io::Error>> + Unpin,
    tx: mpsc::Sender<ClientMessage>,
) -> Result<(), ClientError> {
    while let Some(result) = stream.next().await {
        let raw = result.map_err(|e| ClientError::Receive(format!("Stream error: {}", e)))?;
        
        match ClientMessage::decode(&raw) {
            Ok(msg) => {
                tx.send(msg).await
                    .map_err(|e| ClientError::Channel(format!("Failed to forward message: {}", e)))?;
            }
            Err(e) => {
                eprintln!("decode error: {e}");
                // We might want to continue or stop here depending on policy
            }
        }
    }
    println!("disconnected from server");
    Ok(())
}
