use std::error::Error;

pub mod auth;
pub mod broadcast;
pub mod codec;
pub mod connection;
pub mod message;

pub type ChatError = Box<dyn Error + Send + Sync + 'static>;
pub type ChatResult<T> = Result<T, ChatError>;
