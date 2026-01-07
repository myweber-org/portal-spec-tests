
use pnet::datalink::{self, Channel, Config};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct PacketAnalyzer {
    interface_name: String,
    capture_duration: Duration,
    protocol_stats: HashMap<String, usize>,
    total_packets: usize,
    start_time: Instant,
}

impl PacketAnalyzer {
    pub fn new(interface: &str, duration_secs: u64) -> Self {
        PacketAnalyzer {
            interface_name: interface.to_string(),
            capture_duration: Duration::from_secs(duration_secs),
            protocol_stats: HashMap::new(),
            total_packets: 0,
            start_time: Instant::now(),
        }
    }

    pub fn start_capture(&mut self) -> Result<(), String> {
        let interfaces = datalink::interfaces();
        let interface = interfaces
            .into_iter()
            .find(|iface| iface.name == self.interface_name)
            .ok_or_else(|| format!("Interface {} not found", self.interface_name))?;

        let config = Config::default();
        let (_, mut rx) = match datalink::channel(&interface, config) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => return Err("Unsupported channel type".to_string()),
            Err(e) => return Err(format!("Failed to create channel: {}", e)),
        };

        println!("Starting packet capture on {}...", self.interface_name);
        
        while self.start_time.elapsed() < self.capture_duration {
            match rx.next() {
                Ok(packet) => {
                    self.process_packet(&packet);
                    self.total_packets += 1;
                }
                Err(e) => eprintln!("Error receiving packet: {}", e),
            }
        }

        self.display_statistics();
        Ok(())
    }

    fn process_packet(&mut self, data: &[u8]) {
        if let Some(eth_packet) = EthernetPacket::new(data) {
            match eth_packet.get_ethertype() {
                pnet::packet::ethernet::EtherTypes::Ipv4 => {
                    self.analyze_ipv4_packet(eth_packet.payload());
                }
                pnet::packet::ethernet::EtherTypes::Ipv6 => {
                    self.increment_protocol("IPv6");
                }
                pnet::packet::ethernet::EtherTypes::Arp => {
                    self.increment_protocol("ARP");
                }
                _ => self.increment_protocol("Other"),
            }
        }
    }

    fn analyze_ipv4_packet(&mut self, data: &[u8]) {
        if let Some(ip_packet) = Ipv4Packet::new(data) {
            match ip_packet.get_next_level_protocol() {
                IpNextHeaderProtocols::Tcp => {
                    self.increment_protocol("TCP");
                    self.analyze_tcp_packet(ip_packet.payload());
                }
                IpNextHeaderProtocols::Udp => {
                    self.increment_protocol("UDP");
                    self.analyze_udp_packet(ip_packet.payload());
                }
                IpNextHeaderProtocols::Icmp => {
                    self.increment_protocol("ICMP");
                }
                _ => self.increment_protocol("Other-IP"),
            }
        }
    }

    fn analyze_tcp_packet(&mut self, data: &[u8]) {
        if let Some(tcp_packet) = TcpPacket::new(data) {
            let src_port = tcp_packet.get_source();
            let dst_port = tcp_packet.get_destination();
            
            if src_port == 80 || dst_port == 80 {
                self.increment_protocol("HTTP");
            } else if src_port == 443 || dst_port == 443 {
                self.increment_protocol("HTTPS");
            } else if src_port == 22 || dst_port == 22 {
                self.increment_protocol("SSH");
            }
        }
    }

    fn analyze_udp_packet(&mut self, data: &[u8]) {
        if let Some(udp_packet) = UdpPacket::new(data) {
            let src_port = udp_packet.get_source();
            let dst_port = udp_packet.get_destination();
            
            if src_port == 53 || dst_port == 53 {
                self.increment_protocol("DNS");
            } else if src_port == 67 || dst_port == 67 || src_port == 68 || dst_port == 68 {
                self.increment_protocol("DHCP");
            }
        }
    }

    fn increment_protocol(&mut self, protocol: &str) {
        *self.protocol_stats.entry(protocol.to_string()).or_insert(0) += 1;
    }

    fn display_statistics(&self) {
        println!("\n=== Packet Capture Statistics ===");
        println!("Interface: {}", self.interface_name);
        println!("Duration: {} seconds", self.capture_duration.as_secs());
        println!("Total packets captured: {}", self.total_packets);
        println!("Packets per second: {:.2}", 
                 self.total_packets as f64 / self.capture_duration.as_secs() as f64);
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_stats {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} packets ({:.2}%)", protocol, count, percentage);
        }
    }
}

pub fn run_analysis(interface: &str, duration: u64) {
    let mut analyzer = PacketAnalyzer::new(interface, duration);
    
    if let Err(e) = analyzer.start_capture() {
        eprintln!("Error during packet capture: {}", e);
    }
}