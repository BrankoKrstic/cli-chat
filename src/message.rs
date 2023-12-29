use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: SocketAddr,
    pub payload: MessagePayload,
}

impl Message {
    pub fn new(sender: SocketAddr, payload: MessagePayload) -> Self {
        Self { sender, payload }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Connected,
    Disconnect,
    Signup { email: String, password: String },
    Login { email: String, password: String },
    Message(String),
}
