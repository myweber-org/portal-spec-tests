use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
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

async fn handle_connection(raw_stream: TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(raw_stream).await;
    if let Err(e) = ws_stream {
        eprintln!("Error during WebSocket handshake: {}", e);
        return;
    }

    let (mut ws_sender, mut ws_receiver) = ws_stream.unwrap().split();

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
                let echo_msg = Message::Text(format!("Echo: {}", text));
                if let Err(e) = ws_sender.send(echo_msg).await {
                    eprintln!("Error sending echo message: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client requested close");
                break;
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
            _ => {}
        }
    }
    println!("Connection closed");
}