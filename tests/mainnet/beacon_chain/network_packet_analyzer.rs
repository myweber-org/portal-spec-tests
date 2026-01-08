
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interface_name = env::args().nth(1).unwrap_or_else(|| "eth0".to_string());
    
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .expect("Interface not found");

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unsupported channel type"),
        Err(e) => panic!("Failed to create channel: {}", e),
    };

    println!("Starting packet capture on interface: {}", interface_name);
    
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_ethernet_packet(&ethernet_packet);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn process_ethernet_packet(ethernet_packet: &EthernetPacket) {
    match ethernet_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                process_ipv4_packet(&ipv4_packet);
            }
        }
        EtherTypes::Ipv6 => {
            println!("IPv6 packet detected");
        }
        EtherTypes::Arp => {
            println!("ARP packet detected");
        }
        _ => {
            println!("Other Ethernet type: {:?}", ethernet_packet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4_packet: &Ipv4Packet) {
    let source = ipv4_packet.get_source();
    let destination = ipv4_packet.get_destination();
    let protocol = ipv4_packet.get_next_level_protocol();
    
    match protocol {
        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                let src_port = tcp_packet.get_source();
                let dst_port = tcp_packet.get_destination();
                let flags = tcp_packet.get_flags();
                
                println!(
                    "TCP Packet: {}:{} -> {}:{} | Flags: {:?} | Length: {}",
                    source,
                    src_port,
                    destination,
                    dst_port,
                    flags,
                    ipv4_packet.get_total_length()
                );
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
            println!(
                "UDP Packet: {} -> {} | Length: {}",
                source,
                destination,
                ipv4_packet.get_total_length()
            );
        }
        pnet::packet::ip::IpNextHeaderProtocols::Icmp => {
            println!(
                "ICMP Packet: {} -> {} | Length: {}",
                source,
                destination,
                ipv4_packet.get_total_length()
            );
        }
        _ => {
            println!(
                "Other IP protocol ({:?}): {} -> {} | Length: {}",
                protocol,
                source,
                destination,
                ipv4_packet.get_total_length()
            );
        }
    }
}
use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
    protocol: u8,
    payload: Vec<u8>,
    timestamp: u64,
}

#[derive(Debug)]
pub struct PacketAnalyzer {
    packet_count: usize,
    protocol_distribution: HashMap<u8, usize>,
    source_ip_counter: HashMap<Ipv4Addr, usize>,
    destination_ip_counter: HashMap<Ipv4Addr, usize>,
    total_bytes: usize,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_distribution: HashMap::new(),
            source_ip_counter: HashMap::new(),
            destination_ip_counter: HashMap::new(),
            total_bytes: 0,
        }
    }

    pub fn process_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;
        self.total_bytes += packet.payload.len();

        *self.protocol_distribution
            .entry(packet.protocol)
            .or_insert(0) += 1;

        *self.source_ip_counter
            .entry(packet.source_ip)
            .or_insert(0) += 1;

        *self.destination_ip_counter
            .entry(packet.destination_ip)
            .or_insert(0) += 1;
    }

    pub fn get_statistics(&self) -> PacketStatistics {
        let avg_packet_size = if self.packet_count > 0 {
            self.total_bytes as f64 / self.packet_count as f64
        } else {
            0.0
        };

        PacketStatistics {
            total_packets: self.packet_count,
            total_bytes: self.total_bytes,
            average_packet_size: avg_packet_size,
            unique_source_ips: self.source_ip_counter.len(),
            unique_destination_ips: self.destination_ip_counter.len(),
            protocol_distribution: self.protocol_distribution.clone(),
        }
    }

    pub fn get_top_source_ips(&self, limit: usize) -> Vec<(Ipv4Addr, usize)> {
        let mut items: Vec<_> = self.source_ip_counter.iter().collect();
        items.sort_by(|a, b| b.1.cmp(a.1));
        items
            .into_iter()
            .take(limit)
            .map(|(ip, count)| (*ip, *count))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PacketStatistics {
    pub total_packets: usize,
    pub total_bytes: usize,
    pub average_packet_size: f64,
    pub unique_source_ips: usize,
    pub unique_destination_ips: usize,
    pub protocol_distribution: HashMap<u8, usize>,
}

impl NetworkPacket {
    pub fn new(
        source_ip: Ipv4Addr,
        destination_ip: Ipv4Addr,
        protocol: u8,
        payload: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        NetworkPacket {
            source_ip,
            destination_ip,
            protocol,
            payload,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_analyzer_basic() {
        let mut analyzer = PacketAnalyzer::new();

        let packet1 = NetworkPacket::new(
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(192, 168, 1, 2),
            6,
            vec![1, 2, 3, 4, 5],
            1000,
        );

        let packet2 = NetworkPacket::new(
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(192, 168, 1, 3),
            17,
            vec![6, 7, 8],
            1001,
        );

        analyzer.process_packet(&packet1);
        analyzer.process_packet(&packet2);

        let stats = analyzer.get_statistics();
        assert_eq!(stats.total_packets, 2);
        assert_eq!(stats.total_bytes, 8);
        assert_eq!(stats.average_packet_size, 4.0);
        assert_eq!(stats.unique_source_ips, 1);
        assert_eq!(stats.unique_destination_ips, 2);
    }

    #[test]
    fn test_top_source_ips() {
        let mut analyzer = PacketAnalyzer::new();

        for i in 0..10 {
            let packet = NetworkPacket::new(
                Ipv4Addr::new(192, 168, 1, (i % 3) as u8),
                Ipv4Addr::new(10, 0, 0, 1),
                6,
                vec![],
                1000 + i,
            );
            analyzer.process_packet(&packet);
        }

        let top_ips = analyzer.get_top_source_ips(2);
        assert_eq!(top_ips.len(), 2);
    }
}