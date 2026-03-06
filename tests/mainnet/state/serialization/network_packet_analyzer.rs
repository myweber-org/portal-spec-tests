
use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, PartialEq)]
enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug)]
struct PacketHeader {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
    protocol: Protocol,
    payload_length: usize,
}

struct PacketAnalyzer {
    protocol_counts: HashMap<Protocol, u32>,
    total_packets: u32,
}

impl PacketAnalyzer {
    fn new() -> Self {
        PacketAnalyzer {
            protocol_counts: HashMap::new(),
            total_packets: 0,
        }
    }

    fn process_packet(&mut self, header: &PacketHeader) {
        self.total_packets += 1;
        *self.protocol_counts.entry(header.protocol.clone()).or_insert(0) += 1;
    }

    fn print_statistics(&self) {
        println!("Packet Analysis Report");
        println!("======================");
        println!("Total packets processed: {}", self.total_packets);
        
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f32 / self.total_packets as f32) * 100.0;
            println!("{:?}: {} packets ({:.1}%)", protocol, count, percentage);
        }
    }
}

fn parse_protocol(value: u8) -> Protocol {
    match value {
        6 => Protocol::TCP,
        17 => Protocol::UDP,
        1 => Protocol::ICMP,
        _ => Protocol::Unknown(value),
    }
}

fn create_sample_packets() -> Vec<PacketHeader> {
    vec![
        PacketHeader {
            source_ip: Ipv4Addr::new(192, 168, 1, 100),
            destination_ip: Ipv4Addr::new(10, 0, 0, 1),
            protocol: Protocol::TCP,
            payload_length: 1500,
        },
        PacketHeader {
            source_ip: Ipv4Addr::new(10, 0, 0, 2),
            destination_ip: Ipv4Addr::new(192, 168, 1, 101),
            protocol: Protocol::UDP,
            payload_length: 512,
        },
        PacketHeader {
            source_ip: Ipv4Addr::new(172, 16, 0, 1),
            destination_ip: Ipv4Addr::new(8, 8, 8, 8),
            protocol: Protocol::ICMP,
            payload_length: 64,
        },
        PacketHeader {
            source_ip: Ipv4Addr::new(192, 168, 1, 100),
            destination_ip: Ipv4Addr::new(10, 0, 0, 1),
            protocol: Protocol::TCP,
            payload_length: 1200,
        },
    ]
}

fn main() {
    let mut analyzer = PacketAnalyzer::new();
    let packets = create_sample_packets();
    
    for packet in &packets {
        analyzer.process_packet(packet);
    }
    
    analyzer.print_statistics();
    
    let test_protocol = parse_protocol(6);
    assert_eq!(test_protocol, Protocol::TCP);
    
    let unknown_protocol = parse_protocol(99);
    match unknown_protocol {
        Protocol::Unknown(val) => println!("Detected unknown protocol with value: {}", val),
        _ => unreachable!(),
    }
}rust
use pnet::datalink::{self, Channel, Config};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct PacketStats {
    total_packets: usize,
    protocol_counts: HashMap<String, usize>,
    start_time: u64,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn increment_protocol(&mut self, protocol: &str) {
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
        self.total_packets += 1;
    }

    fn display_stats(&self) {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.start_time;
        
        println!("Packet Capture Statistics:");
        println!("Duration: {} seconds", duration);
        println!("Total packets: {}", self.total_packets);
        println!("Packets per second: {:.2}", self.total_packets as f64 / duration as f64);
        println!("\nProtocol Distribution:");
        
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

fn handle_ethernet_frame(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                handle_ipv4_packet(&ipv4_packet, stats);
            }
        }
        EtherTypes::Arp => {
            stats.increment_protocol("ARP");
        }
        EtherTypes::Ipv6 => {
            stats.increment_protocol("IPv6");
        }
        _ => {
            stats.increment_protocol("Other Ethernet");
        }
    }
}

fn handle_ipv4_packet(ipv4: &Ipv4Packet, stats: &mut PacketStats) {
    match ipv4.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                stats.increment_protocol("TCP");
                analyze_tcp_packet(&tcp_packet);
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                stats.increment_protocol("UDP");
                analyze_udp_packet(&udp_packet);
            }
        }
        IpNextHeaderProtocols::Icmp => {
            stats.increment_protocol("ICMP");
        }
        _ => {
            stats.increment_protocol("Other IPv4");
        }
    }
}

