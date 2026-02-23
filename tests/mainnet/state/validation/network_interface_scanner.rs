use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use pnet_datalink::{interfaces, NetworkInterface};

pub fn scan_interfaces() -> Vec<InterfaceInfo> {
    let all_interfaces = interfaces();
    
    all_interfaces
        .iter()
        .map(|iface| InterfaceInfo::from_interface(iface))
        .collect()
}

#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub name: String,
    pub description: String,
    pub mac_address: Option<String>,
    pub ipv4_addresses: Vec<Ipv4Addr>,
    pub ipv6_addresses: Vec<Ipv6Addr>,
    pub is_up: bool,
    pub is_loopback: bool,
    pub is_multicast: bool,
    pub mtu: u32,
}

impl InterfaceInfo {
    pub fn from_interface(iface: &NetworkInterface) -> Self {
        let ipv4_addrs: Vec<Ipv4Addr> = iface
            .ips
            .iter()
            .filter_map(|ip_network| match ip_network.ip() {
                IpAddr::V4(addr) => Some(addr),
                _ => None,
            })
            .collect();

        let ipv6_addrs: Vec<Ipv6Addr> = iface
            .ips
            .iter()
            .filter_map(|ip_network| match ip_network.ip() {
                IpAddr::V6(addr) => Some(addr),
                _ => None,
            })
            .collect();

        let mac_address = iface.mac.map(|mac| format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac.0, mac.1, mac.2, mac.3, mac.4, mac.5));

        InterfaceInfo {
            name: iface.name.clone(),
            description: iface.description.clone(),
            mac_address,
            ipv4_addresses: ipv4_addrs,
            ipv6_addresses: ipv6_addrs,
            is_up: iface.is_up(),
            is_loopback: iface.is_loopback(),
            is_multicast: iface.is_multicast(),
            mtu: iface.mtu.unwrap_or(0),
        }
    }

    pub fn has_ip_address(&self) -> bool {
        !self.ipv4_addresses.is_empty() || !self.ipv6_addresses.is_empty()
    }

    pub fn primary_ipv4(&self) -> Option<Ipv4Addr> {
        self.ipv4_addresses.first().copied()
    }

    pub fn to_json(&self) -> String {
        let mac_str = self.mac_address
            .as_ref()
            .map(|s| format!("\"{}\"", s))
            .unwrap_or_else(|| "null".to_string());

        let ipv4_strs: Vec<String> = self.ipv4_addresses
            .iter()
            .map(|ip| format!("\"{}\"", ip))
            .collect();

        let ipv6_strs: Vec<String> = self.ipv6_addresses
            .iter()
            .map(|ip| format!("\"{}\"", ip))
            .collect();

        format!(
            r#"{{
                "name": "{}",
                "description": "{}",
                "mac_address": {},
                "ipv4_addresses": [{}],
                "ipv6_addresses": [{}],
                "is_up": {},
                "is_loopback": {},
                "is_multicast": {},
                "mtu": {},
                "has_ip_address": {}
            }}"#,
            self.name,
            self.description,
            mac_str,
            ipv4_strs.join(", "),
            ipv6_strs.join(", "),
            self.is_up,
            self.is_loopback,
            self.is_multicast,
            self.mtu,
            self.has_ip_address()
        )
    }
}

pub fn display_interfaces_summary() {
    let interfaces = scan_interfaces();
    
    println!("Network Interfaces Summary:");
    println!("===========================");
    
    for (i, iface) in interfaces.iter().enumerate() {
        println!("{}. {}", i + 1, iface.name);
        println!("   Description: {}", iface.description);
        println!("   Status: {}", if iface.is_up { "UP" } else { "DOWN" });
        println!("   Type: {}", if iface.is_loopback { "Loopback" } else { "Physical" });
        
        if let Some(mac) = &iface.mac_address {
            println!("   MAC: {}", mac);
        }
        
        if !iface.ipv4_addresses.is_empty() {
            println!("   IPv4: {:?}", iface.ipv4_addresses);
        }
        
        if !iface.ipv6_addresses.is_empty() {
            println!("   IPv6: {:?}", iface.ipv6_addresses);
        }
        
        if iface.mtu > 0 {
            println!("   MTU: {}", iface.mtu);
        }
        
        println!();
    }
    
    let active_interfaces: Vec<&InterfaceInfo> = interfaces
        .iter()
        .filter(|iface| iface.is_up && iface.has_ip_address())
        .collect();
    
    println!("Active interfaces with IP addresses: {}", active_interfaces.len());
    for iface in active_interfaces {
        println!("  - {} ({})", iface.name, iface.description);
    }
}