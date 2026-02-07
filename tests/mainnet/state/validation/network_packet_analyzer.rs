use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

impl From<u8> for Protocol {
    fn from(value: u8) -> Self {
        match value {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            id => Protocol::Unknown(id),
        }
    }
}

#[derive(Debug)]
pub struct PacketHeader {
    pub source_ip: Ipv4Addr,
    pub destination_ip: Ipv4Addr,
    pub protocol: Protocol,
    pub ttl: u8,
    pub payload_length: usize,
}

pub struct PacketAnalyzer {
    protocol_stats: HashMap<Protocol, u32>,
    total_packets: u64,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            protocol_stats: HashMap::new(),
            total_packets: 0,
        }
    }

    pub fn analyze_packet(&mut self, header: &PacketHeader) {
        self.total_packets += 1;
        *self.protocol_stats.entry(header.protocol.clone()).or_insert(0) += 1;
    }

    pub fn get_protocol_distribution(&self) -> HashMap<Protocol, f64> {
        let mut distribution = HashMap::new();
        
        for (protocol, count) in &self.protocol_stats {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            distribution.insert(protocol.clone(), percentage);
        }
        
        distribution
    }

    pub fn reset_stats(&mut self) {
        self.protocol_stats.clear();
        self.total_packets = 0;
    }

    pub fn total_packets_analyzed(&self) -> u64 {
        self.total_packets
    }
}

pub fn parse_ipv4_header(data: &[u8]) -> Option<PacketHeader> {
    if data.len() < 20 {
        return None;
    }

    let version_and_ihl = data[0];
    let version = version_and_ihl >> 4;
    
    if version != 4 {
        return None;
    }

    let ttl = data[8];
    let protocol = Protocol::from(data[9]);
    
    let source_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
    let destination_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
    
    let total_length = ((data[2] as u16) << 8) | data[3] as u16;
    let header_length = (version_and_ihl & 0x0F) as usize * 4;
    let payload_length = total_length as usize - header_length;

    Some(PacketHeader {
        source_ip,
        destination_ip,
        protocol,
        ttl,
        payload_length,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_conversion() {
        assert_eq!(Protocol::from(6), Protocol::TCP);
        assert_eq!(Protocol::from(17), Protocol::UDP);
        assert_eq!(Protocol::from(1), Protocol::ICMP);
        assert_eq!(Protocol::from(99), Protocol::Unknown(99));
    }

    #[test]
    fn test_packet_analyzer_stats() {
        let mut analyzer = PacketAnalyzer::new();
        
        let header = PacketHeader {
            source_ip: Ipv4Addr::new(192, 168, 1, 1),
            destination_ip: Ipv4Addr::new(192, 168, 1, 2),
            protocol: Protocol::TCP,
            ttl: 64,
            payload_length: 100,
        };

        analyzer.analyze_packet(&header);
        analyzer.analyze_packet(&header);
        
        let distribution = analyzer.get_protocol_distribution();
        assert_eq!(distribution.get(&Protocol::TCP), Some(&100.0));
        assert_eq!(analyzer.total_packets_analyzed(), 2);
    }

    #[test]
    fn test_parse_valid_ipv4_header() {
        let mut data = vec![0u8; 20];
        data[0] = 0x45;
        data[8] = 64;
        data[9] = 6;
        data[12] = 192;
        data[13] = 168;
        data[14] = 1;
        data[15] = 1;
        data[16] = 192;
        data[17] = 168;
        data[18] = 1;
        data[19] = 2;
        data[2] = 0x00;
        data[3] = 40;

        let header = parse_ipv4_header(&data).unwrap();
        assert_eq!(header.source_ip, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(header.destination_ip, Ipv4Addr::new(192, 168, 1, 2));
        assert_eq!(header.protocol, Protocol::TCP);
        assert_eq!(header.ttl, 64);
        assert_eq!(header.payload_length, 20);
    }
}use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    start_time: u128,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }

    fn update(&mut self, protocol: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    fn display(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let duration_secs = (current_time - self.start_time) as f64 / 1000.0;
        
        println!("Packet Capture Statistics:");
        println!("Duration: {:.2} seconds", duration_secs);
        println!("Total packets: {}", self.total_packets);
        println!("Packets per second: {:.2}", self.total_packets as f64 / duration_secs);
        println!("\nProtocol distribution:");
        
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

fn handle_transport_layer(packet: &[u8], protocol: u8, stats: &mut PacketStats) {
    match protocol {
        6 => {
            if let Some(tcp_packet) = TcpPacket::new(packet) {
                stats.update("TCP");
                println!(
                    "TCP Packet: {}:{} -> {}:{} [Flags: {:?}]",
                    tcp_packet.get_source(),
                    tcp_packet.get_destination(),
                    tcp_packet.get_sequence(),
                    tcp_packet.get_acknowledgement(),
                    tcp_packet.get_flags()
                );
            }
        }
        17 => {
            if let Some(udp_packet) = UdpPacket::new(packet) {
                stats.update("UDP");
                println!(
                    "UDP Packet: {} -> {} Length: {}",
                    udp_packet.get_source(),
                    udp_packet.get_destination(),
                    udp_packet.get_length()
                );
            }
        }
        _ => {
            stats.update("Other-Transport");
            println!("Other transport protocol: {}", protocol);
        }
    }
}

fn handle_ipv4_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
        stats.update("IPv4");
        println!(
            "IPv4 Packet: {} -> {} Protocol: {}",
            ipv4_packet.get_source(),
            ipv4_packet.get_destination(),
            ipv4_packet.get_next_level_protocol()
        );
        
        handle_transport_layer(
            ipv4_packet.payload(),
            ipv4_packet.get_next_level_protocol().0,
            stats,
        );
    }
}

fn capture_packets(interface_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".into()),
        Err(e) => return Err(format!("Failed to create channel: {}", e).into()),
    };

    let mut stats = PacketStats::new();
    let mut packet_count = 0;
    let max_packets = 100;

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Capturing up to {} packets...\n", max_packets);

    while packet_count < max_packets {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    packet_count += 1;
                    
                    match ethernet_packet.get_ethertype() {
                        pnet::packet::ethernet::EtherTypes::Ipv4 => {
                            handle_ipv4_packet(&ethernet_packet, &mut stats);
                        }
                        pnet::packet::ethernet::EtherTypes::Ipv6 => {
                            stats.update("IPv6");
                            println!("IPv6 Packet detected");
                        }
                        pnet::packet::ethernet::EtherTypes::Arp => {
                            stats.update("ARP");
                            println!("ARP Packet detected");
                        }
                        _ => {
                            stats.update("Other-Ethernet");
                            println!("Other Ethernet type: {:?}", ethernet_packet.get_ethertype());
                        }
                    }
                    
                    println!("---");
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    println!("\nCapture complete!");
    stats.display();
    
    Ok(())
}

fn main() {
    let interface_name = "eth0";
    
    if let Err(e) = capture_packets(interface_name) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}