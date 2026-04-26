# Project Overview: Rust Chat Client

This is a terminal-based chat client implemented in Rust. It establishes an asynchronous, interactive connection to a server using the Tokio runtime. The project utilizes a length-delimited codec for message framing and handles concurrent I/O operations (reading, writing, and stdin processing) using separate tasks coordinated via channels and cancellation tokens.

## Architecture & Technologies
- **Language:** Rust (2024 Edition)
- **Runtime:** [Tokio](https://tokio.rs/) (Asynchronous I/O)
- **Networking:** TCP with `tokio::net::TcpStream`
- **Framing:** `tokio-util`'s `LengthDelimitedCodec`
- **Concurrency:** `tokio::spawn`, `tokio::select!`, and `mpsc` channels for task coordination.
- **Dependencies:** 
  - `common`: A workspace dependency for shared message structures (e.g., `ClientMessage`).
  - `serde`: For serialization/deserialization.
  - `bytes`: Efficient byte buffer management.

## Project Structure
- `src/main.rs`: Entry point that initializes the `ClientConnection`.
- `src/connection.rs`: Manages the TCP connection lifecycle.
- `src/client.rs`: The core logic for running the chat loop and spawning I/O tasks.
- `src/handshake.rs`: Implements the initial identification protocol with the server.
- `src/auth.rs`: Handles user identification prompts (sender and recipient IDs).
- `src/io/`: Module containing specialized tasks for:
  - `stdin.rs`: Processing user input and encoding it into `ClientMessage`.
  - `reader.rs`: Handling incoming messages from the server.
  - `writer.rs`: Sending outbound messages to the server.
- `src/error.rs`: Centralized error handling for the client.

## Building and Running

### Prerequisites
- Rust and Cargo (latest stable version).
- A running server compatible with the expected protocol (default address: `127.0.0.1:8000`).

### Build
```bash
cargo build
```

### Run
```bash
cargo run
```

### Test
```bash
cargo test
```

## Development Conventions
- **Asynchronous Code:** Prefer `tokio` primitives for concurrency and I/O.
- **Error Handling:** Use the custom `ClientError` type for project-specific errors; leverage `Result` for flow control.
- **Modules:** Follow the file-per-module structure. New I/O tasks should be placed in `src/io/`.
- **Formatting:** Adhere to standard Rust formatting (`cargo fmt`).
