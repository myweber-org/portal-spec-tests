use std::process::Command;
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr};
use std::str;

struct PingResult {
    host: String,
    packets_sent: u32,
    packets_received: u32,
    min_latency: Duration,
    max_latency: Duration,
    avg_latency: Duration,
}

fn ping_host(host: &str, count: u32) -> Result<PingResult, String> {
    let output = Command::new("ping")
        .arg("-c")
        .arg(count.to_string())
        .arg(host)
        .output()
        .map_err(|e| format!("Failed to execute ping: {}", e))?;

    if !output.status.success() {
        return Err(format!("Ping command failed for host: {}", host));
    }

    let output_str = str::from_utf8(&output.stdout)
        .map_err(|e| format!("Invalid UTF-8 sequence: {}", e))?;

    parse_ping_output(host, output_str)
}

fn parse_ping_output(host: &str, output: &str) -> Result<PingResult, String> {
    let mut packets_sent = 0;
    let mut packets_received = 0;
    let mut latencies = Vec::new();

    for line in output.lines() {
        if line.contains("packets transmitted") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                packets_sent = parts[0]
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                packets_received = parts[1]
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        } else if line.contains("time=") {
            if let Some(time_start) = line.find("time=") {
                let time_str = &line[time_start + 5..];
                if let Some(time_end) = time_str.find(" ms") {
                    if let Ok(latency_ms) = time_str[..time_end].parse::<f64>() {
                        latencies.push(Duration::from_micros((latency_ms * 1000.0) as u64));
                    }
                }
            }
        }
    }

    if latencies.is_empty() {
        return Err("No latency data found in ping output".to_string());
    }

    let min_latency = *latencies.iter().min().unwrap();
    let max_latency = *latencies.iter().max().unwrap();
    let total_latency: Duration = latencies.iter().sum();
    let avg_latency = total_latency / latencies.len() as u32;

    Ok(PingResult {
        host: host.to_string(),
        packets_sent,
        packets_received,
        min_latency,
        max_latency,
        avg_latency,
    })
}

fn check_network_connectivity(hosts: &[&str]) -> Vec<Result<PingResult, String>> {
    let mut results = Vec::new();
    
    for host in hosts {
        println!("Checking connectivity to {}...", host);
        let start = Instant::now();
        let result = ping_host(host, 4);
        let duration = start.elapsed();
        
        match &result {
            Ok(ping_result) => {
                println!("  Status: Reachable");
                println!("  Packets: {}/{} received", 
                         ping_result.packets_received, ping_result.packets_sent);
                println!("  Latency: min={:.2}ms, avg={:.2}ms, max={:.2}ms",
                         ping_result.min_latency.as_micros() as f64 / 1000.0,
                         ping_result.avg_latency.as_micros() as f64 / 1000.0,
                         ping_result.max_latency.as_micros() as f64 / 1000.0);
            }
            Err(e) => {
                println!("  Status: Unreachable - {}", e);
            }
        }
        println!("  Check duration: {:?}", duration);
        results.push(result);
    }
    
    results
}

fn main() {
    let test_hosts = [
        "8.8.8.8",
        "1.1.1.1",
        "google.com",
        "localhost",
    ];
    
    println!("Network Health Check Utility");
    println!("============================\n");
    
    let results = check_network_connectivity(&test_hosts);
    
    let successful_checks = results.iter()
        .filter(|r| r.is_ok())
        .count();
    
    println!("\nSummary: {}/{} hosts reachable", 
             successful_checks, test_hosts.len());
    
    if successful_checks == test_hosts.len() {
        println!("Network connectivity: EXCELLENT");
    } else if successful_checks >= test_hosts.len() / 2 {
        println!("Network connectivity: DEGRADED");
    } else {
        println!("Network connectivity: POOR");
    }
}
use std::process::Command;
use std::time::Duration;
use reqwest::blocking::Client;
use std::thread;

