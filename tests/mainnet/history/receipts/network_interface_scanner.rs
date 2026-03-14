use pcap::{Capture, Device};
use std::error::Error;

pub fn list_interfaces() -> Result<Vec<String>, Box<dyn Error>> {
    let devices = Device::list()?;
    let interface_names: Vec<String> = devices
        .iter()
        .map(|dev| dev.name.clone())
        .collect();
    
    Ok(interface_names)
}

pub fn capture_packets(interface: &str, count: usize) -> Result<Vec<String>, Box<dyn Error>> {
    let mut cap = Capture::from_device(interface)?
        .promisc(true)
        .timeout(1000)
        .open()?;
    
    let mut packets = Vec::new();
    for _ in 0..count {
        if let Ok(packet) = cap.next() {
            packets.push(format!("Packet length: {}", packet.len()));
        }
    }
    
    Ok(packets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_interfaces() {
        let interfaces = list_interfaces();
        assert!(interfaces.is_ok());
        if let Ok(ifaces) = interfaces {
            println!("Found interfaces: {:?}", ifaces);
        }
    }
}