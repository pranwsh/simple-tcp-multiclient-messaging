use std::fs::{File, OpenOptions};
use std::path::Path;
use tokio::sync::mpsc;
use crate::user_store::User;

pub struct UserPersistence {
    rx: mpsc::UnboundedReceiver<User>,
    file_path: String,
}

impl UserPersistence {
    pub fn new(file_path: &str) -> (Self, mpsc::UnboundedSender<User>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                rx,
                file_path: file_path.to_string(),
            },
            tx,
        )
    }

    pub async fn run(mut self) {
        let path = Path::new(&self.file_path);
        let file_exists = path.exists();
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .expect("Failed to open CSV for writing");

        let mut wtr = csv::WriterBuilder::new()
            .has_headers(!file_exists)
            .from_writer(file);

        if !file_exists {
            wtr.write_record(&["id", "username", "password"]).expect("Failed to write headers");
        }

        while let Some(user) = self.rx.recv().await {
            if let Err(e) = wtr.serialize(&user) {
                eprintln!("Failed to serialize user to CSV: {}", e);
                continue;
            }
            if let Err(e) = wtr.flush() {
                eprintln!("Failed to flush CSV: {}", e);
            }
        }
    }
}
