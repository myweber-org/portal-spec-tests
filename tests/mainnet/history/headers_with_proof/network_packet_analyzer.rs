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
use std::fmt;

#[derive(Debug, Clone)]
pub struct MacAddress([u8; 6]);

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl MacAddress {
    pub fn new(bytes: [u8; 6]) -> Self {
        MacAddress(bytes)
    }
}

#[derive(Debug)]
pub struct EthernetFrame {
    pub destination: MacAddress,
    pub source: MacAddress,
    pub ethertype: u16,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn parse(raw_data: &[u8]) -> Option<Self> {
        if raw_data.len() < 14 {
            return None;
        }

        let mut dest_bytes = [0u8; 6];
        dest_bytes.copy_from_slice(&raw_data[0..6]);

        let mut src_bytes = [0u8; 6];
        src_bytes.copy_from_slice(&raw_data[6..12]);

        let ethertype = u16::from_be_bytes([raw_data[12], raw_data[13]]);
        let payload = raw_data[14..].to_vec();

        Some(EthernetFrame {
            destination: MacAddress::new(dest_bytes),
            source: MacAddress::new(src_bytes),
            ethertype,
            payload,
        })
    }

    pub fn is_ipv4(&self) -> bool {
        self.ethertype == 0x0800
    }

    pub fn is_arp(&self) -> bool {
        self.ethertype == 0x0806
    }
}

pub fn analyze_packet(packet_data: &[u8]) -> Result<EthernetFrame, &'static str> {
    EthernetFrame::parse(packet_data).ok_or("Invalid packet data or insufficient length")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ethernet_frame() {
        let mut test_data = vec![0u8; 64];
        test_data[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        test_data[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        test_data[12..14].copy_from_slice(&[0x08, 0x00]);

        let frame = EthernetFrame::parse(&test_data).unwrap();
        assert_eq!(frame.destination.to_string(), "aa:bb:cc:dd:ee:ff");
        assert_eq!(frame.source.to_string(), "11:22:33:44:55:66");
        assert!(frame.is_ipv4());
        assert!(!frame.is_arp());
    }

    #[test]
    fn test_invalid_packet() {
        let short_data = vec![0u8; 10];
        assert!(EthernetFrame::parse(&short_data).is_none());
    }
}