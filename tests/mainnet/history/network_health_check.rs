use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};

const MAX_RETRIES: u32 = 3;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_DELAY: Duration = Duration::from_secs(1);

pub struct NetworkChecker {
    target: SocketAddr,
    retry_count: u32,
}

impl NetworkChecker {
    pub fn new(target: SocketAddr) -> Self {
        NetworkChecker {
            target,
            retry_count: 0,
        }
    }

    pub fn check_connection(&mut self) -> Result<Duration, String> {
        while self.retry_count < MAX_RETRIES {
            let start_time = Instant::now();
            
            match TcpStream::connect_timeout(&self.target, CONNECTION_TIMEOUT) {
                Ok(mut stream) => {
                    let elapsed = start_time.elapsed();
                    
                    // Send a simple test message
                    let test_data = b"PING";
                    if let Err(e) = stream.write_all(test_data) {
                        return Err(format!("Write failed after connection: {}", e));
                    }
                    
                    self.retry_count = 0;
                    return Ok(elapsed);
                }
                Err(e) => {
                    self.retry_count += 1;
                    
                    if self.retry_count >= MAX_RETRIES {
                        return Err(format!("Failed after {} retries: {}", MAX_RETRIES, e));
                    }
                    
                    std::thread::sleep(RETRY_DELAY);
                }
            }
        }
        
        Err("Max retries exceeded".to_string())
    }

    pub fn is_healthy(&mut self) -> bool {
        self.check_connection().is_ok()
    }
}

pub fn check_multiple_hosts(hosts: &[SocketAddr]) -> Vec<(SocketAddr, bool, Option<Duration>)> {
    let mut results = Vec::new();
    
    for &host in hosts {
        let mut checker = NetworkChecker::new(host);
        match checker.check_connection() {
            Ok(duration) => results.push((host, true, Some(duration))),
            Err(_) => results.push((host, false, None)),
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_network_checker_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let checker = NetworkChecker::new(addr);
        assert_eq!(checker.retry_count, 0);
    }

    #[test]
    fn test_check_multiple_hosts_empty() {
        let results = check_multiple_hosts(&[]);
        assert!(results.is_empty());
    }
}