use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

async fn handle_connection(stream: TcpStream, sender: broadcast::Sender<String>) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during WebSocket handshake");

    let (mut write, mut read) = ws_stream.split();
    let mut rx = sender.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if write.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                println!("Received: {}", text);
                if sender.send(text).is_err() {
                    break;
                }
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    let (tx, _) = broadcast::channel(32);
    let tx = Arc::new(tx);

    while let Ok((stream, _)) = listener.accept().await {
        let tx_clone = Arc::clone(&tx);
        tokio::spawn(async move {
            handle_connection(stream, tx_clone).await;
        });
    }

    Ok(())
}