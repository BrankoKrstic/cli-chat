use std::sync::{Arc, Mutex};

use rust_chat::auth::Auth;
use rust_chat::broadcast::handle_broadcast;
use rust_chat::connection::handle_login;
use rust_chat::ChatResult;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

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
    let auth = Arc::new(Mutex::new(Auth::new()));
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                let tx = tx.clone();
                tokio::spawn(handle_login(
                    socket,
                    addr,
                    Arc::clone(&auth),
                    tx.clone(),
                    broadcast_receiver.resubscribe(),
                ));
            }
            Err(e) => eprintln!("failed connecting to client: {}", e),
        }
    }
}
