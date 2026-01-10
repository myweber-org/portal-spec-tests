
use std::net::{TcpStream, IpAddr};
use std::time::Duration;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

const TIMEOUT: Duration = Duration::from_secs(2);
const PORTS_TO_CHECK: [u16; 5] = [80, 443, 22, 53, 3389];

pub struct NetworkCheckResult {
    pub host: String,
    pub ping_success: bool,
    pub open_ports: Vec<u16>,
    pub total_time_ms: u128,
}

pub fn check_host_connectivity(host: &str) -> NetworkCheckResult {
    let start_time = std::time::Instant::now();
    
    let ping_result = ping_host(host);
    let open_ports = scan_ports(host);
    
    NetworkCheckResult {
        host: host.to_string(),
        ping_success: ping_result,
        open_ports,
        total_time_ms: start_time.elapsed().as_millis(),
    }
}

fn ping_host(host: &str) -> bool {
    let output = std::process::Command::new("ping")
        .arg("-c")
        .arg("2")
        .arg("-W")
        .arg("1")
        .arg(host)
        .output();
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn scan_ports(host: &str) -> Vec<u16> {
    let (tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
    
    for &port in &PORTS_TO_CHECK {
        let tx = tx.clone();
        let host = host.to_string();
        
        thread::spawn(move || {
            if check_port(&host, port) {
                tx.send(port).unwrap();
            }
        });
    }
    
    drop(tx);
    
    let mut open_ports = Vec::new();
    for port in rx {
        open_ports.push(port);
    }
    
    open_ports.sort();
    open_ports
}

fn check_port(host: &str, port: u16) -> bool {
    let addr_string = format!("{}:{}", host, port);
    
    match addr_string.parse::<std::net::SocketAddr>() {
        Ok(addr) => {
            if let Ok(stream) = TcpStream::connect_timeout(&addr, TIMEOUT) {
                drop(stream);
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

pub fn display_results(result: &NetworkCheckResult) {
    println!("Network Health Check for: {}", result.host);
    println!("Ping successful: {}", result.ping_success);
    println!("Open ports: {:?}", result.open_ports);
    println!("Total check time: {}ms", result.total_time_ms);
    
    if result.ping_success && !result.open_ports.is_empty() {
        println!("Status: HEALTHY");
    } else if result.ping_success {
        println!("Status: PARTIAL (ping works but no common ports open)");
    } else {
        println!("Status: UNREACHABLE");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_localhost_connectivity() {
        let result = check_host_connectivity("127.0.0.1");
        assert!(result.ping_success || !result.open_ports.is_empty());
    }
    
    #[test]
    fn test_port_check_invalid_host() {
        assert!(!check_port("invalid.host.name", 80));
    }
}