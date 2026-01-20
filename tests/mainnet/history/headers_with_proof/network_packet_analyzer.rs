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
                0x0800 => println!("  Ethernet Type: IPv4"),
                0x0806 => println!("  Ethernet Type: ARP"),
                0x86DD => println!("  Ethernet Type: IPv6"),
                _ => println!("  Ethernet Type: Unknown (0x{:04x})", eth_type),
            }
        }
        
        if packet.data.len() >= 34 {
            let ip_header_len = (packet.data[14] & 0x0F) as usize * 4;
            if packet.data.len() >= 14 + ip_header_len {
                let protocol = packet.data[23];
                match protocol {
                    1 => println!("  IP Protocol: ICMP"),
                    6 => println!("  IP Protocol: TCP"),
                    17 => println!("  IP Protocol: UDP"),
                    _ => println!("  IP Protocol: Unknown ({})", protocol),
                }
            }
        }
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  {}: {}", device.name, device.desc.unwrap_or_default());
    }
    Ok(())
}