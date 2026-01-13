use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::thread;

const MAX_RETRIES: u32 = 3;
const CONNECTION_TIMEOUT_SECS: u64 = 5;

fn test_connectivity(host: &str, port: u16) -> bool {
    let addr = format!("{}:{}", host, port);
    let socket_addr: SocketAddr = match addr.parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };

    for attempt in 1..=MAX_RETRIES {
        println!("Attempt {} to connect to {}:{}", attempt, host, port);
        
        match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(CONNECTION_TIMEOUT_SECS)) {
            Ok(_) => {
                println!("Successfully connected to {}:{}", host, port);
                return true;
            }
            Err(e) => {
                println!("Connection attempt {} failed: {}", attempt, e);
                if attempt < MAX_RETRIES {
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }
    
    false
}

fn main() {
    let test_hosts = vec![
        ("google.com", 80),
        ("github.com", 443),
        ("example.com", 80),
    ];

    for (host, port) in test_hosts {
        let status = test_connectivity(host, port);
        println!("{}:{} - {}", host, port, if status { "REACHABLE" } else { "UNREACHABLE" });
    }
}