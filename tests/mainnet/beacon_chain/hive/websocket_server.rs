use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;

pub async fn run_websocket_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
    Ok(())
}

async fn handle_connection(raw_stream: TcpStream) {
    let addr = raw_stream.peer_addr().unwrap();
    println!("New connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during WebSocket handshake");

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text from {}: {}", addr, text);
                let response = format!("Echo: {}", text);
                if let Err(e) = write.send(Message::Text(response)).await {
                    eprintln!("Error sending message to {}: {}", addr, e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client {} disconnected", addr);
                break;
            }
            Err(e) => {
                eprintln!("Error processing message from {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }
    println!("Connection closed: {}", addr);
}