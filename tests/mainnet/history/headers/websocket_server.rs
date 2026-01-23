use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }

    Ok(())
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(text) => {
                println!("Received text message: {}", text);
                if let Err(e) = sender.send(Message::Text(text)).await {
                    eprintln!("Error sending message: {}", e);
                    break;
                }
            }
            Message::Close(_) => {
                println!("Client disconnected");
                break;
            }
            _ => {}
        }
    }
}use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server listening on ws://127.0.0.1:8080");

    let clients = Arc::new(Mutex::new(HashSet::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let clients = Arc::clone(&clients);
        tokio::spawn(async move {
            if let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await {
                let (mut sender, mut receiver) = ws_stream.split();
                let addr = sender.get_ref().peer_addr().unwrap();
                println!("New client connected: {}", addr);

                {
                    let mut clients = clients.lock().await;
                    clients.insert(sender);
                }

                while let Some(Ok(msg)) = receiver.next().await {
                    match msg {
                        Message::Text(text) => {
                            println!("Received from {}: {}", addr, text);
                            let mut clients = clients.lock().await;
                            let mut disconnected = Vec::new();
                            for client in clients.iter_mut() {
                                if let Err(e) = client.send(Message::Text(text.clone())).await {
                                    eprintln!("Error sending message: {}", e);
                                    disconnected.push(client.clone());
                                }
                            }
                            for client in disconnected {
                                clients.remove(&client);
                            }
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                }

                {
                    let mut clients = clients.lock().await;
                    clients.remove(&sender);
                }
                println!("Client disconnected: {}", addr);
            }
        });
    }

    Ok(())
}