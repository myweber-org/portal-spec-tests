
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

        let mut packet_counter = 0;
        while let Ok(packet) = self.capture.next_packet() {
            println!("Packet {} captured:", packet_counter + 1);
            println!("  Timestamp: {:?}", packet.header.ts);
            println!("  Length: {} bytes", packet.header.len);
            println!("  Captured length: {} bytes", packet.header.caplen);

            self.analyze_packet(&packet.data);

            packet_counter += 1;
            if packet_count > 0 && packet_counter >= packet_count {
                break;
            }
        }

        println!("Captured {} packets", packet_counter);
        Ok(())
    }

    fn analyze_packet(&self, data: &[u8]) {
        if data.len() >= 14 {
            let eth_type = u16::from_be_bytes([data[12], data[13]]);
            match eth_type {
                0x0800 => println!("  Protocol: IPv4"),
                0x0806 => println!("  Protocol: ARP"),
                0x86DD => println!("  Protocol: IPv6"),
                _ => println!("  Protocol: Unknown (0x{:04x})", eth_type),
            }

            if eth_type == 0x0800 && data.len() >= 34 {
                let protocol = data[23];
                match protocol {
                    1 => println!("  Transport: ICMP"),
                    6 => println!("  Transport: TCP"),
                    17 => println!("  Transport: UDP"),
                    _ => println!("  Transport: Unknown ({})", protocol),
                }

                let src_ip = format!("{}.{}.{}.{}", data[26], data[27], data[28], data[29]);
                let dst_ip = format!("{}.{}.{}.{}", data[30], data[31], data[32], data[33]);
                println!("  Source IP: {}", src_ip);
                println!("  Destination IP: {}", dst_ip);
            }
        }
        println!();
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  - {}", device.name);
        if let Some(desc) = device.desc {
            println!("    Description: {}", desc);
        }
    }
    Ok(())
}