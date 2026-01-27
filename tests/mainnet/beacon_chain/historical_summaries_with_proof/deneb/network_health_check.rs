use std::net::TcpStream;
use std::time::Duration;
use std::thread;

const RETRY_COUNT: u8 = 3;
const CONNECTION_TIMEOUT: u64 = 5;

fn test_connection(host: &str, port: u16) -> bool {
    match TcpStream::connect_timeout(
        &format!("{}:{}", host, port).parse().unwrap(),
        Duration::from_secs(CONNECTION_TIMEOUT)
    ) {
        Ok(_) => true,
        Err(_) => false
    }
}

fn main() {
    let endpoints = vec![
        ("google.com", 80),
        ("github.com", 443),
        ("localhost", 8080)
    ];

    for (host, port) in endpoints {
        println!("Testing connection to {}:{}", host, port);
        
        let mut connected = false;
        for attempt in 1..=RETRY_COUNT {
            print!("  Attempt {}... ", attempt);
            
            if test_connection(host, port) {
                println!("SUCCESS");
                connected = true;
                break;
            } else {
                println!("FAILED");
                if attempt < RETRY_COUNT {
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
        
        if !connected {
            println!("  All connection attempts failed for {}:{}", host, port);
        }
    }
}