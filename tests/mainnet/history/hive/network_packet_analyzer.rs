use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    capture: Capture<pcap::Active>,
}

impl PacketAnalyzer {
    pub fn new(interface_name: &str) -> Result<Self, Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|dev| dev.name == interface_name)
            .ok_or_else(|| format!("Interface {} not found", interface_name))?;

        let capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;

        Ok(PacketAnalyzer { capture })
    }

    pub fn start_capture(&mut self, packet_count: usize) -> Result<(), Box<dyn Error>> {
        println!("Starting packet capture on interface...");
        
        for i in 0..packet_count {
            match self.capture.next_packet() {
                Ok(packet) => {
                    println!("Packet {}: {} bytes captured", i + 1, packet.header.len);
                    self.analyze_packet(&packet);
                }
                Err(e) => {
                    eprintln!("Error capturing packet: {}", e);
                    break;
                }
            }
        }
        
        println!("Capture completed.");
        Ok(())
    }

    fn analyze_packet(&self, packet: &pcap::Packet) {
        if packet.data.len() >= 14 {
            let eth_type = u16::from_be_bytes([packet.data[12], packet.data[13]]);
            
            match eth_type {
                0x0800 => println!("  Protocol: IPv4"),
                0x0806 => println!("  Protocol: ARP"),
                0x86DD => println!("  Protocol: IPv6"),
                _ => println!("  Protocol: Unknown (0x{:04x})", eth_type),
            }
        }
        
        if packet.data.len() >= 34 {
            let protocol = packet.data[23];
            match protocol {
                1 => println!("  Transport: ICMP"),
                6 => println!("  Transport: TCP"),
                17 => println!("  Transport: UDP"),
                _ => println!("  Transport: Unknown ({})", protocol),
            }
        }
    }

    pub fn get_statistics(&self) -> Result<pcap::Stat, Box<dyn Error>> {
        let stats = self.capture.stats()?;
        println!("Packets received: {}", stats.received);
        println!("Packets dropped: {}", stats.dropped);
        println!("Packets dropped by interface: {}", stats.if_dropped);
        Ok(stats)
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  {}: {}", device.name, device.desc.unwrap_or_default());
    }
    Ok(())
}use pnet::datalink::{self, Channel, Config};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct PacketStats {
    pub total_packets: u64,
    pub protocol_counts: HashMap<String, u64>,
    pub start_time: Instant,
}

impl PacketStats {
    pub fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    pub fn update(&mut self, protocol: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    pub fn display_summary(&self) {
        let duration = self.start_time.elapsed();
        println!("Packet Capture Summary:");
        println!("Duration: {:.2} seconds", duration.as_secs_f64());
        println!("Total packets captured: {}", self.total_packets);
        println!("Packets per second: {:.2}", 
                 self.total_packets as f64 / duration.as_secs_f64());
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

pub fn capture_packets(interface_name: &str, duration_secs: u64) -> Result<PacketStats, String> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    let config = Config {
        read_timeout: Some(Duration::from_secs(1)),
        ..Default::default()
    };

    let (mut rx, _) = match datalink::channel(&interface, config) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".to_string()),
        Err(e) => return Err(format!("Failed to create channel: {}", e)),
    };

    let mut stats = PacketStats::new();
    let end_time = Instant::now() + Duration::from_secs(duration_secs);

    println!("Starting packet capture on {} for {} seconds...", interface_name, duration_secs);

    while Instant::now() < end_time {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_ethernet_packet(&ethernet_packet, &mut stats);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
            }
        }
    }

    Ok(stats)
}

fn process_ethernet_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                process_ipv4_packet(&ipv4_packet, stats);
            }
        }
        EtherTypes::Ipv6 => {
            stats.update("IPv6");
        }
        EtherTypes::Arp => {
            stats.update("ARP");
        }
        _ => {
            stats.update("Other");
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet, stats: &mut PacketStats) {
    match ipv4.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                stats.update("TCP");
                log_tcp_packet(ipv4, &tcp_packet);
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                stats.update("UDP");
                log_udp_packet(ipv4, &udp_packet);
            }
        }
        IpNextHeaderProtocols::Icmp => {
            stats.update("ICMP");
        }
        _ => {
            stats.update("Other-IPv4");
        }
    }
}

fn log_tcp_packet(ipv4: &Ipv4Packet, tcp: &TcpPacket) {
    println!(
        "TCP Packet: {}:{} -> {}:{} | Flags: {:?} | Seq: {} | Ack: {} | Window: {}",
        ipv4.get_source(),
        tcp.get_source(),
        ipv4.get_destination(),
        tcp.get_destination(),
        get_tcp_flags(tcp),
        tcp.get_sequence(),
        tcp.get_acknowledgement(),
        tcp.get_window()
    );
}

fn log_udp_packet(ipv4: &Ipv4Packet, udp: &UdpPacket) {
    println!(
        "UDP Packet: {}:{} -> {}:{} | Length: {}",
        ipv4.get_source(),
        udp.get_source(),
        ipv4.get_destination(),
        udp.get_destination(),
        udp.get_length()
    );
}

fn get_tcp_flags(tcp: &TcpPacket) -> String {
    let mut flags = Vec::new();
    if tcp.get_fin() { flags.push("FIN"); }
    if tcp.get_syn() { flags.push("SYN"); }
    if tcp.get_rst() { flags.push("RST"); }
    if tcp.get_psh() { flags.push("PSH"); }
    if tcp.get_ack() { flags.push("ACK"); }
    if tcp.get_urg() { flags.push("URG"); }
    if tcp.get_ece() { flags.push("ECE"); }
    if tcp.get_cwr() { flags.push("CWR"); }
    flags.join("|")
}

pub fn list_interfaces() -> Vec<String> {
    datalink::interfaces()
        .iter()
        .map(|iface| iface.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_stats_new() {
        let stats = PacketStats::new();
        assert_eq!(stats.total_packets, 0);
        assert!(stats.protocol_counts.is_empty());
    }

    #[test]
    fn test_packet_stats_update() {
        let mut stats = PacketStats::new();
        stats.update("TCP");
        stats.update("TCP");
        stats.update("UDP");
        
        assert_eq!(stats.total_packets, 3);
        assert_eq!(stats.protocol_counts.get("TCP"), Some(&2));
        assert_eq!(stats.protocol_counts.get("UDP"), Some(&1));
    }

    #[test]
    fn test_list_interfaces() {
        let interfaces = list_interfaces();
        assert!(!interfaces.is_empty());
    }
}