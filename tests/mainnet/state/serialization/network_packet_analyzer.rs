use pnet::datalink::{self, Channel, DataLinkReceiver, NetworkInterface};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::env;
use std::process;

struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    source_ips: HashMap<String, u64>,
    destination_ips: HashMap<String, u64>,
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

    fn update(&mut self, protocol: &str, src_ip: &str, dst_ip: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
        *self.source_ips.entry(src_ip.to_string()).or_insert(0) += 1;
        *self.destination_ips.entry(dst_ip.to_string()).or_insert(0) += 1;
    }

    fn display_summary(&self) {
        println!("Packet Capture Summary:");
        println!("Total packets captured: {}", self.total_packets);
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            println!("  {}: {}", protocol, count);
        }
        println!("\nTop 5 Source IPs:");
        self.display_top_ips(&self.source_ips);
        println!("\nTop 5 Destination IPs:");
        self.display_top_ips(&self.destination_ips);
    }

    fn display_top_ips(&self, ip_map: &HashMap<String, u64>) {
        let mut ips: Vec<_> = ip_map.iter().collect();
        ips.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in ips.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
    }
}

fn capture_packets(interface_name: &str, packet_limit: u64) -> Result<(), String> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".to_string()),
        Err(e) => return Err(format!("Failed to create channel: {}", e)),
    };

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop capture and display statistics\n");

    let mut stats = PacketStats::new();
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                process_packet(&packet, &mut stats);

                if packet_count >= packet_limit {
                    println!("Reached packet limit of {}", packet_limit);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                continue;
            }
        }
    }

    stats.display_summary();
    Ok(())
}

fn process_packet(packet_data: &[u8], stats: &mut PacketStats) {
    if let Some(ethernet_packet) = EthernetPacket::new(packet_data) {
        match ethernet_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                    let src_ip = ipv4_packet.get_source().to_string();
                    let dst_ip = ipv4_packet.get_destination().to_string();

                    match ipv4_packet.get_next_level_protocol() {
                        IpNextHeaderProtocols::Tcp => {
                            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                                stats.update(
                                    "TCP",
                                    &src_ip,
                                    &dst_ip,
                                );
                                print_packet_info(
                                    "TCP",
                                    &src_ip,
                                    &dst_ip,
                                    tcp_packet.get_source(),
                                    tcp_packet.get_destination(),
                                    tcp_packet.payload().len(),
                                );
                            }
                        }
                        IpNextHeaderProtocols::Udp => {
                            if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                                stats.update(
                                    "UDP",
                                    &src_ip,
                                    &dst_ip,
                                );
                                print_packet_info(
                                    "UDP",
                                    &src_ip,
                                    &dst_ip,
                                    udp_packet.get_source(),
                                    udp_packet.get_destination(),
                                    udp_packet.payload().len(),
                                );
                            }
                        }
                        _ => {
                            stats.update(
                                "Other IPv4",
                                &src_ip,
                                &dst_ip,
                            );
                        }
                    }
                }
            }
            _ => {
                stats.update(
                    "Non-IPv4",
                    "Unknown",
                    "Unknown",
                );
            }
        }
    }
}

fn print_packet_info(
    protocol: &str,
    src_ip: &str,
    dst_ip: &str,
    src_port: u16,
    dst_port: u16,
    payload_size: usize,
) {
    println!(
        "[{}] {}:{} -> {}:{} ({} bytes)",
        protocol, src_ip, src_port, dst_ip, dst_port, payload_size
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <interface_name> [packet_limit]", args[0]);
        eprintln!("Example: {} eth0 100", args[0]);
        process::exit(1);
    }

    let interface_name = &args[1];
    let packet_limit = if args.len() > 2 {
        args[2].parse().unwrap_or(100)
    } else {
        100
    };

    if let Err(e) = capture_packets(interface_name, packet_limit) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}