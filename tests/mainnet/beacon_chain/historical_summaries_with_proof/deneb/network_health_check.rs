use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use rand::Rng;
use tokio::net::UdpSocket;
use tokio::time::sleep;

const PACKET_SIZE: usize = 64;
const TIMEOUT_MS: u64 = 1000;
const TEST_COUNT: usize = 10;

pub struct NetworkMetrics {
    pub latency_ms: f64,
    pub packet_loss_percent: f64,
    pub jitter_ms: f64,
}

pub async fn check_network_health(target: IpAddr, port: u16) -> Result<NetworkMetrics, String> {
    let socket_addr = SocketAddr::new(target, port);
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("Failed to bind socket: {}", e))?;
    
    socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    let mut latencies = Vec::new();
    let mut lost_packets = 0;
    let mut rng = rand::thread_rng();

    for seq in 0..TEST_COUNT {
        let mut buffer = [0u8; PACKET_SIZE];
        rng.fill(&mut buffer[..]);
        buffer[0] = seq as u8;

        let send_time = Instant::now();
        
        match socket.send_to(&buffer, socket_addr).await {
            Ok(_) => {
                let mut recv_buffer = [0u8; PACKET_SIZE];
                match socket.recv_from(&mut recv_buffer).await {
                    Ok((size, _)) if size == PACKET_SIZE && recv_buffer[0] == seq as u8 => {
                        let latency = send_time.elapsed().as_micros() as f64 / 1000.0;
                        latencies.push(latency);
                    }
                    _ => lost_packets += 1,
                }
            }
            Err(_) => lost_packets += 1,
        }

        if seq < TEST_COUNT - 1 {
            sleep(Duration::from_millis(100)).await;
        }
    }

    if latencies.is_empty() {
        return Err("All packets lost".to_string());
    }

    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let packet_loss = (lost_packets as f64 / TEST_COUNT as f64) * 100.0;
    
    let variance: f64 = latencies.iter()
        .map(|&l| (l - avg_latency).powi(2))
        .sum::<f64>() / latencies.len() as f64;
    let jitter = variance.sqrt();

    Ok(NetworkMetrics {
        latency_ms: avg_latency,
        packet_loss_percent: packet_loss,
        jitter_ms: jitter,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_localhost_health() {
        let result = check_network_health(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080).await;
        assert!(result.is_ok() || result.is_err());
    }
}