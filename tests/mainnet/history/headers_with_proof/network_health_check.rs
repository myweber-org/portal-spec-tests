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
}