
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("WebSocket echo server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(raw_stream: tokio::net::TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
                let echo_msg = Message::Text(text);
                ws_sender.send(echo_msg).await.unwrap();
            }
            Ok(Message::Close(_)) => {
                println!("Client disconnected");
                break;
            }
            Ok(other) => {
                println!("Received other message type: {:?}", other);
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }
}