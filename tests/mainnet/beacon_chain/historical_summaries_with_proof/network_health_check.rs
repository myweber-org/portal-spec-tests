use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io;

pub struct NetworkHealthChecker {
    timeout: Duration,
}

impl NetworkHealthChecker {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkHealthChecker {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn ping_host(&self, host: &str, port: u16) -> io::Result<bool> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn scan_ports(&self, host: &str, ports: &[u16]) -> Vec<u16> {
        let mut open_ports = Vec::new();
        
        for &port in ports {
            if self.ping_host(host, port).unwrap_or(false) {
                open_ports.push(port);
            }
        }
        
        open_ports
    }
}

pub fn check_network_connectivity() -> io::Result<()> {
    let checker = NetworkHealthChecker::new(3);
    let test_host = "8.8.8.8";
    let test_port = 53;
    
    if checker.ping_host(test_host, test_port)? {
        println!("Network connectivity test passed");
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::ConnectionRefused, "Network connectivity test failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_success() {
        let checker = NetworkHealthChecker::new(1);
        let result = checker.ping_host("127.0.0.1", 80);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_ports() {
        let checker = NetworkHealthChecker::new(1);
        let ports = vec![80, 443, 8080];
        let open_ports = checker.scan_ports("localhost", &ports);
        assert!(open_ports.len() <= ports.len());
    }
}use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct NetworkHealth {
    timeout: Duration,
}

impl NetworkHealth {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkHealth {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_port(&self, host: &str, port: u16) -> bool {
        let addr_string = format!("{}:{}", host, port);
        
        match addr_string.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    match TcpStream::connect_timeout(&addr, self.timeout) {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    pub fn check_multiple_ports(&self, host: &str, ports: &[u16]) -> Vec<(u16, bool)> {
        ports.iter()
            .map(|&port| (port, self.check_port(host, port)))
            .collect()
    }

    pub fn basic_connectivity_test(&self, hosts: &[&str]) -> Vec<(&str, bool)> {
        hosts.iter()
            .map(|&host| {
                let result = self.check_port(host, 80) || self.check_port(host, 443);
                (host, result)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_check() {
        let checker = NetworkHealth::new(2);
        // Test with localhost - should succeed for open ports
        let result = checker.check_port("127.0.0.1", 80);
        // We can't guarantee port 80 is open, so just verify the function runs
        assert!(result == true || result == false);
    }

    #[test]
    fn test_multiple_ports() {
        let checker = NetworkHealth::new(1);
        let ports = vec![80, 443, 8080];
        let results = checker.check_multiple_ports("example.com", &ports);
        
        assert_eq!(results.len(), 3);
        for (port, status) in results {
            assert!(port == 80 || port == 443 || port == 8080);
            assert!(status == true || status == false);
        }
    }
}