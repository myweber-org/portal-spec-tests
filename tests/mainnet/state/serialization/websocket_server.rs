use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await?;
        println!("New WebSocket connection established");

        tokio::spawn(handle_connection(ws_stream));
    }
    Ok(())
}

async fn handle_connection(ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(message) = receiver.next().await {
        match message {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    let echo_msg = msg.clone();
                    if sender.send(echo_msg).await.is_err() {
                        break;
                    }
                } else if msg.is_close() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    println!("WebSocket connection closed");
}