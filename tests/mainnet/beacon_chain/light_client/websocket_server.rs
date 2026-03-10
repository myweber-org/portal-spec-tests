use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("New WebSocket connection from: {}", addr);
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(message) => {
                if message.is_text() {
                    let text = message.to_text().unwrap();
                    println!("Received text message: {}", text);
                    let response = format!("Echo: {}", text);
                    if let Err(e) = write.send(response.into()).await {
                        eprintln!("Error sending message: {}", e);
                        break;
                    }
                } else if message.is_close() {
                    println!("Client closed connection");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }
    println!("Connection closed: {}", addr);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(async move {
            handle_connection(stream, addr).await;
        });
    }
    Ok(())
}use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("New WebSocket connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Failed to accept WebSocket connection");

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message from {}: {}", addr, text);
                let echo_msg = Message::Text(format!("Echo: {}", text));
                if let Err(e) = sender.send(echo_msg).await {
                    eprintln!("Failed to send echo message to {}: {}", addr, e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client {} closed the connection", addr);
                break;
            }
            Err(e) => {
                eprintln!("Error receiving message from {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }
    println!("Connection closed for: {}", addr);
}

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    loop {
        let (stream, client_addr) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(stream, client_addr).await;
        });
    }
}
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
    Ok(())
}

async fn handle_connection(stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    if let Err(e) = write.send(msg).await {
                        eprintln!("Error sending message: {}", e);
                        break;
                    }
                } else if msg.is_close() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }
}