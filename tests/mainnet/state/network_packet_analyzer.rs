
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
                Err(e) => eprintln!("Error capturing packet: {}", e),
            }
        }

        println!("Capture completed.");
        Ok(())
    }

    fn analyze_packet(&self, packet: &pcap::Packet) {
        if packet.data.len() >= 14 {
            let eth_header = &packet.data[0..14];
            let eth_type = u16::from_be_bytes([eth_header[12], eth_header[13]]);
            
            match eth_type {
                0x0800 => println!("  Protocol: IPv4"),
                0x0806 => println!("  Protocol: ARP"),
                0x86DD => println!("  Protocol: IPv6"),
                _ => println!("  Protocol: Unknown (0x{:04x})", eth_type),
            }

            if eth_type == 0x0800 && packet.data.len() >= 34 {
                let src_ip = format!(
                    "{}.{}.{}.{}",
                    packet.data[26], packet.data[27], packet.data[28], packet.data[29]
                );
                let dst_ip = format!(
                    "{}.{}.{}.{}",
                    packet.data[30], packet.data[31], packet.data[32], packet.data[33]
                );
                println!("  Source IP: {}", src_ip);
                println!("  Destination IP: {}", dst_ip);

                let protocol = packet.data[23];
                match protocol {
                    6 => println!("  Transport: TCP"),
                    17 => println!("  Transport: UDP"),
                    1 => println!("  Transport: ICMP"),
                    _ => println!("  Transport: Protocol {}", protocol),
                }
            }
        }
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  - {}: {}", device.name, device.desc.unwrap_or_default());
    }
    Ok(())
}extern crate pnet;

use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::env;

fn main() {
    let interface_name = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: {} <interface>", env::args().next().unwrap());
        std::process::exit(1);
    });

    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .unwrap_or_else(|| {
            eprintln!("No such interface: {}", interface_name);
            std::process::exit(1);
        });

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            eprintln!("Unsupported channel type");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error creating channel: {}", e);
            std::process::exit(1);
        }
    };

    println!("Starting packet capture on {}...", interface_name);
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                if let Some(eth_packet) = EthernetPacket::new(packet) {
                    process_ethernet_frame(&eth_packet, packet_count);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}

fn process_ethernet_frame(ethernet: &EthernetPacket, count: u64) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                process_ipv4_packet(&ipv4_packet, count);
            }
        }
        EtherTypes::Ipv6 => {
            if let Some(ipv6_packet) = Ipv6Packet::new(ethernet.payload()) {
                process_ipv6_packet(&ipv6_packet, count);
            }
        }
        _ => {
            println!("[{}] Non-IP packet: {:?}", count, ethernet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet, count: u64) {
    let src = ipv4.get_source();
    let dst = ipv4.get_destination();
    let protocol = ipv4.get_next_level_protocol();

    match protocol {
        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                println!(
                    "[{}] TCP {}:{} -> {}:{} ({} bytes)",
                    count,
                    src,
                    tcp_packet.get_source(),
                    dst,
                    tcp_packet.get_destination(),
                    ipv4.packet().len()
                );
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                println!(
                    "[{}] UDP {}:{} -> {}:{} ({} bytes)",
                    count,
                    src,
                    udp_packet.get_source(),
                    dst,
                    udp_packet.get_destination(),
                    ipv4.packet().len()
                );
            }
        }
        _ => {
            println!(
                "[{}] IPv4 {} -> {} Protocol: {:?} ({} bytes)",
                count,
                src,
                dst,
                protocol,
                ipv4.packet().len()
            );
        }
    }
}

fn process_ipv6_packet(ipv6: &Ipv6Packet, count: u64) {
    let src = ipv6.get_source();
    let dst = ipv6.get_destination();
    let next_header = ipv6.get_next_header();

    println!(
        "[{}] IPv6 {} -> {} NextHeader: {:?} ({} bytes)",
        count,
        src,
        dst,
        next_header,
        ipv6.packet().len()
    );
}