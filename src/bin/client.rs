use futures_util::{SinkExt, StreamExt};
use rust_chat::{
    codec::ClientCodec,
    message::{ClientMessage, MessagePayload},
    ChatResult,
};
use tokio::{io::AsyncBufReadExt, net::TcpStream};
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> ChatResult<()> {
    let socket = TcpStream::connect(("127.0.0.1", 8080)).await?;
    let mut stream = Framed::new(socket, ClientCodec);
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);

    let nick = loop {
        println!("/login <email> <password> or /signup <email> <password> to continue");
        let mut buf = vec![];

        reader.read_until(b'\n', &mut buf).await?;

        let msg = String::from_utf8(buf)?;
        let msg = msg.split(' ').collect::<Vec<_>>();
        if msg.len() != 3 {
            continue;
        }
        let client_message = if msg[0] == "/login" {
            ClientMessage::Login {
                email: msg[1].to_owned(),
                password: msg[2].to_owned(),
            }
        } else if msg[0] == "/signup" {
            ClientMessage::Signup {
                email: msg[1].to_owned(),
                password: msg[2].to_owned(),
            }
        } else {
            continue;
        };

        stream.send(client_message).await?;
        if let Some(msg) = stream.next().await {
            match msg?.payload {
                MessagePayload::SignupAccepted { nick } => break nick,
                MessagePayload::LoginAccepted { nick } => break nick,
                MessagePayload::LoginFail => Err("Invalid credentials")?,
                MessagePayload::SignupFail => Err("User already exists")?,
                _ => unreachable!("Can't receive chat messages during login phase"),
            }
        }
    };
    loop {
        let mut buf = vec![];
        tokio::select! {
          msg = stream.next() => {
            if let Some(msg) = msg {
              match msg?.payload {
                  MessagePayload::Nickname { new_nick, nick } => println!("{nick} changed their name to {new_nick}"),
                  MessagePayload::Message { nick, message } => println!("{nick}: {message}"),
                  MessagePayload::Connect { nick } => println!("{nick} joined the chat"),
                  MessagePayload::Disconnect { nick } => println!("{nick} left the chat"),
                  _ => unreachable!()
              }
            }
          },
          _ = reader.read_until(b'\n', &mut buf) => {
            buf.pop();
            let payload = String::from_utf8(buf)?;
            stream.send(ClientMessage::Message(payload)).await?;
          }
        }
    }
}
