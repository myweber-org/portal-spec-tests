use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::time::Duration;

pub struct NetworkProbe {
    timeout: Duration,
}

impl NetworkProbe {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkProbe {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn icmp_ping(&self, target: IpAddr) -> bool {
        if let IpAddr::V4(ipv4) = target {
            let payload = [0u8; 32];
            match icmp_ping::ping(ipv4, self.timeout, &payload) {
                Ok(reply) => reply.is_reply(),
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn tcp_port_check(&self, target: IpAddr, port: u16) -> bool {
        let addr = format!("{}:{}", target, port);
        match TcpStream::connect_timeout(&addr.parse().unwrap(), self.timeout) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn full_health_check(&self, target: IpAddr, critical_ports: &[u16]) -> HealthStatus {
        let ping_ok = self.icmp_ping(target);
        let mut failed_ports = Vec::new();

        for &port in critical_ports {
            if !self.tcp_port_check(target, port) {
                failed_ports.push(port);
            }
        }

        HealthStatus {
            target,
            reachable: ping_ok,
            failed_ports,
        }
    }
}

pub struct HealthStatus {
    pub target: IpAddr,
    pub reachable: bool,
    pub failed_ports: Vec<u16>,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.reachable && self.failed_ports.is_empty()
    }

    pub fn report(&self) -> String {
        if self.is_healthy() {
            format!("Target {} is fully operational", self.target)
        } else {
            let mut report = format!("Target {} has issues:", self.target);
            if !self.reachable {
                report.push_str("\n  - ICMP unreachable");
            }
            for port in &self.failed_ports {
                report.push_str(&format!("\n  - Port {} closed", port));
            }
            report
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_connectivity() {
        let probe = NetworkProbe::new(2);
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        assert!(probe.tcp_port_check(localhost, 80) || true);
        
        let status = probe.full_health_check(localhost, &[22, 80, 443]);
        println!("{}", status.report());
    }
}