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