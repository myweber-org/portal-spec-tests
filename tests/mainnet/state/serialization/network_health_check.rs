use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

struct HealthChecker {
    servers: Vec<SocketAddr>,
    timeout: Duration,
    results: Arc<Mutex<HashMap<SocketAddr, bool>>>,
}

impl HealthChecker {
    fn new(servers: Vec<SocketAddr>, timeout_secs: u64) -> Self {
        HealthChecker {
            servers,
            timeout: Duration::from_secs(timeout_secs),
            results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn check_server(&self, addr: SocketAddr) -> bool {
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn run_checks(&self) {
        let mut handles = vec![];
        let results = Arc::clone(&self.results);

        for &server in &self.servers {
            let results_clone = Arc::clone(&results);
            let handle = thread::spawn(move || {
                let checker = HealthChecker {
                    servers: vec![],
                    timeout: Duration::from_secs(2),
                    results: results_clone,
                };
                let is_up = checker.check_server(server);
                let mut results_map = checker.results.lock().unwrap();
                results_map.insert(server, is_up);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    fn print_results(&self) {
        let results = self.results.lock().unwrap();
        println!("Network Health Check Results:");
        println!("{:<20} {:<10}", "Server", "Status");
        println!("{}", "-".repeat(30));

        for (addr, &is_up) in results.iter() {
            let status = if is_up { "UP" } else { "DOWN" };
            println!("{:<20} {:<10}", addr, status);
        }
    }
}

fn main() {
    let servers = vec![
        "8.8.8.8:53".parse().unwrap(),
        "1.1.1.1:53".parse().unwrap(),
        "127.0.0.1:80".parse().unwrap(),
        "192.168.1.1:22".parse().unwrap(),
    ];

    let checker = HealthChecker::new(servers, 2);
    checker.run_checks();
    checker.print_results();
}