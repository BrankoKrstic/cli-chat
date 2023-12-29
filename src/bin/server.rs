use futures_util::{SinkExt, StreamExt};
use rust_chat::codec::MessagePayloadCodec;
use rust_chat::message::{Message, MessagePayload};
use rust_chat::ChatResult;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio_util::codec::Framed;

async fn handle_connection(
    socket: TcpStream,
    addr: SocketAddr,
    message_sender: mpsc::Sender<Message>,
    mut broadcast_receiver: broadcast::Receiver<Message>,
) -> ChatResult<()> {
    message_sender
        .send(Message::new(addr, MessagePayload::Connected))
        .await?;
    let mut stream = Framed::new(socket, MessagePayloadCodec);
    loop {
        tokio::select! {
            msg = stream.next() => {
                if let Some(Ok(msg)) = msg {
                    message_sender.send(Message::new(addr, msg)).await?;
                }

            }
            msg = broadcast_receiver.recv() => {
                let msg = msg?;
                // println!("Message received: {:?}", msg);
                if msg.sender == addr { continue; }
                stream.send(msg.payload).await?;
            }

        }
    }
}
async fn handle_broadcast(
    mut message_receiver: mpsc::Receiver<Message>,
    broadcast: broadcast::Sender<Message>,
) -> ChatResult<()> {
    while let Some(msg) = message_receiver.recv().await {
        let _ = broadcast.send(msg)?;
    }
    Ok(())
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
