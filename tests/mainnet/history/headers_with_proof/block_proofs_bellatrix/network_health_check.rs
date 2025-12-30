use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::thread;
use std::collections::HashMap;

struct HealthCheck {
    targets: Vec<SocketAddr>,
    timeout: Duration,
}

impl HealthCheck {
    fn new(targets: Vec<SocketAddr>, timeout_secs: u64) -> Self {
        HealthCheck {
            targets,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    fn run_check(&self) -> HashMap<SocketAddr, bool> {
        let mut results = HashMap::new();

        for &target in &self.targets {
            let status = self.check_single(target);
            results.insert(target, status);
        }

        results
    }

    fn check_single(&self, addr: SocketAddr) -> bool {
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn continuous_monitor(&self, interval_secs: u64) {
        loop {
            let results = self.run_check();
            self.log_results(&results);
            thread::sleep(Duration::from_secs(interval_secs));
        }
    }

    fn log_results(&self, results: &HashMap<SocketAddr, bool>) {
        println!("--- Health Check Results ---");
        for (addr, status) in results {
            let status_str = if *status { "UP" } else { "DOWN" };
            println!("{}: {}", addr, status_str);
        }
        println!("----------------------------");
    }
}

fn main() {
    let targets = vec![
        "8.8.8.8:53".parse().unwrap(),
        "1.1.1.1:53".parse().unwrap(),
        "127.0.0.1:80".parse().unwrap(),
    ];

    let checker = HealthCheck::new(targets, 3);
    
    println!("Running single health check...");
    let results = checker.run_check();
    checker.log_results(&results);

    println!("\nStarting continuous monitoring (press Ctrl+C to stop)...");
    checker.continuous_monitor(5);
}