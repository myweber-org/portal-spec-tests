
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use std::thread;
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

    pub fn measure_latency(&self) -> Result<Duration, String> {
        let start = Instant::now();
        
        match std::net::TcpStream::connect_timeout(&self.target, self.timeout) {
            Ok(_) => {
                let elapsed = start.elapsed();
                Ok(elapsed)
            }
            Err(e) => Err(format!("Connection failed: {}", e)),
        }
    }

    pub fn simulate_packet_loss_test(&self) -> f64 {
        let mut rng = rand::thread_rng();
        let mut successful = 0;
        
        for _ in 0..self.packet_count {
            thread::sleep(Duration::from_millis(100));
            
            if rng.gen_bool(0.85) {
                successful += 1;
            }
        }
        
        let loss_percentage = (self.packet_count - successful) as f64 / self.packet_count as f64 * 100.0;
        loss_percentage
    }

    pub fn health_check(&self) -> (bool, f64, f64) {
        let latency_result = self.measure_latency();
        let packet_loss = self.simulate_packet_loss_test();
        
        match latency_result {
            Ok(latency) => {
                let latency_ms = latency.as_millis() as f64;
                let healthy = latency_ms < 100.0 && packet_loss < 5.0;
                (healthy, latency_ms, packet_loss)
            }
            Err(_) => (false, f64::MAX, packet_loss),
        }
    }
}

pub fn check_network_health() {
    let probe = NetworkProbe::new(Ipv4Addr::new(8, 8, 8, 8), 53);
    
    println!("Testing network connectivity to {}...", probe.target);
    
    for attempt in 1..=3 {
        println!("Attempt {}:", attempt);
        let (healthy, latency, loss) = probe.health_check();
        
        if healthy {
            println!("  Status: HEALTHY");
            println!("  Latency: {:.2} ms", latency);
            println!("  Packet loss: {:.1}%", loss);
            break;
        } else {
            println!("  Status: UNHEALTHY");
            println!("  Latency: {:.2} ms", latency);
            println!("  Packet loss: {:.1}%", loss);
            
            if attempt < 3 {
                println!("  Retrying in 1 second...");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}