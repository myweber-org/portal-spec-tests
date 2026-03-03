use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket echo server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    let (mut write, mut read) = ws_stream.split();
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                if let Err(e) = write.send(Message::Text(text)).await {
                                    eprintln!("Error sending message: {}", e);
                                    break;
                                }
                            }
                            Ok(Message::Close(_)) => {
                                break;
                            }
                            Err(e) => {
                                eprintln!("Error reading message: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => eprintln!("Error during WebSocket handshake: {}", e),
            }
        });
    }
    Ok(())
}