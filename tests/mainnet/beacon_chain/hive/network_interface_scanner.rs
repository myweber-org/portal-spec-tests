use pnet::datalink;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let interfaces = datalink::interfaces();
    
    println!("Available network interfaces:");
    println!("{:<20} {:<15} {:<20} {:<10}", "Name", "MAC", "IPv4", "IPv6");
    println!("{}", "-".repeat(70));
    
    for interface in interfaces {
        let name = interface.name;
        let mac = match interface.mac {
            Some(addr) => addr.to_string(),
            None => "N/A".to_string(),
        };
        
        let ipv4_addrs: Vec<String> = interface.ips
            .iter()
            .filter(|ip| ip.is_ipv4())
            .map(|ip| ip.to_string())
            .collect();
        
        let ipv6_addrs: Vec<String> = interface.ips
            .iter()
            .filter(|ip| ip.is_ipv6())
            .map(|ip| ip.to_string())
            .collect();
        
        let ipv4 = if ipv4_addrs.is_empty() {
            "N/A".to_string()
        } else {
            ipv4_addrs.join(", ")
        };
        
        let ipv6 = if ipv6_addrs.is_empty() {
            "N/A".to_string()
        } else {
            ipv6_addrs.join(", ")
        };
        
        println!("{:<20} {:<15} {:<20} {:<10}", name, mac, ipv4, ipv6);
    }
    
    Ok(())
}