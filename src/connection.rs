use crate::auth::Auth;
use crate::codec::ServerCodec;
use crate::message::{ClientMessage, MessagePayload, ServerMessage};
use crate::ChatResult;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio_util::codec::Framed;

pub async fn handle_login(
    socket: TcpStream,
    addr: SocketAddr,
    auth: Arc<Mutex<Auth>>,
    message_sender: mpsc::Sender<ServerMessage>,
    broadcast_receiver: broadcast::Receiver<ServerMessage>,
) -> ChatResult<()> {
    let mut stream = Framed::new(socket, ServerCodec);
    let nick = loop {
        if let Some(Ok(msg)) = stream.next().await {
            match msg {
                ClientMessage::Signup { nick, password } => {
                    let auth_result = {
                        let mut auth = auth.lock().unwrap();
                        auth.signup(nick, password)
                    };
                    if let Ok(nick) = auth_result {
                        stream
                            .send(ServerMessage::new(
                                addr,
                                MessagePayload::LoginAccepted { nick: nick.clone() },
                            ))
                            .await?;
                        break nick;
                    } else {
                        stream
                            .send(ServerMessage::new(addr, MessagePayload::LoginFail))
                            .await?;
                        Err("Signup failed")?;
                    }
                }
                ClientMessage::Login { nick, password } => {
                    let auth_result = {
                        let mut auth = auth.lock().unwrap();
                        auth.login(nick, password)
                    };
                    if let Ok(nick) = auth_result {
                        stream
                            .send(ServerMessage::new(
                                addr,
                                MessagePayload::LoginAccepted { nick: nick.clone() },
                            ))
                            .await?;
                        break nick;
                    } else {
                        stream
                            .send(ServerMessage::new(addr, MessagePayload::LoginFail))
                            .await?;
                        Err("Login failed")?;
                    }
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
        auth.clone(),
        message_sender,
        broadcast_receiver,
    ));
    Ok(())
}

pub async fn handle_connection(
    mut stream: Framed<TcpStream, ServerCodec>,
    addr: SocketAddr,
    mut nick: String,
    auth: Arc<Mutex<Auth>>,
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
                      ClientMessage::Nickname(new_nick) => {
                        let auth_result = {
                            let mut auth = auth.lock().unwrap();
                            auth.update_nick(nick.clone(), new_nick )
                        };
                        match auth_result {
                            Ok(new_nick) => MessagePayload::Nickname { new_nick, nick: nick.clone() },
                            Err(msg) =>  MessagePayload::NickChangeRefused { msg: msg.to_string() }
                        }
                      },
                      ClientMessage::Message(message) => MessagePayload::Message { nick: nick.clone(), message },
                      _ => unreachable!("Can't send login or signup if already logged in")
                  };
                    message_sender.send(ServerMessage::new(addr, payload)).await?;
                }

            }
            msg = broadcast_receiver.recv() => {
                let msg = msg?;
                // println!("Message received: {:?}", msg);
                match &msg.payload {
                    MessagePayload::Nickname { new_nick, .. } => {
                        if msg.sender == addr {
                            nick = new_nick.clone();
                          }
                    },
                    MessagePayload::NickChangeRefused { .. } => {
                        if msg.sender == addr {
                            stream.send(msg).await?;
                          }
                          continue;
                    },
                    _ => {}
                }

                stream.send(msg).await?;
            }

        }
    }
}
