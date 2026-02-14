use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};

pub struct NetworkChecker {
    targets: Vec<String>,
    timeout: Duration,
}

impl NetworkChecker {
    pub fn new(targets: Vec<String>, timeout_secs: u64) -> Self {
        NetworkChecker {
            targets,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_all(&self) -> Vec<CheckResult> {
        self.targets
            .iter()
            .map(|target| self.check_target(target))
            .collect()
    }

    fn check_target(&self, target: &str) -> CheckResult {
        let start = Instant::now();
        let mut result = CheckResult {
            target: target.to_string(),
            reachable: false,
            latency_ms: 0,
            error: None,
        };

        match self.resolve_and_connect(target) {
            Ok(latency) => {
                result.reachable = true;
                result.latency_ms = latency.as_millis() as u64;
            }
            Err(e) => {
                result.error = Some(e.to_string());
            }
        }

        result
    }

    fn resolve_and_connect(&self, target: &str) -> io::Result<Duration> {
        let addr: SocketAddr = target.parse().map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid address: {}", e))
        })?;

        let start = Instant::now();
        let stream = TcpStream::connect_timeout(&addr, self.timeout)?;
        let duration = start.elapsed();
        drop(stream);
        Ok(duration)
    }
}

pub struct CheckResult {
    pub target: String,
    pub reachable: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

impl CheckResult {
    pub fn display(&self) -> String {
        if self.reachable {
            format!("{}: reachable ({} ms)", self.target, self.latency_ms)
        } else {
            format!("{}: unreachable - {}", self.target, self.error.as_deref().unwrap_or("unknown error"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_creation() {
        let targets = vec!["127.0.0.1:80".to_string(), "8.8.8.8:53".to_string()];
        let checker = NetworkChecker::new(targets, 5);
        assert_eq!(checker.timeout.as_secs(), 5);
    }
}