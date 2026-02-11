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
}use pnet::datalink::{self, Channel, NetworkInterface};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

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

    fn increment_protocol(&mut self, protocol: &str) {
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
        self.total_packets += 1;
    }

    fn display_stats(&self) {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - self.start_time;
        println!("Packet capture statistics ({} ms elapsed):", elapsed);
        println!("Total packets: {}", self.total_packets);
        if elapsed > 0 {
            println!("Packets/sec: {:.2}", self.total_packets as f64 / (elapsed as f64 / 1000.0));
        }
        println!("Protocol distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.2}%)", protocol, count, percentage);
        }
    }
}

fn handle_transport_packet(
    source_ip: &str,
    dest_ip: &str,
    protocol: IpNextHeaderProtocol,
    transport_data: &[u8],
    stats: &mut PacketStats,
) {
    match protocol {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(transport_data) {
                stats.increment_protocol("TCP");
                println!(
                    "TCP: {}:{} -> {}:{} [Seq: {} Ack: {} Win: {}]",
                    source_ip,
                    tcp_packet.get_source(),
                    dest_ip,
                    tcp_packet.get_destination(),
                    tcp_packet.get_sequence(),
                    tcp_packet.get_acknowledgement(),
                    tcp_packet.get_window()
                );
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(transport_data) {
                stats.increment_protocol("UDP");
                println!(
                    "UDP: {}:{} -> {}:{} Length: {}",
                    source_ip,
                    udp_packet.get_source(),
                    dest_ip,
                    udp_packet.get_destination(),
                    udp_packet.get_length()
                );
            }
        }
        IpNextHeaderProtocols::Icmp => {
            stats.increment_protocol("ICMP");
            println!("ICMP: {} -> {} ({} bytes)", source_ip, dest_ip, transport_data.len());
        }
        _ => {
            stats.increment_protocol("Other-IP");
            println!("Other IP protocol {:?}: {} -> {}", protocol, source_ip, dest_ip);
        }
    }
}

fn handle_ipv4_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
        let source_ip = ipv4_packet.get_source().to_string();
        let dest_ip = ipv4_packet.get_destination().to_string();
        let protocol = ipv4_packet.get_next_level_protocol();
        
        stats.increment_protocol("IPv4");
        handle_transport_packet(&source_ip, &dest_ip, protocol, ipv4_packet.payload(), stats);
    }
}

fn capture_packets(interface_name: &str, max_packets: Option<u64>) -> Result<(), String> {
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
    println!("Press Ctrl+C to stop and display statistics\n");

    let mut stats = PacketStats::new();
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    match ethernet_packet.get_ethertype() {
                        EtherTypes::Ipv4 => handle_ipv4_packet(&ethernet_packet, &mut stats),
                        EtherTypes::Ipv6 => {
                            stats.increment_protocol("IPv6");
                            println!("IPv6 packet ({} bytes)", ethernet_packet.payload().len());
                        }
                        EtherTypes::Arp => {
                            stats.increment_protocol("ARP");
                            println!("ARP packet ({} bytes)", ethernet_packet.payload().len());
                        }
                        _ => {
                            stats.increment_protocol("Other-Ethernet");
                        }
                    }
                }

                packet_count += 1;
                if let Some(max) = max_packets {
                    if packet_count >= max {
                        println!("\nReached maximum packet count of {}", max);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    stats.display_stats();
    Ok(())
}

fn list_interfaces() {
    println!("Available network interfaces:");
    for interface in datalink::interfaces() {
        println!("  {}: {}", interface.name, interface.description);
        for ip in interface.ips {
            println!("    {}", ip);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <interface_name> [max_packets]", args[0]);
        println!("       {} --list", args[0]);
        return;
    }

    if args[1] == "--list" {
        list_interfaces();
        return;
    }

    let interface_name = &args[1];
    let max_packets = if args.len() > 2 {
        args[2].parse::<u64>().ok()
    } else {
        None
    };

    if let Err(e) = capture_packets(interface_name, max_packets) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}