use std::net::SocketAddr;

use crate::codec::{ChatFrame, ChatFrameTag};

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

#[derive(Debug, Clone)]
pub enum MessagePayload {
    Connected,
    Disconnect,
    Message(String),
}

impl From<ChatFrame> for MessagePayload {
    fn from(frame: ChatFrame) -> Self {
        match frame.tag {
            ChatFrameTag::Message => MessagePayload::Message(frame.payload),
            ChatFrameTag::NameChange => todo!(),
            ChatFrameTag::Connect => MessagePayload::Connected,
            ChatFrameTag::Disconnect => MessagePayload::Disconnect,
        }
    }
}

impl From<MessagePayload> for ChatFrame {
    fn from(payload: MessagePayload) -> ChatFrame {
        match payload {
            MessagePayload::Connected => ChatFrame {
                tag: ChatFrameTag::Connect,
                payload: String::new(),
            },
            MessagePayload::Disconnect => ChatFrame {
                tag: ChatFrameTag::Disconnect,
                payload: String::new(),
            },
            MessagePayload::Message(payload) => ChatFrame {
                tag: ChatFrameTag::Message,
                payload,
            },
        }
    }
}