fn analyze_tcp_packet(tcp: &TcpPacket) {
    let flags = tcp.get_flags();
    let mut flag_str = String::new();
    
    if flags & 0x02 != 0 { flag_str.push_str("SYN "); }
    if flags & 0x10 != 0 { flag_str.push_str("ACK "); }
    if flags & 0x01 != 0 { flag_str.push_str("FIN "); }
    if flags & 0x04 != 0 { flag_str.push_str("RST "); }
    if flags & 0x08 != 0 { flag_str.push_str("PSH "); }
    
    println!("TCP: {}:{} -> {}:{} [{}]", 
             tcp.get_source(), tcp.get_destination(),
             tcp.get_sequence(), tcp.get_acknowledgement(),
             flag_str.trim());
}

fn analyze_udp_packet(udp: &UdpPacket) {
    println!("UDP: {} -> {} Length: {}", 
             udp.get_source(), udp.get_destination(),
             udp.get_length());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .iter()
        .find(|iface| iface.is_up() && !iface.is_loopback() && !iface.ips.is_empty())
        .ok_or("No suitable network interface found")?;

    println!("Starting packet capture on interface: {}", interface.name);
    
    let config = Config {
        read_timeout: Some(std::time::Duration::from_secs(1)),
        ..Default::default()
    };

    let (mut tx, mut rx) = match datalink::channel(&interface, config) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".into()),
        Err(e) => return Err(format!("Failed to create channel: {}", e).into()),
    };

    let mut stats = PacketStats::new();
    let mut packet_count = 0;
    let max_packets = 100;

    println!("Capturing up to {} packets...", max_packets);

    while packet_count < max_packets {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet) = EthernetPacket::new(packet) {
                    handle_ethernet_frame(&ethernet, &mut stats);
                    packet_count += 1;
                    
                    if packet_count % 10 == 0 {
                        println!("Captured {} packets...", packet_count);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    println!("\nCapture completed.");
    stats.display_stats();
    
    Ok(())
}
```use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str;

#[derive(Debug, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown,
}

#[derive(Debug)]
pub struct Packet {
    pub source_ip: IpAddr,
    pub dest_ip: IpAddr,
    pub protocol: Protocol,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

impl Packet {
    pub fn new(raw_data: &[u8]) -> Option<Self> {
        if raw_data.len() < 20 {
            return None;
        }

        let version = (raw_data[0] >> 4) & 0x0F;
        
        let (source_ip, dest_ip, protocol, payload_start) = match version {
            4 => Self::parse_ipv4(raw_data)?,
            6 => Self::parse_ipv6(raw_data)?,
            _ => return None,
        };

        let protocol_enum = match protocol {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            _ => Protocol::Unknown,
        };

        let payload = raw_data[payload_start..].to_vec();

        Some(Packet {
            source_ip,
            dest_ip,
            protocol: protocol_enum,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    fn parse_ipv4(data: &[u8]) -> Option<(IpAddr, IpAddr, u8, usize)> {
        if data.len() < 20 {
            return None;
        }

        let ihl = (data[0] & 0x0F) as usize * 4;
        let protocol = data[9];
        
        let source_ip = IpAddr::V4(Ipv4Addr::new(
            data[12], data[13], data[14], data[15]
        ));
        
        let dest_ip = IpAddr::V4(Ipv4Addr::new(
            data[16], data[17], data[18], data[19]
        ));

        Some((source_ip, dest_ip, protocol, ihl))
    }

    fn parse_ipv6(data: &[u8]) -> Option<(IpAddr, IpAddr, u8, usize)> {
        if data.len() < 40 {
            return None;
        }

        let next_header = data[6];
        let payload_length = u16::from_be_bytes([data[4], data[5]]) as usize;
        
        let source_ip = IpAddr::V6(Ipv6Addr::from([
            data[8], data[9], data[10], data[11],
            data[12], data[13], data[14], data[15],
            data[16], data[17], data[18], data[19],
            data[20], data[21], data[22], data[23],
        ]));
        
        let dest_ip = IpAddr::V6(Ipv6Addr::from([
            data[24], data[25], data[26], data[27],
            data[28], data[29], data[30], data[31],
            data[32], data[33], data[34], data[35],
            data[36], data[37], data[38], data[39],
        ]));

        Some((source_ip, dest_ip, next_header, 40))
    }

    pub fn extract_http_info(&self) -> Option<String> {
        if self.protocol != Protocol::TCP {
            return None;
        }

        if let Ok(payload_str) = str::from_utf8(&self.payload) {
            if payload_str.starts_with("GET") || payload_str.starts_with("POST") {
                let first_line = payload_str.lines().next()?;
                return Some(first_line.to_string());
            }
        }
        None
    }

    pub fn is_local_traffic(&self) -> bool {
        match (self.source_ip, self.dest_ip) {
            (IpAddr::V4(src), IpAddr::V4(dst)) => {
                src.is_private() || dst.is_private() || 
                src.is_loopback() || dst.is_loopback()
            }
            (IpAddr::V6(src), IpAddr::V6(dst)) => {
                src.is_loopback() || dst.is_loopback()
            }
            _ => false,
        }
    }
}

pub struct PacketAnalyzer {
    packets: Vec<Packet>,
    stats: AnalyzerStats,
}

#[derive(Debug, Default)]
pub struct AnalyzerStats {
    pub total_packets: usize,
    pub tcp_count: usize,
    pub udp_count: usize,
    pub icmp_count: usize,
    pub unknown_count: usize,
    pub local_traffic: usize,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packets: Vec::new(),
            stats: AnalyzerStats::default(),
        }
    }

    pub fn add_packet(&mut self, raw_data: &[u8]) -> bool {
        if let Some(packet) = Packet::new(raw_data) {
            self.update_stats(&packet);
            self.packets.push(packet);
            true
        } else {
            false
        }
    }

    fn update_stats(&mut self, packet: &Packet) {
        self.stats.total_packets += 1;
        
        match packet.protocol {
            Protocol::TCP => self.stats.tcp_count += 1,
            Protocol::UDP => self.stats.udp_count += 1,
            Protocol::ICMP => self.stats.icmp_count += 1,
            Protocol::Unknown => self.stats.unknown_count += 1,
        }

        if packet.is_local_traffic() {
            self.stats.local_traffic += 1;
        }
    }

    pub fn get_stats(&self) -> &AnalyzerStats {
        &self.stats
    }

    pub fn find_http_requests(&self) -> Vec<String> {
        self.packets
            .iter()
            .filter_map(|p| p.extract_http_info())
            .collect()
    }

    pub fn clear(&mut self) {
        self.packets.clear();
        self.stats = AnalyzerStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_packet_parsing() {
        let mut ipv4_data = vec![0x45, 0x00, 0x00, 0x1C];
        ipv4_data.extend(vec![0x00, 0x00, 0x00, 0x00, 0x40, 0x06]);
        ipv4_data.extend(vec![0x00, 0x00]);
        ipv4_data.extend(vec![192, 168, 1, 1]);
        ipv4_data.extend(vec![10, 0, 0, 1]);
        ipv4_data.extend(vec![0x48, 0x54, 0x54, 0x50]);

        let packet = Packet::new(&ipv4_data);
        assert!(packet.is_some());
        
        let packet = packet.unwrap();
        assert_eq!(packet.source_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(packet.dest_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert_eq!(packet.protocol, Protocol::TCP);
    }

    #[test]
    fn test_http_extraction() {
        let http_request = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let mut packet_data = vec![0x45, 0x00, 0x00, 0x3C];
        packet_data.extend(vec![0x00, 0x00, 0x00, 0x00, 0x40, 0x06]);
        packet_data.extend(vec![0x00, 0x00]);
        packet_data.extend(vec![192, 168, 1, 100]);
        packet_data.extend(vec![93, 184, 216, 34]);
        packet_data.extend(http_request);

        let packet = Packet::new(&packet_data).unwrap();
        let http_info = packet.extract_http_info();
        
        assert!(http_info.is_some());
        assert_eq!(http_info.unwrap(), "GET /index.html HTTP/1.1");
    }

    #[test]
    fn test_local_traffic_detection() {
        let local_packet = Packet {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            dest_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            protocol: Protocol::TCP,
            payload: Vec::new(),
            timestamp: 0,
        };

        assert!(local_packet.is_local_traffic());
    }
}