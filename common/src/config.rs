pub struct ChatConfig {
    pub server_addr: String,
    pub max_frame_size: usize,
}

impl ChatConfig {
    pub fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:8000".to_string(),
            max_frame_size: 64 * 1024,
        }
    }
}
