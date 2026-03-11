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
        let result = self.attempt_connection(target);
        let duration = start.elapsed();

        CheckResult {
            target: target.to_string(),
            success: result.is_ok(),
            latency: duration,
            error: result.err(),
        }
    }

    fn attempt_connection(&self, target: &str) -> io::Result<()> {
        let addr: SocketAddr = target.parse().map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid address: {}", e))
        })?;

        let stream = TcpStream::connect_timeout(&addr, self.timeout)?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct CheckResult {
    pub target: String,
    pub success: bool,
    pub latency: Duration,
    pub error: Option<io::Error>,
}

impl CheckResult {
    pub fn summary(&self) -> String {
        if self.success {
            format!(
                "{}: OK ({} ms)",
                self.target,
                self.latency.as_millis()
            )
        } else {
            format!(
                "{}: FAILED - {}",
                self.target,
                self.error.as_ref().map(|e| e.to_string()).unwrap_or_default()
            )
        }
    }
}

pub fn run_health_check() {
    let targets = vec![
        "8.8.8.8:53".to_string(),
        "1.1.1.1:53".to_string(),
        "localhost:22".to_string(),
    ];

    let checker = NetworkChecker::new(targets, 5);
    let results = checker.check_all();

    println!("Network Health Check Results:");
    println!("{}", "-".repeat(40));
    
    for result in results {
        println!("{}", result.summary());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_target() {
        let checker = NetworkChecker::new(vec!["127.0.0.1:80".to_string()], 1);
        let results = checker.check_all();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_invalid_target() {
        let checker = NetworkChecker::new(vec!["invalid_address".to_string()], 1);
        let results = checker.check_all();
        assert!(!results[0].success);
    }
}