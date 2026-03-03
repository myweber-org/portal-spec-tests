use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
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
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_ethernet_frame(&ethernet_packet, packet_count);
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

fn process_ethernet_frame(ethernet: &EthernetPacket, count: u32) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                process_ipv4_packet(&ipv4_packet, count);
            }
        }
        EtherTypes::Ipv6 => {
            println!("Packet {}: IPv6 packet detected", count);
        }
        EtherTypes::Arp => {
            println!("Packet {}: ARP packet detected", count);
        }
        _ => {
            println!("Packet {}: Unknown ethertype: {:?}", count, ethernet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet, count: u32) {
    let source = ipv4.get_source();
    let destination = ipv4.get_destination();
    let protocol = ipv4.get_next_level_protocol();
    let length = ipv4.get_total_length();

    match protocol {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                let src_port = tcp_packet.get_source();
                let dst_port = tcp_packet.get_destination();
                println!(
                    "Packet {}: TCP | {}:{} -> {}:{} | Length: {}",
                    count, source, src_port, destination, dst_port, length
                );
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                let src_port = udp_packet.get_source();
                let dst_port = udp_packet.get_destination();
                println!(
                    "Packet {}: UDP | {}:{} -> {}:{} | Length: {}",
                    count, source, src_port, destination, dst_port, length
                );
            }
        }
        IpNextHeaderProtocols::Icmp => {
            println!("Packet {}: ICMP | {} -> {} | Length: {}", count, source, destination, length);
        }
        _ => {
            println!(
                "Packet {}: Other Protocol {:?} | {} -> {} | Length: {}",
                count, protocol, source, destination, length
            );
        }
    }
}