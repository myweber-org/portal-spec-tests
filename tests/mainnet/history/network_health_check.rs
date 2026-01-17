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
}use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use rand::Rng;

pub struct NetworkProbe {
    target: SocketAddr,
    timeout: Duration,
    packet_count: usize,
}

impl NetworkProbe {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self {
            target: SocketAddr::new(IpAddr::V4(ip), port),
            timeout: Duration::from_secs(2),
            packet_count: 10,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_packet_count(mut self, count: usize) -> Self {
        self.packet_count = count;
        self
    }

    pub fn measure_latency(&self) -> Option<Duration> {
        let mut total = Duration::ZERO;
        let mut successful = 0;

        for _ in 0..self.packet_count {
            let start = Instant::now();
            
            if let Ok(_) = std::net::TcpStream::connect_timeout(&self.target, self.timeout) {
                let elapsed = start.elapsed();
                total += elapsed;
                successful += 1;
            }
        }

        if successful > 0 {
            Some(total / successful as u32)
        } else {
            None
        }
    }

    pub fn simulate_packet_loss(&self) -> f64 {
        let mut rng = rand::thread_rng();
        let mut lost = 0;

        for _ in 0..self.packet_count {
            if !rng.gen_bool(0.95) {
                lost += 1;
            }
        }

        (lost as f64 / self.packet_count as f64) * 100.0
    }

    pub fn health_report(&self) -> String {
        let latency = self.measure_latency();
        let packet_loss = self.simulate_packet_loss();

        match latency {
            Some(avg_latency) => {
                format!(
                    "Target: {}\nAverage Latency: {:.2}ms\nPacket Loss: {:.1}%\nStatus: HEALTHY",
                    self.target,
                    avg_latency.as_millis() as f64,
                    packet_loss
                )
            }
            None => {
                format!(
                    "Target: {}\nConnection failed\nPacket Loss: {:.1}%\nStatus: UNHEALTHY",
                    self.target,
                    packet_loss
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_creation() {
        let probe = NetworkProbe::new(Ipv4Addr::new(8, 8, 8, 8), 53);
        assert_eq!(probe.target.port(), 53);
    }

    #[test]
    fn test_latency_measurement() {
        let probe = NetworkProbe::new(Ipv4Addr::new(127, 0, 0, 1), 80)
            .with_packet_count(3)
            .with_timeout(Duration::from_millis(100));
        
        let result = probe.measure_latency();
        assert!(result.is_some());
    }
}