
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("New WebSocket connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Failed to accept WebSocket connection");

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        println!("Received text message from {}: {}", addr, text);
                        let response = format!("Echo: {}", text);
                        if let Err(e) = write.send(Message::Text(response)).await {
                            eprintln!("Failed to send message to {}: {}", addr, e);
                            break;
                        }
                    }
                    Message::Close(_) => {
                        println!("Client {} disconnected", addr);
                        break;
                    }
                    _ => {
                        println!("Received non-text message from {}", addr);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading message from {}: {}", addr, e);
                break;
            }
        }
    }
    println!("Connection with {} closed", addr);
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind to address");
    println!("WebSocket server listening on ws://{}", addr);

    while let Ok((stream, client_addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, client_addr));
    }
}