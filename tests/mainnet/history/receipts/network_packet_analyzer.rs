use std::net::UdpSocket;
use std::str;

struct PacketInfo {
    source_port: u16,
    dest_port: u16,
    payload: Vec<u8>,
    protocol_type: Protocol,
}

#[derive(Debug, PartialEq)]
enum Protocol {
    HTTP,
    DNS,
    UNKNOWN,
}

impl PacketInfo {
    fn new(source_port: u16, dest_port: u16, payload: Vec<u8>) -> Self {
        let protocol_type = PacketInfo::detect_protocol(source_port, dest_port, &payload);
        PacketInfo {
            source_port,
            dest_port,
            payload,
            protocol_type,
        }
    }

    fn detect_protocol(source_port: u16, dest_port: u16, payload: &[u8]) -> Protocol {
        match (source_port, dest_port) {
            (53, _) | (_, 53) => Protocol::DNS,
            (80, _) | (_, 80) | (443, _) | (_, 443) => Protocol::HTTP,
            _ => {
                if PacketInfo::contains_http_keywords(payload) {
                    Protocol::HTTP
                } else {
                    Protocol::UNKNOWN
                }
            }
        }
    }

    fn contains_http_keywords(payload: &[u8]) -> bool {
        let keywords = ["GET", "POST", "HTTP", "Host:"];
        if let Ok(payload_str) = str::from_utf8(payload) {
            keywords.iter().any(|&kw| payload_str.contains(kw))
        } else {
            false
        }
    }

    fn display(&self) {
        println!("Packet Analysis:");
        println!("  Source Port: {}", self.source_port);
        println!("  Destination Port: {}", self.dest_port);
        println!("  Protocol: {:?}", self.protocol_type);
        println!("  Payload Length: {} bytes", self.payload.len());
        
        if let Ok(payload_str) = str::from_utf8(&self.payload) {
            if !payload_str.is_empty() {
                println!("  Payload Preview: {}", 
                    payload_str.chars().take(50).collect::<String>());
            }
        }
    }
}

fn capture_packets(interface: &str, port: u16) -> std::io::Result<()> {
    let bind_addr = format!("{}:{}", interface, port);
    let socket = UdpSocket::bind(bind_addr)?;
    println!("Listening on {}:{}", interface, port);
    
    let mut buffer = [0; 1024];
    
    loop {
        match socket.recv_from(&mut buffer) {
            Ok((size, src_addr)) => {
                let payload = buffer[..size].to_vec();
                let packet = PacketInfo::new(
                    src_addr.port(),
                    port,
                    payload
                );
                packet.display();
                println!("  From: {}", src_addr);
                println!("{}", "-".repeat(40));
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

fn main() {
    let interface = "127.0.0.1";
    let port = 8080;
    
    if let Err(e) = capture_packets(interface, port) {
        eprintln!("Failed to capture packets: {}", e);
    }
}use std::fmt;

#[derive(Debug)]
pub struct EthernetFrame {
    pub destination_mac: [u8; 6],
    pub source_mac: [u8; 6],
    pub ethertype: u16,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 14 {
            return None;
        }

        let mut dest_mac = [0u8; 6];
        dest_mac.copy_from_slice(&data[0..6]);

        let mut src_mac = [0u8; 6];
        src_mac.copy_from_slice(&data[6..12]);

        let ethertype = u16::from_be_bytes([data[12], data[13]]);

        let payload = data[14..].to_vec();

        Some(EthernetFrame {
            destination_mac: dest_mac,
            source_mac: src_mac,
            ethertype,
            payload,
        })
    }

    pub fn ethertype_name(&self) -> &'static str {
        match self.ethertype {
            0x0800 => "IPv4",
            0x0806 => "ARP",
            0x86DD => "IPv6",
            _ => "Unknown",
        }
    }
}

impl fmt::Display for EthernetFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Destination: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n\
             Source:      {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n\
             EtherType:   0x{:04X} ({})\n\
             Payload:     {} bytes",
            self.destination_mac[0],
            self.destination_mac[1],
            self.destination_mac[2],
            self.destination_mac[3],
            self.destination_mac[4],
            self.destination_mac[5],
            self.source_mac[0],
            self.source_mac[1],
            self.source_mac[2],
            self.source_mac[3],
            self.source_mac[4],
            self.source_mac[5],
            self.ethertype,
            self.ethertype_name(),
            self.payload.len()
        )
    }
}

