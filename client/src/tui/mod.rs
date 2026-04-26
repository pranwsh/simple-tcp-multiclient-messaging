use std::io;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, BorderType},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::mpsc;
use bytes::Bytes;
use common::ClientMessage;
use crate::error::ClientError;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::StreamExt;
use tokio_util::sync::CancellationToken;
use crate::io::{reader, writer};

mod widgets;
use widgets::MessageRenderer;

pub enum InputMode {
    Normal,
    Editing,
}

pub struct TuiApp {
    from_id: u64,
    to_id: u64,
    input: String,
    messages: Vec<ClientMessage>,
    input_mode: InputMode,
}

impl TuiApp {
    pub fn new(from_id: u64, to_id: u64) -> Self {
        Self {
            from_id,
            to_id,
            input: String::new(),
            messages: Vec::new(),
            input_mode: InputMode::Editing,
        }
    }

    pub async fn run(mut self, stream: tokio::net::TcpStream) -> Result<(), ClientError> {
        enable_raw_mode().map_err(|e| ClientError::Connection(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(|e| ClientError::Connection(e.to_string()))?;

        let (sink, stream_parts) = Framed::new(stream, LengthDelimitedCodec::new()).split();
        let (tx_outbound, rx_outbound) = mpsc::channel::<Bytes>(64);
        let (tx_inbound, mut rx_inbound) = mpsc::channel::<ClientMessage>(64);
        let token = CancellationToken::new();

        let _writer_handle = tokio::spawn(async move {
            let _ = writer::run(sink, rx_outbound).await;
        });

        let _reader_handle = tokio::spawn(async move {
            let _ = reader::run(stream_parts, tx_inbound).await;
        });

        let mut ticker = tokio::time::interval(std::time::Duration::from_millis(16)); // ~60fps

        let res = loop {
            terminal.draw(|f| self.draw(f)).map_err(|e| ClientError::Connection(e.to_string()))?;

            tokio::select! {
                _ = ticker.tick() => {
                    if event::poll(std::time::Duration::from_millis(0)).map_err(|e| ClientError::Connection(e.to_string()))? {
                        if let Event::Key(key) = event::read().map_err(|e| ClientError::Connection(e.to_string()))? {
                            match self.input_mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('e') => {
                                        self.input_mode = InputMode::Editing;
                                    }
                                    KeyCode::Char('q') | KeyCode::Esc => {
                                        break Ok(());
                                    }
                                    _ => {}
                                },
                                InputMode::Editing => match key.code {
                                    KeyCode::Enter => {
                                        if !self.input.is_empty() {
                                            let content = self.input.drain(..).collect::<String>();
                                            let msg = ClientMessage::new(self.from_id, self.to_id, content);
                                            if let Ok(bytes) = msg.encode() {
                                                tx_outbound.send(Bytes::from(bytes)).await.ok();
                                                self.messages.push(msg);
                                            }
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        self.input.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        self.input.pop();
                                    }
                                    KeyCode::Esc => {
                                        self.input_mode = InputMode::Normal;
                                    }
                                    _ => {}
                                },
                            }
                        }
                    }
                }
                Some(msg) = rx_inbound.recv() => {
                    self.messages.push(msg);
                }
            }
        };

        // Cleanup
        token.cancel();
        disable_raw_mode().ok();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).ok();
        terminal.show_cursor().ok();

        res
    }

    fn draw(&self, f: &mut ratatui::Frame) {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        // Main Box
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Chat ");
        f.render_widget(block, chunks[0]);

        // Input Box
        let mode_text = match self.input_mode {
            InputMode::Normal => " (NORMAL) - Press 'e' to edit, 'q' to quit ",
            InputMode::Editing => " (EDITING) - Press 'Esc' to stop editing, 'Enter' to send ",
        };
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!(" Message{} ", mode_text));
        let input_para = Paragraph::new(self.input.as_str()).block(input_block);
        f.render_widget(input_para, chunks[1]);

        // Message List Rendering
        let inner_area = Rect::new(
            chunks[0].x + 1,
            chunks[0].y + 1,
            chunks[0].width - 2,
            chunks[0].height - 2,
        );
        MessageRenderer::render(f, inner_area, &self.messages, self.from_id);
    }
}