fn ping_host(host: &str) -> bool {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "ping", "-n", "1", host])
            .output()
    } else {
        Command::new("ping")
            .args(["-c", "1", host])
            .output()
    };

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn check_http_endpoint(url: &str) -> bool {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build();

    match client {
        Ok(client) => match client.get(url).send() {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

fn monitor_service(host: &str, url: &str, interval_secs: u64) {
    loop {
        println!("Checking network health at {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        
        let ping_result = ping_host(host);
        let http_result = check_http_endpoint(url);
        
        println!("Ping to {}: {}", host, if ping_result { "OK" } else { "FAILED" });
        println!("HTTP request to {}: {}", url, if http_result { "OK" } else { "FAILED" });
        
        if ping_result && http_result {
            println!("All checks passed");
        } else {
            println!("Some checks failed");
        }
        
        println!("---");
        thread::sleep(Duration::from_secs(interval_secs));
    }
}

fn main() {
    let host = "8.8.8.8";
    let url = "https://httpbin.org/status/200";
    let interval = 10;
    
    println!("Starting network health monitor");
    println!("Target host: {}", host);
    println!("Target URL: {}", url);
    println!("Check interval: {} seconds", interval);
    println!();
    
    monitor_service(host, url, interval);
}
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use std::thread;
use rand::Rng;

const PACKET_COUNT: usize = 10;
const TIMEOUT_MS: u64 = 1000;

struct NetworkMetrics {
    latency_ms: f64,
    packet_loss_percent: f64,
    jitter_ms: f64,
}

impl NetworkMetrics {
    fn new() -> Self {
        NetworkMetrics {
            latency_ms: 0.0,
            packet_loss_percent: 0.0,
            jitter_ms: 0.0,
        }
    }

    fn display(&self) {
        println!("Network Health Report:");
        println!("  Latency: {:.2} ms", self.latency_ms);
        println!("  Packet Loss: {:.1}%", self.packet_loss_percent);
        println!("  Jitter: {:.2} ms", self.jitter_ms);
        
        if self.latency_ms < 50.0 && self.packet_loss_percent < 1.0 {
            println!("Status: EXCELLENT");
        } else if self.latency_ms < 100.0 && self.packet_loss_percent < 5.0 {
            println!("Status: GOOD");
        } else if self.latency_ms < 200.0 && self.packet_loss_percent < 10.0 {
            println!("Status: FAIR");
        } else {
            println!("Status: POOR");
        }
    }
}

fn simulate_ping(destination: IpAddr) -> Option<Duration> {
    let mut rng = rand::thread_rng();
    
    if rng.gen_bool(0.95) {
        let latency = rng.gen_range(5..200) as u64;
        thread::sleep(Duration::from_millis(latency));
        Some(Duration::from_millis(latency))
    } else {
        thread::sleep(Duration::from_millis(TIMEOUT_MS));
        None
    }
}

fn calculate_jitter(latencies: &[Duration]) -> f64 {
    if latencies.len() < 2 {
        return 0.0;
    }
    
    let mut diffs = Vec::new();
    for i in 1..latencies.len() {
        let diff = (latencies[i].as_millis() as f64 - latencies[i-1].as_millis() as f64).abs();
        diffs.push(diff);
    }
    
    diffs.iter().sum::<f64>() / diffs.len() as f64
}

fn perform_health_check(target: IpAddr) -> NetworkMetrics {
    let mut metrics = NetworkMetrics::new();
    let mut successful_pings = 0;
    let mut total_latency = Duration::ZERO;
    let mut latencies = Vec::new();
    
    println!("Testing connection to {}...", target);
    
    for i in 0..PACKET_COUNT {
        print!("Packet {}: ", i + 1);
        
        let start = Instant::now();
        if let Some(latency) = simulate_ping(target) {
            successful_pings += 1;
            total_latency += latency;
            latencies.push(latency);
            println!("Reply received in {} ms", latency.as_millis());
        } else {
            println!("Request timed out");
        }
        
        if i < PACKET_COUNT - 1 {
            thread::sleep(Duration::from_millis(500));
        }
    }
    
    if successful_pings > 0 {
        metrics.latency_ms = total_latency.as_millis() as f64 / successful_pings as f64;
        metrics.jitter_ms = calculate_jitter(&latencies);
    }
    
    metrics.packet_loss_percent = 
        ((PACKET_COUNT - successful_pings) as f64 / PACKET_COUNT as f64) * 100.0;
    
    metrics
}

fn main() {
    let target = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
    let metrics = perform_health_check(target);
    println!();
    metrics.display();
}use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

struct Host {
    address: IpAddr,
    port: u16,
}

impl Host {
    fn new(address: IpAddr, port: u16) -> Self {
        Host { address, port }
    }

    fn check(&self, timeout_secs: u64) -> bool {
        let socket_addr = SocketAddr::new(self.address, self.port);
        match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(timeout_secs)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

struct NetworkHealthChecker {
    hosts: Vec<Host>,
    timeout_secs: u64,
}

impl NetworkHealthChecker {
    fn new(timeout_secs: u64) -> Self {
        NetworkHealthChecker {
            hosts: Vec::new(),
            timeout_secs,
        }
    }

    fn add_host(&mut self, address: IpAddr, port: u16) {
        self.hosts.push(Host::new(address, port));
    }

    fn run_check(&self) -> Vec<(IpAddr, u16, bool)> {
        self.hosts
            .iter()
            .map(|host| {
                let status = host.check(self.timeout_secs);
                (host.address, host.port, status)
            })
            .collect()
    }
}

fn main() {
    let mut checker = NetworkHealthChecker::new(5);
    checker.add_host(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
    checker.add_host(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 80);
    checker.add_host(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 443);

    let results = checker.run_check();
    for (addr, port, status) in results {
        println!(
            "Host {}:{} is {}",
            addr,
            port,
            if status { "reachable" } else { "unreachable" }
        );
    }
}