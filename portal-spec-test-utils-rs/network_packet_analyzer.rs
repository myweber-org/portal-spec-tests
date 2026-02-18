use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct PacketStats {
    pub total_packets: u64,
    pub protocol_counts: HashMap<String, u64>,
    pub start_time: Instant,
}

impl PacketStats {
    pub fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    pub fn update(&mut self, protocol: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    pub fn display_summary(&self) {
        let duration = self.start_time.elapsed();
        println!("Packet Capture Summary:");
        println!("Duration: {:.2} seconds", duration.as_secs_f64());
        println!("Total packets captured: {}", self.total_packets);
        
        if self.total_packets > 0 {
            let packets_per_second = self.total_packets as f64 / duration.as_secs_f64();
            println!("Packets per second: {:.2}", packets_per_second);
            
            println!("\nProtocol Distribution:");
            for (protocol, count) in &self.protocol_counts {
                let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
                println!("  {}: {} ({:.1}%)", protocol, count, percentage);
            }
        }
    }
}

pub fn capture_packets(interface_name: &str, duration_secs: u64) -> Result<PacketStats, String> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".to_string()),
        Err(e) => return Err(format!("Failed to create channel: {}", e)),
    };

    let mut stats = PacketStats::new();
    let timeout = Duration::from_secs(duration_secs);
    let start_time = Instant::now();

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Capture will run for {} seconds", duration_secs);

    while start_time.elapsed() < timeout {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_ethernet_packet(&ethernet_packet, &mut stats);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
            }
        }
    }

    Ok(stats)
}

fn process_ethernet_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                process_ipv4_packet(&ipv4_packet, stats);
            }
        }
        EtherTypes::Ipv6 => {
            stats.update("IPv6");
        }
        EtherTypes::Arp => {
            stats.update("ARP");
        }
        _ => {
            stats.update("Other");
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet, stats: &mut PacketStats) {
    match ipv4.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                let src_port = tcp_packet.get_source();
                let dst_port = tcp_packet.get_destination();
                stats.update("TCP");
                
                if src_port == 80 || dst_port == 80 {
                    stats.update("HTTP");
                } else if src_port == 443 || dst_port == 443 {
                    stats.update("HTTPS");
                } else if src_port == 22 || dst_port == 22 {
                    stats.update("SSH");
                }
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                let src_port = udp_packet.get_source();
                let dst_port = udp_packet.get_destination();
                stats.update("UDP");
                
                if src_port == 53 || dst_port == 53 {
                    stats.update("DNS");
                } else if src_port == 67 || dst_port == 67 || src_port == 68 || dst_port == 68 {
                    stats.update("DHCP");
                }
            }
        }
        IpNextHeaderProtocols::Icmp => {
            stats.update("ICMP");
        }
        _ => {
            stats.update("Other-IP");
        }
    }
}

pub fn list_interfaces() -> Vec<String> {
    datalink::interfaces()
        .iter()
        .map(|iface| iface.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_stats_new() {
        let stats = PacketStats::new();
        assert_eq!(stats.total_packets, 0);
        assert!(stats.protocol_counts.is_empty());
    }

    #[test]
    fn test_packet_stats_update() {
        let mut stats = PacketStats::new();
        stats.update("TCP");
        stats.update("TCP");
        stats.update("UDP");
        
        assert_eq!(stats.total_packets, 3);
        assert_eq!(stats.protocol_counts.get("TCP"), Some(&2));
        assert_eq!(stats.protocol_counts.get("UDP"), Some(&1));
    }

    #[test]
    fn test_list_interfaces() {
        let interfaces = list_interfaces();
        assert!(!interfaces.is_empty());
    }
}use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug)]
pub struct PacketHeader {
    pub source_ip: Ipv4Addr,
    pub destination_ip: Ipv4Addr,
    pub protocol: Protocol,
    pub payload_size: usize,
}

pub struct PacketAnalyzer {
    protocol_stats: HashMap<Protocol, u32>,
    total_packets: u32,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            protocol_stats: HashMap::new(),
            total_packets: 0,
        }
    }

    pub fn analyze_packet(&mut self, header: PacketHeader) {
        self.total_packets += 1;
        *self.protocol_stats.entry(header.protocol.clone()).or_insert(0) += 1;

        println!("Packet analyzed: {:?}", header);
    }

    pub fn get_protocol_distribution(&self) -> HashMap<Protocol, f32> {
        let mut distribution = HashMap::new();
        
        for (protocol, count) in &self.protocol_stats {
            let percentage = (*count as f32 / self.total_packets as f32) * 100.0;
            distribution.insert(protocol.clone(), percentage);
        }
        
        distribution
    }

    pub fn print_statistics(&self) {
        println!("Total packets analyzed: {}", self.total_packets);
        println!("Protocol distribution:");
        
        for (protocol, percentage) in self.get_protocol_distribution() {
            println!("  {:?}: {:.2}%", protocol, percentage);
        }
    }
}

impl Protocol {
    pub fn from_u8(value: u8) -> Self {
        match value {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            _ => Protocol::Unknown(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_analysis() {
        let mut analyzer = PacketAnalyzer::new();
        
        let packet1 = PacketHeader {
            source_ip: Ipv4Addr::new(192, 168, 1, 1),
            destination_ip: Ipv4Addr::new(192, 168, 1, 2),
            protocol: Protocol::TCP,
            payload_size: 512,
        };
        
        let packet2 = PacketHeader {
            source_ip: Ipv4Addr::new(10, 0, 0, 1),
            destination_ip: Ipv4Addr::new(10, 0, 0, 2),
            protocol: Protocol::UDP,
            payload_size: 256,
        };
        
        analyzer.analyze_packet(packet1);
        analyzer.analyze_packet(packet2);
        
        assert_eq!(analyzer.total_packets, 2);
        assert_eq!(analyzer.protocol_stats.get(&Protocol::TCP), Some(&1));
        assert_eq!(analyzer.protocol_stats.get(&Protocol::UDP), Some(&1));
    }

    #[test]
    fn test_protocol_conversion() {
        assert_eq!(Protocol::from_u8(6), Protocol::TCP);
        assert_eq!(Protocol::from_u8(17), Protocol::UDP);
        assert_eq!(Protocol::from_u8(1), Protocol::ICMP);
        assert_eq!(Protocol::from_u8(99), Protocol::Unknown(99));
    }
}