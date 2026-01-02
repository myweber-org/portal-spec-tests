
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::io;

pub struct NetworkCheckResult {
    pub host: String,
    pub ping_ms: Option<u128>,
    pub ports_open: Vec<u16>,
    pub check_time: Instant,
}

pub fn check_host_connectivity(host: &str, timeout_secs: u64) -> io::Result<NetworkCheckResult> {
    let timeout = Duration::from_secs(timeout_secs);
    let start_time = Instant::now();
    
    let mut result = NetworkCheckResult {
        host: host.to_string(),
        ping_ms: None,
        ports_open: Vec::new(),
        check_time: Instant::now(),
    };

    let socket_addr = format!("{}:80", host)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Cannot resolve host"))?;

    let ping_start = Instant::now();
    match TcpStream::connect_timeout(&socket_addr, timeout) {
        Ok(_) => {
            result.ping_ms = Some(ping_start.elapsed().as_millis());
        }
        Err(e) => {
            return Err(e);
        }
    }

    let common_ports = [80, 443, 22, 21, 25, 53];
    for &port in &common_ports {
        let addr = format!("{}:{}", host, port);
        if let Ok(socket_addr) = addr.to_socket_addrs() {
            if let Some(addr) = socket_addr.next() {
                if TcpStream::connect_timeout(&addr, Duration::from_secs(1)).is_ok() {
                    result.ports_open.push(port);
                }
            }
        }
    }

    Ok(result)
}

pub fn format_check_result(result: &NetworkCheckResult) -> String {
    let mut output = format!("Network check for {}:\n", result.host);
    
    if let Some(ping) = result.ping_ms {
        output.push_str(&format!("  Ping: {} ms\n", ping));
    } else {
        output.push_str("  Ping: Failed\n");
    }
    
    if result.ports_open.is_empty() {
        output.push_str("  Open ports: None detected\n");
    } else {
        output.push_str(&format!("  Open ports: {:?}\n", result.ports_open));
    }
    
    output.push_str(&format!("  Checked at: {:?}\n", result.check_time));
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_connectivity() {
        let result = check_host_connectivity("localhost", 2);
        assert!(result.is_ok());
        
        if let Ok(res) = result {
            println!("{}", format_check_result(&res));
            assert!(!res.host.is_empty());
        }
    }
}