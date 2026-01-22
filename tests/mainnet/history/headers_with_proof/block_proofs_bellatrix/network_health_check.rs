use std::net::TcpStream;
use std::time::Duration;
use std::thread;

const HOST: &str = "example.com";
const PORT: u16 = 80;
const TIMEOUT_SECS: u64 = 5;
const MAX_RETRIES: u8 = 3;

fn test_connection(host: &str, port: u16, timeout: Duration) -> bool {
    match TcpStream::connect_timeout(&format!("{}:{}", host, port).parse().unwrap(), timeout) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn main() {
    let timeout_duration = Duration::from_secs(TIMEOUT_SECS);
    
    for attempt in 1..=MAX_RETRIES {
        println!("Connection attempt {} to {}:{}", attempt, HOST, PORT);
        
        if test_connection(HOST, PORT, timeout_duration) {
            println!("Network connection successful!");
            return;
        }
        
        if attempt < MAX_RETRIES {
            println!("Connection failed. Retrying in 2 seconds...");
            thread::sleep(Duration::from_secs(2));
        }
    }
    
    println!("Failed to establish connection after {} attempts", MAX_RETRIES);
}