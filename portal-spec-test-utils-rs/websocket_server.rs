use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};

pub async fn run_websocket_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
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

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if let Err(e) = write.send(msg).await {
                    eprintln!("Error sending message: {}", e);
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