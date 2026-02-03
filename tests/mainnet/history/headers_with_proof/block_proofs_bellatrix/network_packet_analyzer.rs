
use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    capture: Capture<pcap::Active>,
}

impl PacketAnalyzer {
    pub fn new(interface: &str) -> Result<Self, Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|dev| dev.name == interface)
            .ok_or_else(|| format!("Interface {} not found", interface))?;

        let capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;

        Ok(PacketAnalyzer { capture })
    }

    pub fn start_capture(&mut self, packet_count: i32) -> Result<(), Box<dyn Error>> {
        println!("Starting packet capture on interface...");
        
        for i in 0..packet_count {
            match self.capture.next_packet() {
                Ok(packet) => {
                    println!("Packet {}: {} bytes", i + 1, packet.header.len);
                    self.analyze_packet(&packet);
                }
                Err(e) => eprintln!("Error capturing packet: {}", e),
            }
        }
        
        Ok(())
    }

    fn analyze_packet(&self, packet: &pcap::Packet) {
        let data = packet.data;
        
        if data.len() >= 14 {
            let eth_type = u16::from_be_bytes([data[12], data[13]]);
            
            match eth_type {
                0x0800 => println!("  IPv4 Packet"),
                0x0806 => println!("  ARP Packet"),
                0x86DD => println!("  IPv6 Packet"),
                _ => println!("  Unknown Ethernet Type: 0x{:04x}", eth_type),
            }
        }
        
        if data.len() >= 34 {
            let protocol = data[23];
            match protocol {
                6 => println!("  TCP Protocol"),
                17 => println!("  UDP Protocol"),
                1 => println!("  ICMP Protocol"),
                _ => println!("  Unknown Protocol: {}", protocol),
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
}use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;
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
        Err(e) => panic!("Failed to create datalink channel: {}", e),
    };

    println!("Starting packet capture on interface: {}", interface_name);
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                if let Some(ethernet) = EthernetPacket::new(packet) {
                    analyze_packet(&ethernet);
                }
                
                if packet_count >= 100 {
                    println!("Captured {} packets. Stopping.", packet_count);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to receive packet: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn analyze_packet(ethernet: &EthernetPacket) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                println!(
                    "IPv4 Packet: {} -> {} | Protocol: {}",
                    ipv4.get_source(),
                    ipv4.get_destination(),
                    ipv4.get_next_level_protocol()
                );
                
                if ipv4.get_next_level_protocol() == pnet::packet::ip::IpNextHeaderProtocols::Tcp {
                    if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                        println!(
                            "  TCP: {} -> {} | Flags: {:?}",
                            tcp.get_source(),
                            tcp.get_destination(),
                            tcp.get_flags()
                        );
                    }
                }
            }
        }
        EtherTypes::Arp => {
            println!("ARP Packet detected");
        }
        _ => {
            println!("Other EtherType: {:?}", ethernet.get_ethertype());
        }
    }
}