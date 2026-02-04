
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
}