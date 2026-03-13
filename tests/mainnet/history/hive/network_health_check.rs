
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use rand::Rng;

pub struct NetworkProbe {
    target: SocketAddr,
    timeout: Duration,
    packet_size: usize,
}

impl NetworkProbe {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        NetworkProbe {
            target: SocketAddr::new(IpAddr::V4(ip), port),
            timeout: Duration::from_secs(2),
            packet_size: 64,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn measure_latency(&self, samples: usize) -> Option<Duration> {
        let mut latencies = Vec::with_capacity(samples);
        
        for _ in 0..samples {
            if let Some(latency) = self.single_ping() {
                latencies.push(latency);
            }
        }

        if latencies.is_empty() {
            return None;
        }

        latencies.sort();
        let median_index = latencies.len() / 2;
        Some(latencies[median_index])
    }

    pub fn calculate_packet_loss(&self, attempts: usize) -> f64 {
        let mut successful = 0;
        
        for _ in 0..attempts {
            if self.single_ping().is_some() {
                successful += 1;
            }
        }

        let loss_percentage = ((attempts - successful) as f64 / attempts as f64) * 100.0;
        loss_percentage
    }

    fn single_ping(&self) -> Option<Duration> {
        let start = Instant::now();
        
        let mut rng = rand::thread_rng();
        let payload: Vec<u8> = (0..self.packet_size)
            .map(|_| rng.gen())
            .collect();

        let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.set_read_timeout(Some(self.timeout)).ok()?;
        
        socket.send_to(&payload, self.target).ok()?;
        
        let mut buffer = vec![0u8; self.packet_size];
        socket.recv_from(&mut buffer).ok()?;
        
        Some(start.elapsed())
    }
}

pub fn check_network_quality(target_ip: Ipv4Addr) -> String {
    let probe = NetworkProbe::new(target_ip, 53);
    
    let latency = probe.measure_latency(5);
    let packet_loss = probe.calculate_packet_loss(10);
    
    match latency {
        Some(lat) => {
            if lat < Duration::from_millis(50) && packet_loss < 1.0 {
                "Network quality: Excellent".to_string()
            } else if lat < Duration::from_millis(100) && packet_loss < 5.0 {
                "Network quality: Good".to_string()
            } else if lat < Duration::from_millis(200) && packet_loss < 10.0 {
                "Network quality: Fair".to_string()
            } else {
                "Network quality: Poor".to_string()
            }
        }
        None => "Target unreachable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_localhost_probe() {
        let localhost = Ipv4Addr::new(127, 0, 0, 1);
        let probe = NetworkProbe::new(localhost, 8080);
        
        let result = probe.single_ping();
        assert!(result.is_none());
    }
    
    #[test]
    fn test_packet_loss_calculation() {
        let google_dns = Ipv4Addr::new(8, 8, 8, 8);
        let probe = NetworkProbe::new(google_dns, 53);
        
        let loss = probe.calculate_packet_loss(3);
        assert!(loss >= 0.0 && loss <= 100.0);
    }
}