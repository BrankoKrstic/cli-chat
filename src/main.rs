use tokio::{io::AsyncWriteExt, net::TcpListener};

type ChatResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> ChatResult<()> {
    let addr = "127.0.0.1";
    let port = 8080;
    let listener = TcpListener::bind((addr, port))
        .await
        .map_err(|e| format!("Failed to bind to socket {addr} {}", e))?;

    loop {
        match listener.accept().await {
            Ok((mut socket, _)) => {
                socket.write(b"Hello, brother").await;
            }
            Err(e) => eprintln!("failed connecting to client: {}", e),
        }
    }
}
