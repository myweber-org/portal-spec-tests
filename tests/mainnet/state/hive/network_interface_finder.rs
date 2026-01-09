use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ipv4_addresses: Vec<Ipv4Addr>,
    pub ipv6_addresses: Vec<Ipv6Addr>,
    pub mac_address: Option<String>,
}

pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();
    
    match pnet_datalink::interfaces() {
        Ok(net_interfaces) => {
            for iface in net_interfaces {
                let mut ipv4_addrs = Vec::new();
                let mut ipv6_addrs = Vec::new();
                
                for ip in iface.ips {
                    match ip.ip() {
                        IpAddr::V4(addr) => ipv4_addrs.push(addr),
                        IpAddr::V6(addr) => ipv6_addrs.push(addr),
                    }
                }
                
                let mac_address = iface.mac.map(|mac| format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    mac.0, mac.1, mac.2, mac.3, mac.4, mac.5));
                
                interfaces.push(NetworkInterface {
                    name: iface.name,
                    ipv4_addresses: ipv4_addrs,
                    ipv6_addresses: ipv6_addrs,
                    mac_address,
                });
            }
            Ok(interfaces)
        }
        Err(e) => Err(format!("Failed to get network interfaces: {}", e)),
    }
}

pub fn find_interface_by_ip(ip_addr: IpAddr) -> Option<NetworkInterface> {
    match get_network_interfaces() {
        Ok(interfaces) => {
            for iface in interfaces {
                match ip_addr {
                    IpAddr::V4(addr) => {
                        if iface.ipv4_addresses.contains(&addr) {
                            return Some(iface);
                        }
                    }
                    IpAddr::V6(addr) => {
                        if iface.ipv6_addresses.contains(&addr) {
                            return Some(iface);
                        }
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

pub fn get_interface_map() -> HashMap<String, NetworkInterface> {
    let mut map = HashMap::new();
    
    if let Ok(interfaces) = get_network_interfaces() {
        for iface in interfaces {
            map.insert(iface.name.clone(), iface);
        }
    }
    
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_interfaces() {
        let interfaces = get_network_interfaces();
        assert!(interfaces.is_ok());
        
        let interfaces = interfaces.unwrap();
        assert!(!interfaces.is_empty());
        
        for iface in interfaces {
            println!("Interface: {}", iface.name);
            println!("  IPv4: {:?}", iface.ipv4_addresses);
            println!("  IPv6: {:?}", iface.ipv6_addresses);
            println!("  MAC: {:?}", iface.mac_address);
        }
    }
    
    #[test]
    fn test_interface_map() {
        let map = get_interface_map();
        assert!(!map.is_empty());
    }
}