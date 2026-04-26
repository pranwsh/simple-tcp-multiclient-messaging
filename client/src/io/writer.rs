use bytes::Bytes;
use futures::SinkExt;
use tokio::sync::mpsc;
use crate::error::ClientError;

pub async fn run(
    mut sink: impl SinkExt<Bytes, Error = std::io::Error> + Unpin,
    mut rx: mpsc::Receiver<Bytes>,
) -> Result<(), ClientError> {
    while let Some(bytes) = rx.recv().await {
        sink.send(bytes).await
            .map_err(|e| ClientError::Send(format!("Failed to send message: {}", e)))?;
    }
    Ok(())
}
