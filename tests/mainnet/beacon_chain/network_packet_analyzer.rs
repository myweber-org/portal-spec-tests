
use pnet::datalink::{self, Channel::Ethernet};
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

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unsupported channel type"),
        Err(e) => panic!("Failed to create channel: {}", e),
    };

    println!("Starting packet capture on interface: {}", interface_name);
    
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_ethernet_packet(&ethernet_packet);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn process_ethernet_packet(ethernet_packet: &EthernetPacket) {
    match ethernet_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                process_ipv4_packet(&ipv4_packet);
            }
        }
        EtherTypes::Ipv6 => {
            println!("IPv6 packet detected");
        }
        EtherTypes::Arp => {
            println!("ARP packet detected");
        }
        _ => {
            println!("Other Ethernet type: {:?}", ethernet_packet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4_packet: &Ipv4Packet) {
    let source = ipv4_packet.get_source();
    let destination = ipv4_packet.get_destination();
    let protocol = ipv4_packet.get_next_level_protocol();
    
    match protocol {
        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                let src_port = tcp_packet.get_source();
                let dst_port = tcp_packet.get_destination();
                let flags = tcp_packet.get_flags();
                
                println!(
                    "TCP Packet: {}:{} -> {}:{} | Flags: {:?} | Length: {}",
                    source,
                    src_port,
                    destination,
                    dst_port,
                    flags,
                    ipv4_packet.get_total_length()
                );
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
            println!(
                "UDP Packet: {} -> {} | Length: {}",
                source,
                destination,
                ipv4_packet.get_total_length()
            );
        }
        pnet::packet::ip::IpNextHeaderProtocols::Icmp => {
            println!(
                "ICMP Packet: {} -> {} | Length: {}",
                source,
                destination,
                ipv4_packet.get_total_length()
            );
        }
        _ => {
            println!(
                "Other IP protocol ({:?}): {} -> {} | Length: {}",
                protocol,
                source,
                destination,
                ipv4_packet.get_total_length()
            );
        }
    }
}