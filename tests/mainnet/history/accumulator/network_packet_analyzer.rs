use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interface_name = env::args().nth(1).unwrap_or_else(|| "eth0".to_string());
    
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .expect("Interface not found");

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unsupported channel type"),
        Err(e) => panic!("Failed to create channel: {}", e),
    };

    let mut packet_stats = HashMap::new();
    println!("Starting packet capture on interface: {}", interface_name);

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_ethernet_frame(&ethernet_packet, &mut packet_stats);
                }
            }
            Err(e) => {
                eprintln!("Failed to receive packet: {}", e);
                break;
            }
        }

        if packet_stats.len() > 100 {
            display_statistics(&packet_stats);
            packet_stats.clear();
        }
    }

    Ok(())
}

fn process_ethernet_frame(
    ethernet: &EthernetPacket,
    stats: &mut HashMap<String, usize>,
) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                process_ipv4_packet(&ipv4_packet, stats);
            }
        }
        EtherTypes::Ipv6 => {
            *stats.entry("IPv6".to_string()).or_insert(0) += 1;
        }
        EtherTypes::Arp => {
            *stats.entry("ARP".to_string()).or_insert(0) += 1;
        }
        _ => {
            *stats.entry("Other".to_string()).or_insert(0) += 1;
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet, stats: &mut HashMap<String, usize>) {
    let protocol_key = match ipv4.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                format!("TCP:{}->{}", tcp_packet.get_source(), tcp_packet.get_destination())
            } else {
                "TCP".to_string()
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                format!("UDP:{}->{}", udp_packet.get_source(), udp_packet.get_destination())
            } else {
                "UDP".to_string()
            }
        }
        IpNextHeaderProtocols::Icmp => "ICMP".to_string(),
        _ => "Other-IPv4".to_string(),
    };
    
    *stats.entry(protocol_key).or_insert(0) += 1;
}

fn display_statistics(stats: &HashMap<String, usize>) {
    println!("\n=== Packet Statistics ===");
    let total: usize = stats.values().sum();
    
    let mut sorted_stats: Vec<(&String, &usize)> = stats.iter().collect();
    sorted_stats.sort_by(|a, b| b.1.cmp(a.1));
    
    for (protocol, count) in sorted_stats.iter().take(10) {
        let percentage = (*count as f64 / total as f64) * 100.0;
        println!("{:<30} {:>6} packets ({:5.1}%)", protocol, count, percentage);
    }
    println!("Total packets: {}", total);
}