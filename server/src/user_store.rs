use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
mod registry;
mod persistence;

use registry::UserRegistry;
use persistence::UserPersistence;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password: String,
}

pub struct UserStore {
    registry: UserRegistry,
    persistence_tx: mpsc::UnboundedSender<User>,
}

impl UserStore {
    pub fn new(file_path: &str) -> Self {
        let registry = UserRegistry::new(file_path);
        let (persistence, tx) = UserPersistence::new(file_path);
        
        // Spawn the persistence worker
        tokio::spawn(async move {
            persistence.run().await;
        });

        Self {
            registry,
            persistence_tx: tx,
        }
    }

    pub fn register(&self, username: String, password: String) -> Result<u64, String> {
        if self.registry.exists(&username) {
            return Err("User already exists".to_string());
        }

        let id = (self.registry.count() as u64) + 1;
        let user = User {
            id,
            username,
            password,
        };

        // Update in-memory registry (Active/Registered distinction)
        self.registry.insert(user.clone());
        
        // Asynchronously persist to CSV (Non-blocking)
        let _ = self.persistence_tx.send(user);

        Ok(id)
    }

    pub fn login(&self, username: &str, password: &str) -> Result<u64, String> {
        if let Some(user) = self.registry.get_by_username(username) {
            if user.password == password {
                Ok(user.id)
            } else {
                Err("Invalid password".to_string())
            }
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn lookup(&self, username: &str) -> Result<u64, String> {
        if let Some(user) = self.registry.get_by_username(username) {
            Ok(user.id)
        } else {
            Err("User not found".to_string())
        }
    }
}
