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

pub fn find_interface_by_ip(ip_to_find: IpAddr) -> Option<NetworkInterface> {
    match get_network_interfaces() {
        Ok(interfaces) => {
            for iface in interfaces {
                match ip_to_find {
                    IpAddr::V4(ipv4) => {
                        if iface.ipv4_addresses.contains(&ipv4) {
                            return Some(iface);
                        }
                    }
                    IpAddr::V6(ipv6) => {
                        if iface.ipv6_addresses.contains(&ipv6) {
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

pub fn group_interfaces_by_subnet() -> HashMap<String, Vec<NetworkInterface>> {
    let mut grouped = HashMap::new();
    
    if let Ok(interfaces) = get_network_interfaces() {
        for iface in interfaces {
            for ipv4 in &iface.ipv4_addresses {
                let subnet = format!("{}.{}.{}.0/24", 
                    ipv4.octets()[0], 
                    ipv4.octets()[1], 
                    ipv4.octets()[2]);
                
                grouped.entry(subnet)
                    .or_insert_with(Vec::new)
                    .push(iface.clone());
            }
        }
    }
    
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_interfaces() {
        let interfaces = get_network_interfaces();
        assert!(interfaces.is_ok());
        
        if let Ok(ifaces) = interfaces {
            assert!(!ifaces.is_empty());
            
            for iface in ifaces {
                assert!(!iface.name.is_empty());
            }
        }
    }
    
    #[test]
    fn test_loopback_interface() {
        let interfaces = get_network_interfaces().unwrap();
        
        let has_loopback = interfaces.iter()
            .any(|iface| iface.name.contains("lo") || 
                iface.ipv4_addresses.contains(&Ipv4Addr::new(127, 0, 0, 1)));
        
        assert!(has_loopback);
    }
}