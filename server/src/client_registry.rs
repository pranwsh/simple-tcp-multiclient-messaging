use bytes::Bytes;
use dashmap::DashMap;
use tokio::sync::mpsc;

pub struct ClientRegistry {
    clients: DashMap<u64, mpsc::Sender<Bytes>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
        }
    }

    pub fn register(&self, id: u64, sender: mpsc::Sender<Bytes>) {
        self.clients.insert(id, sender);
    }

    pub fn deregister(&self, id: u64) {
        self.clients.remove(&id);
    }

    pub fn get_sender(&self, id: u64) -> Option<mpsc::Sender<Bytes>> {
        self.clients.get(&id).map(|entry| entry.clone())
    }

    pub fn is_registered(&self, id: u64) -> bool {
        self.clients.contains_key(&id)
    }
}
