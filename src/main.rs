use futures_util::StreamExt;
use rust_chat::codec::ChatFrameCodec;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio_util::codec::Framed;

pub type ChatError = Box<dyn Error + Send + Sync + 'static>;
type ChatResult<T> = Result<T, ChatError>;

#[derive(Debug, Clone)]
enum Message {
    Connected,
    Disconnected,
    Message,
}

async fn handle_connection(
    socket: TcpStream,
    addr: SocketAddr,
    message_sender: mpsc::Sender<Message>,
    broadcast_receiver: broadcast::Receiver<Message>,
) -> ChatResult<()> {
    message_sender.send(Message::Connected).await?;
    let mut stream = Framed::new(socket, ChatFrameCodec);

    while let Some(msg) = stream.next().await {}
    Ok(())
}
async fn handle_broadcast(
    message_receiver: mpsc::Receiver<Message>,
    broadcast: broadcast::Sender<Message>,
) {
}

#[tokio::main]
async fn main() -> ChatResult<()> {
    let addr = "127.0.0.1";
    let port = 8080;
    let listener = TcpListener::bind((addr, port))
        .await
        .map_err(|e| format!("Failed to bind to socket {addr} {}", e))?;
    let (tx, rx) = mpsc::channel(100);
    let (broadcast_sender, broadcast_receiver) = broadcast::channel(100);
    tokio::spawn(handle_broadcast(rx, broadcast_sender));

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                let tx = tx.clone();
                tokio::spawn(handle_connection(
                    socket,
                    addr,
                    tx.clone(),
                    broadcast_receiver.resubscribe(),
                ));
            }
            Err(e) => eprintln!("failed connecting to client: {}", e),
        }
    }
}
