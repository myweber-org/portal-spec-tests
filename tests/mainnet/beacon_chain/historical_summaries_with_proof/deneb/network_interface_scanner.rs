use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::Command;
use std::str;

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ipv4_addresses: Vec<Ipv4Addr>,
    pub ipv6_addresses: Vec<Ipv6Addr>,
}

pub fn scan_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();
    
    match Command::new("ip").arg("addr").output() {
        Ok(output) => {
            let output_str = str::from_utf8(&output.stdout).map_err(|e| e.to_string())?;
            parse_ip_output(&output_str, &mut interfaces);
        }
        Err(_) => {
            match Command::new("ifconfig").output() {
                Ok(output) => {
                    let output_str = str::from_utf8(&output.stdout).map_err(|e| e.to_string())?;
                    parse_ifconfig_output(&output_str, &mut interfaces);
                }
                Err(e) => return Err(format!("Failed to execute network command: {}", e)),
            }
        }
    }
    
    Ok(interfaces)
}

fn parse_ip_output(output: &str, interfaces: &mut Vec<NetworkInterface>) {
    let mut current_interface: Option<NetworkInterface> = None;
    
    for line in output.lines() {
        if line.starts_with(|c: char| c.is_numeric()) {
            if let Some(iface) = current_interface.take() {
                interfaces.push(iface);
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[1].trim_end_matches(':').to_string();
                current_interface = Some(NetworkInterface {
                    name,
                    ipv4_addresses: Vec::new(),
                    ipv6_addresses: Vec::new(),
                });
            }
        } else if line.trim().starts_with("inet") {
            if let Some(ref mut iface) = current_interface {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(ip) = parts[1].parse::<Ipv4Addr>() {
                        iface.ipv4_addresses.push(ip);
                    }
                }
            }
        } else if line.trim().starts_with("inet6") {
            if let Some(ref mut iface) = current_interface {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(ip) = parts[1].parse::<Ipv6Addr>() {
                        iface.ipv6_addresses.push(ip);
                    }
                }
            }
        }
    }
    
    if let Some(iface) = current_interface.take() {
        interfaces.push(iface);
    }
}

fn parse_ifconfig_output(output: &str, interfaces: &mut Vec<NetworkInterface>) {
    let mut current_name = String::new();
    let mut ipv4_addrs = Vec::new();
    let mut ipv6_addrs = Vec::new();
    
    for line in output.lines() {
        if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
            if !current_name.is_empty() {
                interfaces.push(NetworkInterface {
                    name: current_name.clone(),
                    ipv4_addresses: ipv4_addrs.clone(),
                    ipv6_addresses: ipv6_addrs.clone(),
                });
                ipv4_addrs.clear();
                ipv6_addrs.clear();
            }
            
            let parts: Vec<&str> = line.split(':').collect();
            if !parts.is_empty() {
                current_name = parts[0].to_string();
            }
        } else if line.contains("inet ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for i in 0..parts.len() {
                if parts[i] == "inet" && i + 1 < parts.len() {
                    if let Ok(ip) = parts[i + 1].parse::<Ipv4Addr>() {
                        ipv4_addrs.push(ip);
                    }
                }
            }
        } else if line.contains("inet6 ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for i in 0..parts.len() {
                if parts[i] == "inet6" && i + 1 < parts.len() {
                    if let Ok(ip) = parts[i + 1].parse::<Ipv6Addr>() {
                        ipv6_addrs.push(ip);
                    }
                }
            }
        }
    }
    
    if !current_name.is_empty() {
        interfaces.push(NetworkInterface {
            name: current_name,
            ipv4_addresses: ipv4_addrs,
            ipv6_addresses: ipv6_addrs,
        });
    }
}

pub fn display_interfaces(interfaces: &[NetworkInterface]) {
    for iface in interfaces {
        println!("Interface: {}", iface.name);
        
        if !iface.ipv4_addresses.is_empty() {
            println!("  IPv4 Addresses:");
            for addr in &iface.ipv4_addresses {
                println!("    - {}", addr);
            }
        }
        
        if !iface.ipv6_addresses.is_empty() {
            println!("  IPv6 Addresses:");
            for addr in &iface.ipv6_addresses {
                println!("    - {}", addr);
            }
        }
        
        if iface.ipv4_addresses.is_empty() && iface.ipv6_addresses.is_empty() {
            println!("  No IP addresses assigned");
        }
        
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interface_creation() {
        let iface = NetworkInterface {
            name: "eth0".to_string(),
            ipv4_addresses: vec![Ipv4Addr::new(192, 168, 1, 100)],
            ipv6_addresses: vec![Ipv6Addr::new(0xfe80, 0, 0, 0, 0xdead, 0xbeef, 0xcafe, 0xbabe)],
        };
        
        assert_eq!(iface.name, "eth0");
        assert_eq!(iface.ipv4_addresses.len(), 1);
        assert_eq!(iface.ipv6_addresses.len(), 1);
    }
}