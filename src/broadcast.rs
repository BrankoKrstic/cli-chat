use crate::message::ServerMessage;
use crate::ChatResult;
use tokio::sync::{broadcast, mpsc};

pub async fn handle_broadcast(
    mut message_receiver: mpsc::Receiver<ServerMessage>,
    broadcast: broadcast::Sender<ServerMessage>,
) -> ChatResult<()> {
    while let Some(msg) = message_receiver.recv().await {
        let _ = broadcast.send(msg)?;
    }
    Ok(())
}
