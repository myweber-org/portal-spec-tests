use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use rand::Rng;

const PACKET_SIZE: usize = 64;
const TIMEOUT_MS: u64 = 1000;
const MAX_PACKETS: usize = 10;

pub struct NetworkHealth {
    pub avg_latency_ms: f64,
    pub packet_loss_percent: f64,
    pub jitter_ms: f64,
}

pub fn check_network_health(target: IpAddr, port: u16) -> Result<NetworkHealth, String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind socket: {}", e))?;
    
    socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    let target_addr = SocketAddr::new(target, port);
    let mut rng = rand::thread_rng();
    let mut latencies = Vec::new();
    let mut lost_packets = 0;

    for seq in 0..MAX_PACKETS {
        let mut buffer = [0u8; PACKET_SIZE];
        buffer[0..8].copy_from_slice(&seq.to_be_bytes());
        buffer[8..16].copy_from_slice(&rng.gen::<u64>().to_be_bytes());

        let send_time = Instant::now();
        
        if socket.send_to(&buffer, target_addr).is_err() {
            lost_packets += 1;
            continue;
        }

        let mut recv_buffer = [0u8; PACKET_SIZE];
        match socket.recv_from(&mut recv_buffer) {
            Ok((size, _)) if size == PACKET_SIZE => {
                let recv_time = Instant::now();
                let latency = recv_time.duration_since(send_time).as_micros() as f64 / 1000.0;
                latencies.push(latency);
            }
            _ => {
                lost_packets += 1;
            }
        }
    }

    if latencies.is_empty() {
        return Err("All packets lost".to_string());
    }

    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    
    let variance: f64 = latencies.iter()
        .map(|&l| (l - avg_latency).powi(2))
        .sum::<f64>() / latencies.len() as f64;
    
    let jitter = variance.sqrt();

    Ok(NetworkHealth {
        avg_latency_ms: avg_latency,
        packet_loss_percent: (lost_packets as f64 / MAX_PACKETS as f64) * 100.0,
        jitter_ms: jitter,
    })
}

pub fn is_network_healthy(health: &NetworkHealth, max_latency: f64, max_loss: f64) -> bool {
    health.avg_latency_ms <= max_latency && health.packet_loss_percent <= max_loss
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_health_check_invalid_target() {
        let target = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1));
        let result = check_network_health(target, 9999);
        assert!(result.is_err());
    }

    #[test]
    fn test_health_thresholds() {
        let health = NetworkHealth {
            avg_latency_ms: 50.0,
            packet_loss_percent: 5.0,
            jitter_ms: 10.0,
        };
        
        assert!(is_network_healthy(&health, 100.0, 10.0));
        assert!(!is_network_healthy(&health, 30.0, 10.0));
    }
}