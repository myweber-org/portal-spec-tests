
use std::process::Command;
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr};

const PACKET_COUNT: usize = 4;
const TIMEOUT_SECONDS: u64 = 2;

#[derive(Debug)]
pub struct NetworkMetrics {
    pub destination: String,
    pub packets_sent: usize,
    pub packets_received: usize,
    pub packet_loss: f32,
    pub avg_latency_ms: Option<f32>,
    pub max_latency_ms: Option<f32>,
    pub min_latency_ms: Option<f32>,
}

impl NetworkMetrics {
    pub fn new(destination: &str) -> Self {
        NetworkMetrics {
            destination: destination.to_string(),
            packets_sent: 0,
            packets_received: 0,
            packet_loss: 0.0,
            avg_latency_ms: None,
            max_latency_ms: None,
            min_latency_ms: None,
        }
    }
}

pub fn check_connectivity(destination: &str) -> Result<NetworkMetrics, String> {
    let mut metrics = NetworkMetrics::new(destination);
    let mut latencies = Vec::new();

    for _ in 0..PACKET_COUNT {
        let start = Instant::now();
        
        let output = Command::new("ping")
            .arg("-c")
            .arg("1")
            .arg("-W")
            .arg(TIMEOUT_SECONDS.to_string())
            .arg(destination)
            .output();

        match output {
            Ok(output) => {
                metrics.packets_sent += 1;
                
                if output.status.success() {
                    metrics.packets_received += 1;
                    let duration = start.elapsed();
                    latencies.push(duration.as_millis() as f32);
                }
            }
            Err(e) => {
                return Err(format!("Failed to execute ping command: {}", e));
            }
        }
    }

    if metrics.packets_sent > 0 {
        metrics.packet_loss = ((metrics.packets_sent - metrics.packets_received) as f32 / metrics.packets_sent as f32) * 100.0;
    }

    if !latencies.is_empty() {
        metrics.avg_latency_ms = Some(latencies.iter().sum::<f32>() / latencies.len() as f32);
        metrics.max_latency_ms = latencies.iter().copied().reduce(f32::max);
        metrics.min_latency_ms = latencies.iter().copied().reduce(f32::min);
    }

    Ok(metrics)
}

pub fn is_local_address(ip: &str) -> bool {
    if let Ok(parsed_ip) = ip.parse::<IpAddr>() {
        match parsed_ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private() || 
                ipv4.is_loopback() || 
                ipv4.is_link_local() ||
                ipv4 == Ipv4Addr::new(0, 0, 0, 0)
            }
            IpAddr::V6(_) => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_address_detection() {
        assert!(is_local_address("127.0.0.1"));
        assert!(is_local_address("192.168.1.1"));
        assert!(is_local_address("10.0.0.1"));
        assert!(!is_local_address("8.8.8.8"));
    }

    #[test]
    fn test_network_metrics_creation() {
        let metrics = NetworkMetrics::new("example.com");
        assert_eq!(metrics.destination, "example.com");
        assert_eq!(metrics.packets_sent, 0);
        assert_eq!(metrics.packets_received, 0);
        assert_eq!(metrics.packet_loss, 0.0);
        assert!(metrics.avg_latency_ms.is_none());
    }
}