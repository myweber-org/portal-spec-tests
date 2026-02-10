use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;
use std::thread;

struct Server {
    host: IpAddr,
    port: u16,
    name: String,
}

impl Server {
    fn new(host: IpAddr, port: u16, name: &str) -> Self {
        Server {
            host,
            port,
            name: name.to_string(),
        }
    }

    fn check_connection(&self, timeout_secs: u64) -> bool {
        let socket = SocketAddr::new(self.host, self.port);
        match TcpStream::connect_timeout(&socket, Duration::from_secs(timeout_secs)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

fn main() {
    let servers = vec![
        Server::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53, "Google DNS"),
        Server::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53, "Cloudflare DNS"),
        Server::new(IpAddr::V4(Ipv4Addr::new(208, 67, 222, 222)), 53, "OpenDNS"),
    ];

    let mut handles = vec![];

    for server in servers {
        let handle = thread::spawn(move || {
            let is_up = server.check_connection(3);
            let status = if is_up { "UP" } else { "DOWN" };
            println!("{} ({}) is {}", server.name, server.host, status);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::thread;

const RETRY_COUNT: u32 = 3;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

fn test_connectivity(host: &str, port: u16) -> Result<(), String> {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;

    for attempt in 1..=RETRY_COUNT {
        println!("Attempt {} to connect to {}:{}", attempt, host, port);
        
        match TcpStream::connect_timeout(&addr, CONNECTION_TIMEOUT) {
            Ok(_) => {
                println!("Successfully connected to {}:{}", host, port);
                return Ok(());
            }
            Err(e) => {
                println!("Connection failed: {}", e);
                if attempt < RETRY_COUNT {
                    println!("Retrying in 2 seconds...");
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }
    
    Err(format!("Failed to connect to {}:{} after {} attempts", host, port, RETRY_COUNT))
}

fn main() {
    let test_hosts = vec![
        ("google.com", 80),
        ("github.com", 443),
        ("localhost", 8080),
    ];

    for (host, port) in test_hosts {
        match test_connectivity(host, port) {
            Ok(_) => println!("{}:{} - PASS", host, port),
            Err(e) => println!("{}:{} - FAIL: {}", host, port, e),
        }
        println!("---");
    }
}