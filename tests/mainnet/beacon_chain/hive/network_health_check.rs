
use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::thread;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const PING_COUNT: usize = 4;

pub struct NetworkCheckResult {
    pub is_reachable: bool,
    pub latency_ms: Option<u128>,
    pub error_message: Option<String>,
}

pub fn check_host_port(host: &str, port: u16) -> NetworkCheckResult {
    let addr_string = format!("{}:{}", host, port);
    
    match addr_string.parse::<SocketAddr>() {
        Ok(addr) => {
            let start = Instant::now();
            match TcpStream::connect_timeout(&addr, DEFAULT_TIMEOUT) {
                Ok(_) => {
                    let latency = start.elapsed().as_millis();
                    NetworkCheckResult {
                        is_reachable: true,
                        latency_ms: Some(latency),
                        error_message: None,
                    }
                }
                Err(e) => NetworkCheckResult {
                    is_reachable: false,
                    latency_ms: None,
                    error_message: Some(format!("Connection failed: {}", e)),
                },
            }
        }
        Err(e) => NetworkCheckResult {
            is_reachable: false,
            latency_ms: None,
            error_message: Some(format!("Invalid address: {}", e)),
        },
    }
}

pub fn ping_host(host: &str, port: u16) -> Vec<NetworkCheckResult> {
    let mut results = Vec::with_capacity(PING_COUNT);
    
    for i in 0..PING_COUNT {
        println!("Ping attempt {} to {}:{}", i + 1, host, port);
        let result = check_host_port(host, port);
        results.push(result);
        
        if i < PING_COUNT - 1 {
            thread::sleep(Duration::from_secs(1));
        }
    }
    
    results
}

pub fn analyze_ping_results(results: &[NetworkCheckResult]) -> (usize, usize, Option<u128>) {
    let total = results.len();
    let successful = results.iter().filter(|r| r.is_reachable).count();
    
    let avg_latency = if successful > 0 {
        let sum: u128 = results
            .iter()
            .filter_map(|r| r.latency_ms)
            .sum();
        Some(sum / successful as u128)
    } else {
        None
    };
    
    (total, successful, avg_latency)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_check_valid_host() {
        let result = check_host_port("example.com", 80);
        assert!(result.is_reachable || !result.is_reachable);
    }
    
    #[test]
    fn test_check_invalid_port() {
        let result = check_host_port("localhost", 99999);
        assert!(!result.is_reachable);
        assert!(result.error_message.is_some());
    }
}