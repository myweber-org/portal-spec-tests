
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
```