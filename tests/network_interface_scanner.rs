use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use pnet_datalink;

pub fn list_interfaces() -> Vec<String> {
    let mut interfaces = Vec::new();
    
    for iface in pnet_datalink::interfaces() {
        let mut info = format!("Interface: {}", iface.name);
        
        if let Some(mac) = iface.mac {
            info.push_str(&format!(", MAC: {}", mac));
        }
        
        for ip in iface.ips {
            match ip.ip() {
                IpAddr::V4(ipv4) => {
                    info.push_str(&format!(", IPv4: {}", ipv4));
                }
                IpAddr::V6(ipv6) => {
                    info.push_str(&format!(", IPv6: {}", ipv6));
                }
            }
        }
        
        if iface.is_up() {
            info.push_str(", Status: UP");
        } else {
            info.push_str(", Status: DOWN");
        }
        
        if iface.is_loopback() {
            info.push_str(", Type: LOOPBACK");
        } else if iface.is_broadcast() {
            info.push_str(", Type: BROADCAST");
        }
        
        interfaces.push(info);
    }
    
    interfaces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_interfaces() {
        let interfaces = list_interfaces();
        assert!(!interfaces.is_empty());
        
        for iface_info in interfaces {
            println!("{}", iface_info);
        }
    }
}