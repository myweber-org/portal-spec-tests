use pnet::datalink;
use std::net::IpAddr;

pub fn list_network_interfaces() -> Vec<(String, Vec<IpAddr>)> {
    let interfaces = datalink::interfaces();
    let mut result = Vec::new();

    for interface in interfaces {
        let ips: Vec<IpAddr> = interface.ips.iter()
            .map(|ip_network| ip_network.ip())
            .collect();
        
        if !ips.is_empty() {
            result.push((interface.name.clone(), ips));
        }
    }

    result
}

pub fn display_interfaces() {
    let interfaces = list_network_interfaces();
    
    println!("Available network interfaces:");
    for (name, ips) in interfaces {
        println!("Interface: {}", name);
        for ip in ips {
            println!("  IP: {}", ip);
        }
        println!();
    }
}