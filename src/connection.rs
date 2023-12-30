use crate::codec::ServerCodec;
use crate::message::{ClientMessage, MessagePayload, ServerMessage};
use crate::ChatResult;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio_util::codec::Framed;

pub async fn handle_login(
    socket: TcpStream,
    addr: SocketAddr,
    message_sender: mpsc::Sender<ServerMessage>,
    broadcast_receiver: broadcast::Receiver<ServerMessage>,
) -> ChatResult<()> {
    let mut stream = Framed::new(socket, ServerCodec);
    let nick = loop {
        if let Some(Ok(msg)) = stream.next().await {
            match msg {
                ClientMessage::Signup { email, .. } => {
                    stream
                        .send(ServerMessage::new(
                            addr,
                            MessagePayload::LoginAccepted {
                                nick: String::from("Heyo"),
                            },
                        ))
                        .await?;
                    break email;
                }
                ClientMessage::Login { email, .. } => {
                    stream
                        .send(ServerMessage::new(
                            addr,
                            MessagePayload::LoginAccepted {
                                nick: String::from("Heyo"),
                            },
                        ))
                        .await?;
                    break email;
                }

                _ => continue,
            }
        } else {
            continue;
        }
    };
    tokio::spawn(handle_connection(
        stream,
        addr,
        nick,
        message_sender,
        broadcast_receiver,
    ));
    Ok(())
}

pub async fn handle_connection(
    mut stream: Framed<TcpStream, ServerCodec>,
    addr: SocketAddr,
    mut nick: String,
    message_sender: mpsc::Sender<ServerMessage>,
    mut broadcast_receiver: broadcast::Receiver<ServerMessage>,
) -> ChatResult<()> {
    message_sender
        .send(ServerMessage::new(
            addr,
            MessagePayload::Connect { nick: nick.clone() },
        ))
        .await?;

    loop {
        tokio::select! {
            msg = stream.next() => {
                if let Some(Ok(msg)) = msg {
                  let payload = match msg {
                      ClientMessage::Nickname(new_name) => MessagePayload::Nickname { new_nick: new_name, nick: nick.clone() },
                      ClientMessage::Message(message) => MessagePayload::Message { nick: nick.clone(), message },
                      _ => unreachable!("Can't send login or signup if already logged in")
                  };
                    message_sender.send(ServerMessage::new(addr, payload)).await?;
                }

            }
            msg = broadcast_receiver.recv() => {
                let msg = msg?;
                // println!("Message received: {:?}", msg);
                if let MessagePayload::Nickname { new_nick, .. } = &msg.payload {
                  if msg.sender == addr {
                    nick = new_nick.clone();
                  }
                }

                stream.send(msg).await?;
            }

        }
    }
}
