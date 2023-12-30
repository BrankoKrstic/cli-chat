use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMessage {
    pub sender: SocketAddr,
    pub time: u64,
    pub payload: MessagePayload,
}

impl ServerMessage {
    pub fn new(sender: SocketAddr, payload: MessagePayload) -> Self {
        Self {
            sender,
            time: 0,
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Signup { email: String, password: String },
    Login { email: String, password: String },
    SignupAccepted { nick: String },
    LoginAccepted { nick: String },
    LoginFail,
    SignupFail,
    Nickname { new_nick: String, nick: String },
    Message { nick: String, message: String },
    Connect { nick: String },
    Disconnect { nick: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Signup { email: String, password: String },
    Login { email: String, password: String },
    Nickname(String),
    Message(String),
}
