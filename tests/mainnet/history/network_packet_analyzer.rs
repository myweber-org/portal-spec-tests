use pcap::{Capture, Device};
use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug)]
struct PacketMetadata {
    timestamp: f64,
    source_ip: Ipv4Addr,
    dest_ip: Ipv4Addr,
    protocol: u8,
    length: usize,
}

struct ProtocolAnalyzer {
    packet_count: u64,
    protocol_distribution: HashMap<u8, u64>,
    traffic_by_ip: HashMap<Ipv4Addr, u64>,
}

impl ProtocolAnalyzer {
    fn new() -> Self {
        ProtocolAnalyzer {
            packet_count: 0,
            protocol_distribution: HashMap::new(),
            traffic_by_ip: HashMap::new(),
        }
    }

    fn process_packet(&mut self, metadata: &PacketMetadata) {
        self.packet_count += 1;
        
        *self.protocol_distribution
            .entry(metadata.protocol)
            .or_insert(0) += 1;
        
        *self.traffic_by_ip
            .entry(metadata.source_ip)
            .or_insert(0) += metadata.length as u64;
        
        *self.traffic_by_ip
            .entry(metadata.dest_ip)
            .or_insert(0) += metadata.length as u64;
    }

    fn print_statistics(&self) {
        println!("Total packets captured: {}", self.packet_count);
        println!("\nProtocol distribution:");
        for (protocol, count) in &self.protocol_distribution {
            println!("  Protocol {}: {} packets", protocol, count);
        }
        
        println!("\nTop 5 IP addresses by traffic:");
        let mut ip_traffic: Vec<(&Ipv4Addr, &u64)> = self.traffic_by_ip.iter().collect();
        ip_traffic.sort_by(|a, b| b.1.cmp(a.1));
        
        for (i, (ip, traffic)) in ip_traffic.iter().take(5).enumerate() {
            println!("  {}. {}: {} bytes", i + 1, ip, traffic);
        }
    }
}

fn extract_packet_metadata(packet: &[u8]) -> Option<PacketMetadata> {
    if packet.len() < 20 {
        return None;
    }
    
    let version = (packet[0] >> 4) & 0x0F;
    if version != 4 {
        return None;
    }
    
    let ihl = (packet[0] & 0x0F) as usize * 4;
    if packet.len() < ihl {
        return None;
    }
    
    let source_ip = Ipv4Addr::new(packet[12], packet[13], packet[14], packet[15]);
    let dest_ip = Ipv4Addr::new(packet[16], packet[17], packet[18], packet[19]);
    let protocol = packet[9];
    
    Some(PacketMetadata {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
        source_ip,
        dest_ip,
        protocol,
        length: packet.len(),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::lookup()?.ok_or("No network device found")?;
    let mut cap = Capture::from_device(device)?
        .promisc(true)
        .snaplen(65535)
        .open()?;
    
    let mut analyzer = ProtocolAnalyzer::new();
    let mut packet_counter = 0;
    let max_packets = 100;
    
    println!("Starting packet capture on network interface...");
    
    while packet_counter < max_packets {
        match cap.next_packet() {
            Ok(packet) => {
                if let Some(metadata) = extract_packet_metadata(&packet.data) {
                    analyzer.process_packet(&metadata);
                    packet_counter += 1;
                    
                    if packet_counter % 10 == 0 {
                        println!("Processed {} packets...", packet_counter);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error capturing packet: {}", e);
                continue;
            }
        }
    }
    
    println!("\nCapture complete!");
    analyzer.print_statistics();
    
    Ok(())
}