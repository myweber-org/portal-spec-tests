
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    source_ips: HashMap<String, u64>,
    destination_ips: HashMap<String, u64>,
    start_time: u64,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            source_ips: HashMap::new(),
            destination_ips: HashMap::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn update(&mut self, ethernet: &EthernetPacket) {
        self.total_packets += 1;

        match ethernet.get_ethertype() {
            EtherTypes::Ipv4 => {
                *self.protocol_counts.entry("IPv4".to_string()).or_insert(0) += 1;
                
                if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                    let src_ip = ipv4_packet.get_source().to_string();
                    let dst_ip = ipv4_packet.get_destination().to_string();
                    
                    *self.source_ips.entry(src_ip).or_insert(0) += 1;
                    *self.destination_ips.entry(dst_ip).or_insert(0) += 1;

                    match ipv4_packet.get_next_level_protocol() {
                        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                            *self.protocol_counts.entry("TCP".to_string()).or_insert(0) += 1;
                            
                            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                                let src_port = tcp_packet.get_source();
                                let dst_port = tcp_packet.get_destination();
                                
                                if dst_port == 80 || src_port == 80 {
                                    *self.protocol_counts.entry("HTTP".to_string()).or_insert(0) += 1;
                                } else if dst_port == 443 || src_port == 443 {
                                    *self.protocol_counts.entry("HTTPS".to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                            *self.protocol_counts.entry("UDP".to_string()).or_insert(0) += 1;
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Icmp => {
                            *self.protocol_counts.entry("ICMP".to_string()).or_insert(0) += 1;
                        }
                        _ => {
                            *self.protocol_counts.entry("Other".to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
            EtherTypes::Arp => {
                *self.protocol_counts.entry("ARP".to_string()).or_insert(0) += 1;
            }
            _ => {
                *self.protocol_counts.entry("Unknown".to_string()).or_insert(0) += 1;
            }
        }
    }

    fn display_summary(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let duration = current_time - self.start_time;
        
        println!("Packet Capture Summary");
        println!("======================");
        println!("Capture duration: {} seconds", duration);
        println!("Total packets: {}", self.total_packets);
        
        if duration > 0 {
            println!("Packets per second: {:.2}", self.total_packets as f64 / duration as f64);
        }
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.2}%)", protocol, count, percentage);
        }
        
        println!("\nTop 5 Source IPs:");
        let mut source_vec: Vec<(&String, &u64)> = self.source_ips.iter().collect();
        source_vec.sort_by(|a, b| b.1.cmp(a.1));
        
        for (i, (ip, count)) in source_vec.iter().take(5).enumerate() {
            println!("  {}. {}: {}", i + 1, ip, count);
        }
        
        println!("\nTop 5 Destination IPs:");
        let mut dest_vec: Vec<(&String, &u64)> = self.destination_ips.iter().collect();
        dest_vec.sort_by(|a, b| b.1.cmp(a.1));
        
        for (i, (ip, count)) in dest_vec.iter().take(5).enumerate() {
            println!("  {}. {}: {}", i + 1, ip, count);
        }
    }
}

fn capture_packets(interface_name: &str, packet_limit: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
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

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop and display statistics\n");

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    stats.update(&ethernet_packet);
                    packet_count += 1;

                    if packet_count % 100 == 0 {
                        print!("\rPackets captured: {}", packet_count);
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    }

                    if let Some(limit) = packet_limit {
                        if packet_count >= limit {
                            println!("\n\nReached packet limit of {}", limit);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                continue;
            }
        }
    }

    println!("\n");
    stats.display_summary();
    Ok(())
}

fn main() {
    let interface_name = "eth0";
    let packet_limit = Some(1000u64);

    if let Err(e) = capture_packets(interface_name, packet_limit) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}