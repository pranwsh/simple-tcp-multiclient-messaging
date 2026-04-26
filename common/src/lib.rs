pub mod error;
pub mod config;

use bincode::{Decode, Encode};

// Define a consistent bincode configuration for the whole project.
// We use BigEndian and Fixint to ensure u64 is always exactly 8 bytes and easy to read from headers.
fn bincode_config() -> impl bincode::config::Config {
    bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding()
}

#[derive(Encode, Decode, Debug)]
pub enum AuthRequest {
    Register { username: String, password: String },
    Login { username: String, password: String },
    Lookup { username: String },
    StartChat,
}

impl AuthRequest {
    pub fn encode(&self) -> error::ChatResult<Vec<u8>> {
        let bytes = bincode::encode_to_vec(self, bincode_config())?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> error::ChatResult<Self> {
        let (request, _) = bincode::decode_from_slice(bytes, bincode_config())?;
        Ok(request)
    }
}

#[derive(Encode, Decode, Debug)]
pub enum AuthResponse {
    Success { user_id: u64 },
    Failure { reason: String },
    LookupResult { user_id: u64 },
}

impl AuthResponse {
    pub fn encode(&self) -> error::ChatResult<Vec<u8>> {
        let bytes = bincode::encode_to_vec(self, bincode_config())?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> error::ChatResult<Self> {
        let (response, _) = bincode::decode_from_slice(bytes, bincode_config())?;
        Ok(response)
    }
}

#[derive(Encode, Decode, Debug)]
pub struct ClientMessage {
  recipient_id: u64,
  sender_id: u64,
  content: String,
}

impl ClientMessage {
  pub fn new(sender_id: u64, recipient_id: u64, content: String) -> Self {
    Self {
      recipient_id,
      sender_id,
      content,
    }
  }

  pub fn get_sender_id(&self) -> u64 {
    self.sender_id
  }

  pub fn get_recipient_id(&self) -> u64 {
    self.recipient_id
  }

  pub fn get_content(&self) -> &str {
    &self.content
  }

  /// Encodes the message with a fixed 16-byte header:
  /// [0..8]   recipient_id (BigEndian)
  /// [8..16]  sender_id (BigEndian)
  /// [16..]   content (bincode)
  pub fn encode(&self) -> error::ChatResult<Vec<u8>> {
    let mut bytes = Vec::with_capacity(16 + self.content.len());
    bytes.extend_from_slice(&self.recipient_id.to_be_bytes());
    bytes.extend_from_slice(&self.sender_id.to_be_bytes());
    
    let content_bytes = bincode::encode_to_vec(&self.content, bincode_config())?;
    bytes.extend_from_slice(&content_bytes);
    
    Ok(bytes)
  }

  pub fn decode(bytes: &[u8]) -> error::ChatResult<Self> {
    if bytes.len() < 16 {
        return Err(error::ChatError::Protocol("Message too short for header".into()));
    }
    
    let recipient_id = u64::from_be_bytes(bytes[0..8].try_into().map_err(|_| error::ChatError::Protocol("Invalid recipient_id header".into()))?);
    let sender_id = u64::from_be_bytes(bytes[8..16].try_into().map_err(|_| error::ChatError::Protocol("Invalid sender_id header".into()))?);
    
    let (content, _) = bincode::decode_from_slice::<String, _>(&bytes[16..], bincode_config())?;
    
    Ok(Self {
      recipient_id,
      sender_id,
      content,
    })
  }

  /// Efficiently decodes only the recipient_id from the fixed 16-byte header.
  pub fn decode_recipient_id(bytes: &[u8]) -> error::ChatResult<u64> {
    if bytes.len() < 8 {
        return Err(error::ChatError::Protocol("Buffer too short for recipient_id".into()));
    }
    let recipient_id = u64::from_be_bytes(bytes[0..8].try_into().map_err(|_| error::ChatError::Protocol("Invalid recipient_id header".into()))?);
    Ok(recipient_id)
  }
}
