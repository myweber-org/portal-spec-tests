use std::net::{TcpStream, IpAddr};
use std::time::Duration;
use std::thread;

pub struct NetworkChecker {
    timeout: Duration,
}

impl NetworkChecker {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkChecker {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn ping_host(&self, host: IpAddr) -> bool {
        let port = 80;
        match TcpStream::connect_timeout(&(host, port).into(), self.timeout) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn check_port(&self, host: IpAddr, port: u16) -> bool {
        match TcpStream::connect_timeout(&(host, port).into(), self.timeout) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn scan_ports(&self, host: IpAddr, start_port: u16, end_port: u16) -> Vec<u16> {
        let mut open_ports = Vec::new();
        let mut handles = vec![];

        for port in start_port..=end_port {
            let checker = self.clone();
            let host_clone = host;
            let handle = thread::spawn(move || {
                if checker.check_port(host_clone, port) {
                    Some(port)
                } else {
                    None
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Ok(Some(port)) = handle.join() {
                open_ports.push(port);
            }
        }

        open_ports.sort();
        open_ports
    }
}

impl Clone for NetworkChecker {
    fn clone(&self) -> Self {
        NetworkChecker {
            timeout: self.timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_ping_localhost() {
        let checker = NetworkChecker::new(2);
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        assert!(checker.ping_host(localhost));
    }

    #[test]
    fn test_check_invalid_port() {
        let checker = NetworkChecker::new(1);
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        assert!(!checker.check_port(localhost, 9999));
    }
}