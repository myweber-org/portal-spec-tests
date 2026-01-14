use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("New WebSocket connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Failed to accept WebSocket connection");

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                println!("Received message from {}: {:?}", addr, msg);
                if let Message::Text(text) = msg {
                    let echo_msg = Message::Text(format!("Echo: {}", text));
                    if let Err(e) = write.send(echo_msg).await {
                        eprintln!("Failed to send echo message to {}: {}", addr, e);
                        break;
                    }
                } else if msg.is_close() {
                    println!("Client {} closed the connection.", addr);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message from {}: {}", addr, e);
                break;
            }
        }
    }
    println!("Connection closed for: {}", addr);
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind to address");
    println!("WebSocket echo server listening on ws://{}", addr);

    while let Ok((stream, client_addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, client_addr));
    }
}