pub fn analyze_packet(packet_data: &[u8]) {
    match EthernetFrame::from_bytes(packet_data) {
        Some(frame) => {
            println!("Ethernet Frame Analysis:");
            println!("{}", frame);
            
            if frame.payload.len() > 0 {
                println!("\nFirst 16 bytes of payload:");
                for (i, byte) in frame.payload.iter().take(16).enumerate() {
                    if i % 8 == 0 && i > 0 {
                        print!("  ");
                    }
                    print!("{:02X} ", byte);
                }
                println!();
            }
        }
        None => println!("Invalid packet data: too short for Ethernet frame"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ethernet_frame() {
        let mut test_data = vec![0u8; 64];
        test_data[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        test_data[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        test_data[12..14].copy_from_slice(&[0x08, 0x00]);
        
        let frame = EthernetFrame::from_bytes(&test_data).unwrap();
        
        assert_eq!(frame.destination_mac, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(frame.source_mac, [0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        assert_eq!(frame.ethertype, 0x0800);
        assert_eq!(frame.ethertype_name(), "IPv4");
        assert_eq!(frame.payload.len(), 50);
    }

    #[test]
    fn test_invalid_frame() {
        let short_data = vec![0u8; 10];
        assert!(EthernetFrame::from_bytes(&short_data).is_none());
    }
}use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
    protocol: Protocol,
    payload_size: usize,
    timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

pub struct PacketAnalyzer {
    packet_count: usize,
    total_bytes: usize,
    protocol_distribution: HashMap<Protocol, usize>,
    ip_traffic: HashMap<Ipv4Addr, usize>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            total_bytes: 0,
            protocol_distribution: HashMap::new(),
            ip_traffic: HashMap::new(),
        }
    }

    pub fn process_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;
        self.total_bytes += packet.payload_size;

        *self.protocol_distribution
            .entry(packet.protocol.clone())
            .or_insert(0) += 1;

        *self.ip_traffic
            .entry(packet.source_ip)
            .or_insert(0) += 1;
        *self.ip_traffic
            .entry(packet.destination_ip)
            .or_insert(0) += 1;
    }

    pub fn get_statistics(&self) -> AnalyzerStats {
        let avg_packet_size = if self.packet_count > 0 {
            self.total_bytes / self.packet_count
        } else {
            0
        };

        let top_source = self.ip_traffic
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(ip, count)| (ip.to_string(), *count))
            .unwrap_or_else(|| ("0.0.0.0".to_string(), 0));

        AnalyzerStats {
            total_packets: self.packet_count,
            total_bytes: self.total_bytes,
            average_packet_size: avg_packet_size,
            top_source_ip: top_source.0,
            top_source_count: top_source.1,
            protocol_counts: self.protocol_distribution.clone(),
        }
    }

    pub fn reset(&mut self) {
        self.packet_count = 0;
        self.total_bytes = 0;
        self.protocol_distribution.clear();
        self.ip_traffic.clear();
    }
}

#[derive(Debug)]
pub struct AnalyzerStats {
    pub total_packets: usize,
    pub total_bytes: usize,
    pub average_packet_size: usize,
    pub top_source_ip: String,
    pub top_source_count: usize,
    pub protocol_counts: HashMap<Protocol, usize>,
}

impl Protocol {
    pub fn from_u8(value: u8) -> Self {
        match value {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            other => Protocol::Other(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_processing() {
        let mut analyzer = PacketAnalyzer::new();
        
        let packet1 = NetworkPacket {
            source_ip: Ipv4Addr::new(192, 168, 1, 1),
            destination_ip: Ipv4Addr::new(192, 168, 1, 2),
            protocol: Protocol::TCP,
            payload_size: 1500,
            timestamp: 1234567890,
        };

        let packet2 = NetworkPacket {
            source_ip: Ipv4Addr::new(192, 168, 1, 2),
            destination_ip: Ipv4Addr::new(192, 168, 1, 1),
            protocol: Protocol::UDP,
            payload_size: 512,
            timestamp: 1234567891,
        };

        analyzer.process_packet(&packet1);
        analyzer.process_packet(&packet2);

        let stats = analyzer.get_statistics();
        
        assert_eq!(stats.total_packets, 2);
        assert_eq!(stats.total_bytes, 2012);
        assert_eq!(stats.average_packet_size, 1006);
        assert_eq!(*stats.protocol_counts.get(&Protocol::TCP).unwrap(), 1);
        assert_eq!(*stats.protocol_counts.get(&Protocol::UDP).unwrap(), 1);
    }

    #[test]
    fn test_protocol_conversion() {
        assert_eq!(Protocol::from_u8(6), Protocol::TCP);
        assert_eq!(Protocol::from_u8(17), Protocol::UDP);
        assert_eq!(Protocol::from_u8(1), Protocol::ICMP);
        assert_eq!(Protocol::from_u8(99), Protocol::Other(99));
    }
}