use pcap::{Capture, Device};
use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug)]
struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    source_ips: HashMap<Ipv4Addr, u64>,
    destination_ips: HashMap<Ipv4Addr, u64>,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            source_ips: HashMap::new(),
            destination_ips: HashMap::new(),
        }
    }

    fn analyze_packet(&mut self, packet: &[u8]) {
        self.total_packets += 1;

        if packet.len() >= 20 {
            let protocol_byte = packet[9];
            let protocol = match protocol_byte {
                6 => "TCP",
                17 => "UDP",
                1 => "ICMP",
                _ => "OTHER",
            };
            *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;

            if packet.len() >= 24 {
                let src_ip = Ipv4Addr::new(packet[12], packet[13], packet[14], packet[15]);
                let dst_ip = Ipv4Addr::new(packet[16], packet[17], packet[18], packet[19]);
                
                *self.source_ips.entry(src_ip).or_insert(0) += 1;
                *self.destination_ips.entry(dst_ip).or_insert(0) += 1;
            }
        }
    }

    fn print_summary(&self) {
        println!("Packet Analysis Summary:");
        println!("Total packets captured: {}", self.total_packets);
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            println!("  {}: {}", protocol, count);
        }
        println!("\nTop Source IPs:");
        let mut sorted_src: Vec<_> = self.source_ips.iter().collect();
        sorted_src.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in sorted_src.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
    }
}

fn capture_packets(device_name: &str, packet_limit: u32) -> Result<PacketStats, pcap::Error> {
    let device = Device::list()?
        .into_iter()
        .find(|dev| dev.name == device_name)
        .ok_or_else(|| pcap::Error::InvalidString)?;

    let mut cap = Capture::from_device(device)?
        .promisc(true)
        .timeout(1000)
        .open()?;

    let mut stats = PacketStats::new();
    let mut packet_count = 0;

    while let Ok(packet) = cap.next() {
        stats.analyze_packet(&packet.data);
        packet_count += 1;
        
        if packet_count >= packet_limit {
            break;
        }
    }

    Ok(stats)
}

fn main() {
    let default_device = "eth0";
    
    match capture_packets(default_device, 100) {
        Ok(stats) => {
            stats.print_summary();
        }
        Err(e) => {
            eprintln!("Error capturing packets: {}", e);
        }
    }
}