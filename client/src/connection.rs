use tokio::net::TcpStream;
use crate::error::ClientError;
use crate::auth::Auth;
use crate::handshake::AuthSession;
use crate::tui::TuiApp;

pub enum ClientState {
    Initial,
    Authenticating,
    SelectingRecipient { from_id: u64, session: AuthSession },
    Chatting { from_id: u64, to_id: u64, stream: TcpStream },
    Error(ClientError),
    Terminated,
}

pub struct ClientConnection {
    addr: String,
}

impl ClientConnection {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
        }
    }

    pub async fn connect(&self) -> Result<TcpStream, ClientError> {
        let stream = TcpStream::connect(&self.addr).await?;
        stream.set_nodelay(true)?;
        Ok(stream)
    }

    pub async fn run_interactive(self) -> Result<(), ClientError> {
        let mut state = ClientState::Initial;

        loop {
            state = match state {
                ClientState::Initial => {
                    println!("--- Starting Chat Client ---");
                    ClientState::Authenticating
                }
                
                ClientState::Authenticating => {
                    let auth_req = match Auth::prompt_auth().await {
                        Ok(req) => req,
                        Err(e) => break Err(e),
                    };

                    println!("Connecting to {}...", self.addr);
                    match self.connect().await {
                        Ok(stream) => {
                            let mut session = AuthSession::new(stream);
                            match session.authenticate(auth_req).await {
                                Ok(from_id) => ClientState::SelectingRecipient { from_id, session },
                                Err(e) => ClientState::Error(e),
                            }
                        }
                        Err(e) => ClientState::Error(e),
                    }
                }

                ClientState::SelectingRecipient { from_id, mut session } => {
                    let to_username = match Auth::prompt_recipient().await {
                        Ok(u) => u,
                        Err(e) => break Err(e),
                    };

                    match session.lookup_username(to_username).await {
                        Ok(to_id) => {
                            println!("Resolved recipient to ID: {}", to_id);
                            match session.start_chat().await {
                                Ok(_) => ClientState::Chatting { 
                                    from_id, 
                                    to_id, 
                                    stream: session.into_inner() 
                                },
                                Err(e) => ClientState::Error(e),
                            }
                        }
                        Err(e) => {
                            println!("Recipient lookup failed: {}", e);
                            // Fallback: stay in the same state to try another recipient
                            ClientState::SelectingRecipient { from_id, session }
                        }
                    }
                }

                ClientState::Chatting { from_id, to_id, stream } => {
                    let app = TuiApp::new(from_id, to_id);
                    match app.run(stream).await {
                        Ok(_) => ClientState::Terminated,
                        Err(e) => ClientState::Error(e),
                    }
                }

                ClientState::Error(e) => {
                    eprintln!("\n[!] Application Error: {}", e);
                    println!("Would you like to try again? (y/n)");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).ok();
                    if input.trim().to_lowercase() == "y" {
                        ClientState::Initial
                    } else {
                        break Err(e);
                    }
                }

                ClientState::Terminated => {
                    println!("\nSession terminated gracefully. Goodbye!");
                    break Ok(());
                }
            };
        }
    }
}
