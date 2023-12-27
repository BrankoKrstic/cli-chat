use futures_util::{SinkExt, StreamExt};
use rust_chat::{
    codec::{ChatFrame, ChatFrameCodec, ChatFrameTag},
    ChatResult,
};
use tokio::{io::AsyncBufReadExt, net::TcpStream};
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> ChatResult<()> {
    let socket = TcpStream::connect(("127.0.0.1", 8080)).await?;
    let mut stream = Framed::new(socket, ChatFrameCodec);
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    loop {
        let mut buf = vec![];
        tokio::select! {
          msg = stream.next() => {
            if let Some(msg) = msg {
              println!("{}", msg?.payload);
            }
          },
          _ = reader.read_until(b'\n', &mut buf) => {
            buf.pop();
            let payload = String::from_utf8(buf)?;
            stream.send(ChatFrame {
              tag: ChatFrameTag::Message,
              payload
            }).await?;
          }
        }
    }
}
