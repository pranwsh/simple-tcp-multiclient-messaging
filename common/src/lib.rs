use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub struct ClientMessage {
    sender_id: u64,
    recipient_id: u64,
    content: String,
}

impl ClientMessage {
    pub fn new(sender_id: u64, recipient_id: u64, content: String) -> Self {
        Self {
            sender_id,
            recipient_id,
            content,
        }
    }

    pub fn get_sender_id(&self) -> u64 {
        self.sender_id
    }

    pub fn get_recipient_id(&self) -> u64 {
        self.recipient_id
    }

    pub fn encode(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = bincode::encode_to_vec(self, bincode::config::standard())?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (message, _) = bincode::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(message)
    }
}
