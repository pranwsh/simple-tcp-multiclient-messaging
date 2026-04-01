use common::ClientMessage;
use futures::{SinkExt, StreamExt};
use tokio::{
  io::{AsyncBufReadExt, BufReader},
  net::TcpStream,
  sync::mpsc,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // connect to server
  let stream = TcpStream::connect("127.0.0.1:8000").await?;
  println!("client connected to port 8000");

  // frame TCP stream for byte-level communication
  let framed = Framed::new(stream, LengthDelimitedCodec::new());
  let (mut sink, mut stream) = framed.split();

  // get user ids no blockingly
  let mut to_id_str = String::new();
  println!("enter recipient id");
  BufReader::new(tokio::io::stdin())
    .read_line(&mut to_id_str)
    .await?;
  let to_id: u64 = to_id_str.trim().parse().expect("invalid recipient id");

  let mut from_id_str = String::new();
  println!("enter self id");
  BufReader::new(tokio::io::stdin())
    .read_line(&mut from_id_str)
    .await?;
  let from_id: u64 = from_id_str.trim().parse().expect("invalid self id");

  // create channel for sender task
  let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);

  // task to read from stream and print to console
  let reader_task = tokio::spawn(async move {
    while let Some(Ok(received_raw)) = stream.next().await {
      let received_bytes: &[u8] = &received_raw;

      match ClientMessage::decode(received_bytes) {
        Ok(msg) => println!("{:?}", msg),
        Err(e) => eprintln!("failed to decode message: {}", e),
      };

      // TODO: Decode received_bytes into ClientMessage
      // Example:
      // if let Ok(msg) = ClientMessage::decode(received_bytes) {
      //     println!("from {}: {}", msg.from_id, msg.content);
      // }

      // TODO: Handle different message types (handshake, chat, disconnect, etc.)

      // TODO: Print or process the message
    }
    println!("disconnected from server");
  });

  // writer task to send from channel to server
  let writer_task = tokio::spawn(async move {
    while let Some(bytes) = rx.recv().await {
      if sink
        .send(tokio_util::bytes::Bytes::from(bytes))
        .await
        .is_err()
      {
        eprintln!("failed to send message");
        break;
      }
    }
  });

  // 6. stdin task to read user input and send to channel
  let stdin_task = tokio::spawn(async move {
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    if tx.send(from_id.to_be_bytes().to_vec()).await.is_err() {
      eprintln!("failed to send id");
    }

    loop {
      line.clear();
      match stdin.read_line(&mut line).await {
        Ok(0) => {
          println!("stdin closed");
          break;
        }
        Ok(_) => {
          let trimmed = line.trim();
          if trimmed.is_empty() {
            continue;
          }

          if trimmed == "/quit" {
            println!("quitting...");
            break;
          }

          let message = ClientMessage::new(from_id, to_id, trimmed.to_string());

          match message.encode() {
            Ok(bytes) => {
              if tx.send(bytes).await.is_err() {
                eprintln!("failed to queue message");
                break;
              }
            }
            Err(e) => {
              eprintln!("failed to encode message: {}", e);
            }
          }
        }
        Err(e) => {
          eprintln!("error reading stdin: {}", e);
          break;
        }
      }
    }
    // tx dropped automatically here
  });

  // wait for all tasks
  let _ = tokio::join!(reader_task, writer_task, stdin_task);

  Ok(())
}
