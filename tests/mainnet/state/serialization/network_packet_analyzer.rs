
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
}