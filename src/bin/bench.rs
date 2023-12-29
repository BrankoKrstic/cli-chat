use std::time::Instant;

use futures_util::{SinkExt, StreamExt};
use rust_chat::{
    codec::{ChatFrame, ChatFrameCodec, ChatFrameTag},
    ChatResult,
};
use tokio::{net::TcpStream, task::JoinSet};
use tokio_util::codec::Framed;

async fn handle_connection(count: i32, idx: i32) -> ChatResult<()> {
    let s = String::from("Test message");

    let mut messages_received = 0;
    let messages_max = 10 * (count - 1);
    let mut messages_sent = 0;
    let socket = TcpStream::connect(("127.0.0.1", 8080)).await?;
    let (mut sink, mut stream) = Framed::new(socket, ChatFrameCodec).split();
    // tokio::time::sleep(Duration::from_millis((count * 100) as u64)).await;
    loop {
        // println!("process {idx} {messages_received} {messages_sent}");
        if messages_received >= messages_max && messages_sent >= count {
            break;
        }
        if messages_received >= messages_max {
            let _ = sink
                .send(ChatFrame {
                    tag: ChatFrameTag::Message,
                    payload: s.clone(),
                })
                .await;
            messages_sent += 1;
        } else if messages_sent >= count {
            let _ = stream.next().await;
            messages_received += 1;
        } else {
            tokio::select! {
              msg = stream.next() => {
                if msg.is_some() && msg.unwrap().is_ok() {
                    messages_received += 1;
                }
              },
              _ = sink.send(ChatFrame {
                tag: ChatFrameTag::Message,
                payload: s.clone()
              }) => {
                messages_sent += 1;
              }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ChatResult<()> {
    let v = vec![1, 10, 100, 1000];

    for count in v {
        let start_time = Instant::now();
        let mut set = JoinSet::new();
        for i in 0..count {
            set.spawn(handle_connection(count, i));
        }
        while set.join_next().await.is_some() {}
        let elapsed_time = start_time.elapsed();

        println!(
            "Processed {} messages in {}ms",
            10 * count * count,
            elapsed_time.as_millis()
        )
    }
    Ok(())
}
